use crate::widgets::base::{SpatialHashGrid, Widget};
use arboard::Clipboard;
use glam::{Mat4, Vec2};
use rore_layout::Node as TaffyNode;
use rore_types::RoreConfig;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use crate::reactive::command::{CommandQueue, UICommand, COMMAND_RECEIVER};

pub use crate::reactive::signals::{Signal, SignalId};

pub static GLOBAL_CURSOR_IDX: AtomicUsize = AtomicUsize::new(0);

pub const DIRTY_NONE: u8 = 0;
pub const DIRTY_COLOR: u8 = 1 << 0;
pub const DIRTY_LAYOUT: u8 = 1 << 1;
pub const DIRTY_TEXT: u8 = 1 << 2;
pub const DIRTY_ALL: u8 = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32, pub u32);

#[derive(Debug, Clone, Copy)]
pub struct AabbRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub clip_rect: [f32; 4],
    pub taffy_node: TaffyNode,
}

pub struct WakeRegistry {
    locks: HashMap<String, usize>,
    waker: Option<Box<dyn Fn() + Send + Sync>>,
}

impl WakeRegistry {
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
            waker: None,
        }
    }

    pub fn set_waker<F: Fn() + Send + Sync + 'static>(&mut self, waker: F) {
        self.waker = Some(Box::new(waker));
    }

    // INQILOB: Qulfsiz to'g'ridan-to'g'ri OS ni uyg'otish
    pub fn wake(&self) {
        if let Some(waker) = &self.waker {
            waker();
        }
    }

    pub fn acquire(&mut self, id: &str) {
        let current_count = {
            let count = self.locks.entry(id.to_string()).or_insert(0);
            *count += 1;
            *count
        };
        let total_locks = self.locks.len();

        if total_locks == 1 && current_count == 1 {
            self.wake();
        }
    }

    pub fn release(&mut self, id: &str) {
        let mut removed = false;
        if let Some(count) = self.locks.get_mut(id) {
            if *count > 0 {
                *count -= 1;
            }
            if *count == 0 {
                removed = true;
            }
        }
        if removed {
            self.locks.remove(id);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.locks.is_empty()
    }
}

pub struct WidgetHandle {
    pub id: NodeId,
    pub drop_queue: Rc<RefCell<Vec<NodeId>>>,
}

impl Drop for WidgetHandle {
    fn drop(&mut self) {
        self.drop_queue.borrow_mut().push(self.id);
    }
}

pub struct UiArena {
    pub colors: Vec<[f32; 4]>,
    pub transforms: Vec<Mat4>,
    pub opacities: Vec<f32>,
    pub parents: Vec<Option<NodeId>>,
    pub active: Vec<bool>,
    pub free_list: Vec<u32>,
    pub generations: Vec<u32>,
    pub dirty_flags: Vec<u8>,
    pub taffy_map: HashMap<NodeId, TaffyNode>,
    pub node_map: HashMap<TaffyNode, NodeId>,
    pub widgets: Vec<Option<Box<dyn Widget>>>,
    pub overlays: Vec<TaffyNode>,
    pub anchors: HashMap<TaffyNode, String>,
    pub dynamic_nodes: HashMap<String, NodeId>,
    pub node_to_id_str: HashMap<NodeId, String>,
    pub node_scopes: HashMap<NodeId, crate::reactive::signals::ScopeId>,
    pub logical_children: HashMap<TaffyNode, Vec<TaffyNode>>,
}

impl UiArena {
    pub fn new() -> Self {
        Self {
            colors: Vec::new(),
            transforms: Vec::new(),
            opacities: Vec::new(),
            parents: Vec::new(),
            active: Vec::new(),
            free_list: Vec::new(),
            generations: Vec::new(),
            dirty_flags: Vec::new(),
            taffy_map: HashMap::new(),
            node_map: HashMap::new(),
            widgets: Vec::new(),
            overlays: Vec::new(),
            anchors: HashMap::new(),
            dynamic_nodes: HashMap::new(),
            node_to_id_str: HashMap::new(),
            node_scopes: HashMap::new(),
            logical_children: HashMap::new(),
        }
    }

    pub fn allocate_node(&mut self) -> NodeId {
        let node_id = if let Some(reused_idx) = self.free_list.pop() {
            let idx = reused_idx as usize;
            self.active[idx] = true;
            self.dirty_flags[idx] = DIRTY_ALL;
            NodeId(reused_idx, self.generations[idx])
        } else {
            let index = self.colors.len() as u32;
            self.colors.push([0.0; 4]);
            self.transforms.push(Mat4::IDENTITY);
            self.opacities.push(1.0);
            self.parents.push(None);
            self.active.push(true);
            self.widgets.push(None);
            self.generations.push(0);
            self.dirty_flags.push(DIRTY_ALL);
            NodeId(index, 0)
        };
        if let Some(scope_id) = crate::reactive::signals::get_active_scope() {
            self.node_scopes.insert(node_id, scope_id);
        }
        node_id
    }
    pub fn add(&mut self, widget: Box<dyn Widget>) -> NodeId {
        let id = self.allocate_node();
        self.widgets[id.0 as usize] = Some(widget);
        id
    }
    pub fn register_id(&mut self, id: &str, node_id: NodeId) {
        self.dynamic_nodes.insert(id.to_string(), node_id);
        self.node_to_id_str.insert(node_id, id.to_string());
    }
    pub fn get(&self, id: NodeId) -> Option<&dyn Widget> {
        let idx = id.0 as usize;
        if self.active.get(idx).copied() == Some(true) && self.generations[idx] == id.1 {
            self.widgets
                .get(idx)
                .and_then(|w| w.as_ref().map(|b| b.as_ref()))
        } else {
            None
        }
    }
    pub fn remove_node(&mut self, id: NodeId) {
        let idx = id.0 as usize;
        if idx < self.active.len() && self.active[idx] && self.generations[idx] == id.1 {
            self.active[idx] = false;
            self.widgets[idx] = None;
            self.generations[idx] += 1;
            if let Some(taffy_node) = self.taffy_map.remove(&id) {
                self.node_map.remove(&taffy_node);
                self.overlays.retain(|&x| x != taffy_node);
                self.anchors.remove(&taffy_node);
                self.logical_children.remove(&taffy_node);
            }
            if let Some(id_str) = self.node_to_id_str.remove(&id) {
                self.dynamic_nodes.remove(&id_str);
            }
            if let Some(scope_id) = self.node_scopes.remove(&id) {
                crate::reactive::signals::dispose_scope(scope_id);
            }
            self.free_list.push(id.0);
        }
    }
    pub fn clear(&mut self) {
        self.colors.clear();
        self.transforms.clear();
        self.opacities.clear();
        self.parents.clear();
        self.active.clear();
        self.free_list.clear();
        self.generations.clear();
        self.dirty_flags.clear();
        self.taffy_map.clear();
        self.node_map.clear();
        self.widgets.clear();
        self.overlays.clear();
        self.anchors.clear();
        self.dynamic_nodes.clear();
        self.node_to_id_str.clear();
        self.node_scopes.clear();
        self.logical_children.clear();
    }
}

pub struct FrameworkState {
    pub wake_registry: Arc<Mutex<WakeRegistry>>, // <--- INQILOB
    pub arena: UiArena,
    pub draw_order_set: std::collections::HashSet<u32>,
    pub drop_queue: Rc<RefCell<Vec<NodeId>>>,
    pub parent_map: HashMap<TaffyNode, TaffyNode>,
    pub logical_parents: HashMap<TaffyNode, TaffyNode>,
    pub logical_parent_ids: HashMap<TaffyNode, String>,
    pub pending_dirty_nodes: Vec<NodeId>,
    pub sparse_update_queue: Vec<NodeId>,
    pub node_transforms: HashMap<NodeId, Vec2>,
    pub dirty_nodes: Vec<NodeId>,
    pub node_bounds: HashMap<TaffyNode, [f32; 4]>,
    pub aabb_list: Vec<AabbRect>,
    pub spatial_grid: SpatialHashGrid,
    pub current_z_index: i32,
    pub needs_aabb_update: bool,
    pub is_overlay_pass: bool,
    pub cursor_pos: Vec2,
    pub last_cursor_pos: Option<Vec2>,
    pub hovered_node: Option<TaffyNode>,
    pub focused_node: Option<TaffyNode>,
    pub active_node: Option<TaffyNode>,
    pub clipboard: Option<std::sync::Mutex<Clipboard>>,
    pub full_redraw: bool,
    pub dirty_rect: Option<[u32; 4]>,
    pub current_cursor_icon: winit::window::CursorIcon,
    pub global_time: f32,
    pub config: RoreConfig,
    pub current_draw_order: Vec<u32>,
    pub pending_rebuilds: Vec<(NodeId, u32)>,
}

impl FrameworkState {
    pub fn new(config: RoreConfig, wake_registry: Arc<Mutex<WakeRegistry>>) -> Self {
        CommandQueue::init();
        let clipboard = Clipboard::new().ok().map(|c| std::sync::Mutex::new(c));
        Self {
            wake_registry,
            arena: UiArena::new(),
            drop_queue: Rc::new(RefCell::new(Vec::new())),
            parent_map: HashMap::new(),
            logical_parents: HashMap::new(),
            logical_parent_ids: HashMap::new(),
            pending_dirty_nodes: Vec::new(),
            sparse_update_queue: Vec::new(),
            node_transforms: HashMap::new(),
            dirty_nodes: Vec::new(),
            node_bounds: HashMap::new(),
            aabb_list: Vec::new(),
            spatial_grid: SpatialHashGrid::new(),
            current_z_index: 0,
            needs_aabb_update: false,
            is_overlay_pass: false,
            cursor_pos: Vec2::ZERO,
            last_cursor_pos: None,
            hovered_node: None,
            focused_node: None,
            active_node: None,
            clipboard,
            full_redraw: true,
            dirty_rect: None,
            current_cursor_icon: winit::window::CursorIcon::Default,
            global_time: 0.0,
            config,
            current_draw_order: Vec::new(),
            pending_rebuilds: Vec::new(),
            draw_order_set: std::collections::HashSet::new(),
        }
    }

    pub fn get_parent_pos(&self, engine: &rore_layout::LayoutEngine, node: TaffyNode) -> Vec2 {
        if let Some(bounds) = self.node_bounds.get(&node) {
            if let Ok(layout) = engine.taffy.layout(node) {
                return Vec2::new(bounds[0] - layout.location.x, bounds[1] - layout.location.y);
            }
        }
        Vec2::ZERO
    }

    pub fn get_clip_rect(&self, node: TaffyNode) -> Option<[f32; 4]> {
        self.aabb_list
            .iter()
            .find(|a| a.taffy_node == node)
            .map(|a| a.clip_rect)
    }

    pub fn process_commands(&mut self, engine: &mut rore_layout::LayoutEngine) {
        if let Some(rx_mutex) = COMMAND_RECEIVER.get() {
            if let Ok(rx) = rx_mutex.try_lock() {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        UICommand::SetColor(id_str, color) => {
                            if let Some(&node_id) = self.arena.dynamic_nodes.get(&id_str) {
                                let idx = node_id.0 as usize;
                                if idx < self.arena.colors.len()
                                    && self.arena.generations[idx] == node_id.1
                                {
                                    self.arena.colors[idx] = color;
                                    if !self.sparse_update_queue.contains(&node_id) {
                                        self.sparse_update_queue.push(node_id);
                                    }
                                    self.mark_dirty_with_flag(node_id, DIRTY_COLOR);
                                }
                            }
                        }
                        UICommand::UpdateText(node_id, _new_text) => {
                            if !self.sparse_update_queue.contains(&node_id) {
                                self.sparse_update_queue.push(node_id);
                            }
                            self.mark_dirty_with_flag(node_id, DIRTY_TEXT);
                            self.request_redraw();
                        }
                        UICommand::MarkDirty(node_id, flag) => {
                            if !self.sparse_update_queue.contains(&node_id) {
                                self.sparse_update_queue.push(node_id);
                            }
                            self.mark_dirty_with_flag(node_id, flag);
                            if flag != DIRTY_COLOR {
                                self.request_redraw();
                            }
                        }
                        UICommand::RebuildNode(node_id, action) => {
                            self.pending_rebuilds.push((node_id, action));
                            self.request_redraw();
                        }
                        UICommand::UpdateStyle(node_id, new_style) => {
                            if let Some(&taffy_node) = self.arena.taffy_map.get(&node_id) {
                                engine.update_style(taffy_node, new_style);
                                if !self.sparse_update_queue.contains(&node_id) {
                                    self.sparse_update_queue.push(node_id);
                                }
                                self.mark_dirty_with_flag(node_id, DIRTY_LAYOUT);
                                self.request_redraw();
                            }
                        }
                        UICommand::UpdateResource(sig_id, boxed_val) => {
                            crate::reactive::signals::set_signal_any(
                                crate::reactive::signals::SignalId(sig_id),
                                boxed_val,
                            );
                            crate::reactive::signals::process_pending_effects();
                            self.request_redraw();
                        }
                        UICommand::UpdateTransform(node_id, dx, dy) => {
                            self.node_transforms.insert(node_id, Vec2::new(dx, dy));
                            self.needs_aabb_update = true;
                            if !self.sparse_update_queue.contains(&node_id) {
                                self.sparse_update_queue.push(node_id);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn mark_dirty(&mut self, node: NodeId) {
        self.mark_dirty_with_flag(node, DIRTY_ALL);
    }
    pub fn mark_dirty_with_flag(&mut self, node: NodeId, flag: u8) {
        let idx = node.0 as usize;
        if idx < self.arena.dirty_flags.len() && self.arena.generations[idx] == node.1 {
            self.arena.dirty_flags[idx] |= flag;
            if !self.pending_dirty_nodes.contains(&node) {
                self.pending_dirty_nodes.push(node);
            }
        }
    }
    pub fn clear_dirty_flags(&mut self) {
        for flag in &mut self.arena.dirty_flags {
            *flag = DIRTY_NONE;
        }
    }

    pub fn update_aabbs(
        &mut self,
        engine: &rore_layout::LayoutEngine,
        root: TaffyNode,
        force: bool,
    ) {
        let mut needs_rebuild = force;
        if !needs_rebuild {
            if self.arena.taffy_map.len() != self.node_bounds.len() {
                needs_rebuild = true;
            } else {
                needs_rebuild = self.check_layout_changed(engine, root, Vec2::ZERO, false);
            }
        }
        if !needs_rebuild {
            return;
        }

        self.arena.node_map.clear();
        for (id, node) in &self.arena.taffy_map {
            self.arena.node_map.insert(*node, *id);
        }
        self.aabb_list.clear();
        self.spatial_grid.clear();
        self.current_z_index = 0;
        self.node_bounds.clear();
        self.parent_map.clear();
        self.is_overlay_pass = false;

        let initial_clip = [-10000.0, -10000.0, 20000.0, 20000.0];
        self.build_aabb_recursive(engine, root, Vec2::ZERO, initial_clip);
    }

    fn check_layout_changed(
        &self,
        engine: &rore_layout::LayoutEngine,
        node: TaffyNode,
        parent_pos: Vec2,
        is_overlay_pass: bool,
    ) -> bool {
        if !is_overlay_pass && self.arena.overlays.contains(&node) {
            return false;
        }
        if let Some(&node_id) = self.arena.node_map.get(&node) {
            if self.sparse_update_queue.contains(&node_id) {
                let flags = self.arena.dirty_flags[node_id.0 as usize];
                if (flags & DIRTY_LAYOUT) != 0 || (flags & DIRTY_ALL) != 0 {
                    return true;
                }
            }
        }
        let layout = engine.get_final_layout(node, parent_pos.x, parent_pos.y);
        let mut final_x = layout.x;
        let mut final_y = layout.y;
        if let Some(&node_id) = self.arena.node_map.get(&node) {
            if let Some(offset) = self.node_transforms.get(&node_id) {
                final_x = offset.x;
                final_y = offset.y;
            }
        }
        if let Some(ob) = self.node_bounds.get(&node) {
            if (ob[0] - final_x).abs() > 0.5
                || (ob[1] - final_y).abs() > 0.5
                || (ob[2] - layout.width).abs() > 0.5
                || (ob[3] - layout.height).abs() > 0.5
            {
                return true;
            }
        } else {
            return true;
        }

        let children_parent_pos = Vec2::new(final_x, final_y);
        if let Ok(children) = engine.taffy.children(node) {
            for child in children {
                if self.check_layout_changed(engine, child, children_parent_pos, is_overlay_pass) {
                    return true;
                }
            }
        }
        false
    }

    fn build_aabb_recursive(
        &mut self,
        engine: &rore_layout::LayoutEngine,
        node: TaffyNode,
        parent_pos: Vec2,
        current_clip: [f32; 4],
    ) {
        if !self.is_overlay_pass && self.arena.overlays.contains(&node) {
            return;
        }
        let layout = engine.get_final_layout(node, parent_pos.x, parent_pos.y);
        let mut pos = Vec2::new(layout.x, layout.y);

        if let Some(&node_id) = self.arena.node_map.get(&node) {
            if let Some(offset) = self.node_transforms.get(&node_id) {
                pos.x = offset.x;
                pos.y = offset.y;
            }
        }
        self.node_bounds
            .insert(node, [pos.x, pos.y, layout.width, layout.height]);

        let rect = AabbRect {
            x: pos.x,
            y: pos.y,
            w: layout.width,
            h: layout.height,
            clip_rect: current_clip,
            taffy_node: node,
        };
        self.aabb_list.push(rect);

        self.current_z_index += 1;
        if let Some(&node_id) = self.arena.node_map.get(&node) {
            self.spatial_grid.insert(
                node_id,
                [pos.x, pos.y, layout.width, layout.height],
                self.current_z_index,
            );
        }

        let next_clip = current_clip;

        let children_parent_pos = Vec2::new(pos.x, pos.y);
        if let Ok(children) = engine.taffy.children(node) {
            for child in children {
                self.parent_map.insert(child, node);
                self.build_aabb_recursive(engine, child, children_parent_pos, next_clip);
            }
        }
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<TaffyNode> {
        let items = self.spatial_grid.query_point(x, y);
        for item in items {
            if let Some(&taffy_node) = self.arena.taffy_map.get(&item.node_id) {
                if let Some(aabb) = self.aabb_list.iter().find(|a| a.taffy_node == taffy_node) {
                    let cx1 = aabb.clip_rect[0];
                    let cy1 = aabb.clip_rect[1];
                    let cx2 = cx1 + aabb.clip_rect[2];
                    let cy2 = cy1 + aabb.clip_rect[3];

                    if x >= cx1 && x <= cx2 && y >= cy1 && y <= cy2 {
                        let is_interactive = self
                            .arena
                            .get(item.node_id)
                            .map(|w| w.is_interactive())
                            .unwrap_or(false);
                        let has_id = self.arena.node_to_id_str.contains_key(&item.node_id);
                        if is_interactive || has_id {
                            return Some(taffy_node);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_event_bubble_chain(&self, start_node: TaffyNode) -> Vec<TaffyNode> {
        let mut chain = Vec::new();
        let mut current = Some(start_node);
        while let Some(node) = current {
            chain.push(node);
            if let Some(logical) = self.logical_parents.get(&node) {
                current = Some(*logical);
            } else if let Some(target_id) = self.logical_parent_ids.get(&node) {
                if let Some(target_node_id) = self.arena.dynamic_nodes.get(target_id) {
                    current = self.arena.taffy_map.get(target_node_id).copied();
                } else {
                    current = self.parent_map.get(&node).copied();
                }
            } else {
                current = self.parent_map.get(&node).copied();
            }
        }
        chain
    }

    pub fn process_drop_queue(&mut self, engine: &rore_layout::LayoutEngine) {
        let mut to_remove = Vec::new();
        {
            let mut queue = self.drop_queue.borrow_mut();
            to_remove.append(&mut queue);
        }
        let mut i = 0;
        while i < to_remove.len() {
            let id = to_remove[i];
            self.node_transforms.remove(&id);
            if let Some(&taffy_node) = self.arena.taffy_map.get(&id) {
                if let Ok(children) = engine.taffy.children(taffy_node) {
                    for child_node in children {
                        if let Some(&child_id) = self.arena.node_map.get(&child_node) {
                            if !to_remove.contains(&child_id) {
                                to_remove.push(child_id);
                            }
                        }
                    }
                }
            }
            i += 1;
        }
        for id in to_remove {
            self.arena.remove_node(id);
        }
    }

    pub fn add_damage(&mut self, rect: [f32; 4]) {
        let x1 = rect[0].max(0.0) as u32;
        let y1 = rect[1].max(0.0) as u32;
        let w = rect[2].max(0.0) as u32;
        let h = rect[3].max(0.0) as u32;
        if w == 0 || h == 0 {
            return;
        }
        if let Some(existing) = self.dirty_rect {
            let min_x = x1.min(existing[0]);
            let min_y = y1.min(existing[1]);
            let max_x = (x1 + w).max(existing[0] + existing[2]);
            let max_y = (y1 + h).max(existing[1] + existing[3]);
            self.dirty_rect = Some([min_x, min_y, max_x - min_x, max_y - min_y]);
        } else {
            self.dirty_rect = Some([x1, y1, w, h]);
        }
    }

    pub fn request_redraw(&mut self) {
        self.full_redraw = true;
    }
    pub fn update_cursor(&mut self, x: f32, y: f32) {
        self.last_cursor_pos = Some(self.cursor_pos);
        self.cursor_pos = Vec2::new(x, y);
    }
}
