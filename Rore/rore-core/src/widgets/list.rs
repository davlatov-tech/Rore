use crate::reactive::command::{CommandQueue, UICommand};
use crate::reactive::signals::Signal;
use crate::state::{FrameworkState, NodeId, UiArena};
use crate::widgets::base::{BuildContext, RenderOutput, Widget};
use glam::Vec2;
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::{FlexDirection, Position, Style, Val};
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; // INQILOB 1: Multithreading xavfsizligi!

// =====================================================================
// 1. FOR LIST (Kichik ro'yxatlar uchun)
// =====================================================================
pub struct ForList<T: Clone + PartialEq + 'static> {
    pub items: Signal<Vec<T>>,
    pub builder: Box<dyn FnMut(T) -> Box<dyn Widget> + Send>,
    pub style: Style,
    pub current_items: Vec<T>,
    pub child_nodes: Vec<NodeId>,
    pub taffy_node: Option<TaffyNode>,
    my_id: Option<NodeId>,
}

impl<T: Clone + PartialEq + 'static> ForList<T> {
    pub fn new<F>(items: Signal<Vec<T>>, builder: F) -> Self
    where
        F: FnMut(T) -> Box<dyn Widget> + Send + 'static,
    {
        let mut default_style = Style::default();
        default_style.flex_direction = FlexDirection::Column;
        default_style.width = Val::Percent(100.0);

        Self {
            items,
            builder: Box::new(builder),
            style: default_style,
            current_items: Vec::new(),
            child_nodes: Vec::new(),
            taffy_node: None,
            my_id: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<T: Clone + PartialEq + Send + 'static> Widget for ForList<T> {
    fn type_name(&self) -> &'static str {
        "ForList"
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let mut t_children = Vec::new();
        let initial_items = self.items.get_untracked();
        self.current_items = initial_items.clone();

        for item in initial_items {
            let child_widget = (self.builder)(item);
            let (_, child_id) =
                crate::reactive::signals::create_scope(|| child_widget.build(arena, engine, ctx));
            self.child_nodes.push(child_id);
            if let Some(&t_node) = arena.taffy_map.get(&child_id) {
                t_children.push(t_node);
            }
        }

        let taffy_node = engine.new_node(self.style.clone(), &t_children);

        let my_id = arena.allocate_node();
        self.my_id = Some(my_id);
        self.taffy_node = Some(taffy_node);
        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let sig = self.items;
        crate::reactive::signals::create_effect(move || {
            let _ = sig.get();
            CommandQueue::send(UICommand::RebuildNode(my_id, 2));
        });

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn rebuild(&mut self, state: &mut FrameworkState, engine: &mut LayoutEngine, _action: u32) {
        let new_items = self.items.get_untracked();

        let mut prefix = 0;
        let mut suffix = 0;
        let old_len = self.current_items.len();
        let new_len = new_items.len();

        while prefix < old_len
            && prefix < new_len
            && self.current_items[prefix] == new_items[prefix]
        {
            prefix += 1;
        }
        while suffix < (old_len - prefix)
            && suffix < (new_len - prefix)
            && self.current_items[old_len - 1 - suffix] == new_items[new_len - 1 - suffix]
        {
            suffix += 1;
        }

        let old_end = old_len - suffix;
        let new_end = new_len - suffix;

        for i in prefix..old_end {
            let old_id = self.child_nodes[i];
            state.drop_queue.borrow_mut().push(old_id);
        }

        let mut new_middle_nodes = Vec::new();
        for i in prefix..new_end {
            let item = new_items[i].clone();
            let child_widget = (self.builder)(item);
            let ctx = BuildContext {};
            let (_, child_id) = crate::reactive::signals::create_scope(|| {
                child_widget.build(&mut state.arena, engine, &ctx)
            });
            new_middle_nodes.push(child_id);
        }

        let mut next_child_nodes = Vec::with_capacity(new_len);
        next_child_nodes.extend_from_slice(&self.child_nodes[..prefix]);
        next_child_nodes.extend(new_middle_nodes);
        next_child_nodes.extend_from_slice(&self.child_nodes[old_end..]);

        self.child_nodes = next_child_nodes;
        self.current_items = new_items;

        if let Some(parent_taffy) = self.taffy_node {
            let mut taffy_children = Vec::with_capacity(self.child_nodes.len());
            for &child_id in &self.child_nodes {
                if let Some(&t_node) = state.arena.taffy_map.get(&child_id) {
                    taffy_children.push(t_node);
                }
            }
            let _ = engine.taffy.set_children(parent_taffy, &taffy_children);
            let _ = engine.taffy.dirty(parent_taffy);
        }
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);

        for (i, &child_id) in self.child_nodes.iter().enumerate() {
            if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                if let Some(&child_node) = state.arena.taffy_map.get(&child_id) {
                    let child_output = widget_ref.render(
                        engine,
                        state,
                        child_node,
                        Vec2::new(layout.x, layout.y),
                        clip_rect,
                        format!("{}_for_{}", path, i),
                    );
                    output.extend(child_output);
                }
                state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
            }
        }
        output
    }
}

// =====================================================================
// 2. INQILOB: A.R.T.E (Absolute Real-Time Engine) VIRTUAL LIST
// Hech qanday taxminlarsiz! O'zini o'zi o'rgatadi.
// =====================================================================
pub struct VirtualList<T: Clone + PartialEq + Default + Send + Sync + 'static> {
    pub items: Signal<Vec<T>>,
    pub scroll_y: Signal<f32>,
    pub buffer_size: usize,
    pub builder: Box<dyn FnMut(Signal<T>) -> Box<dyn Widget> + Send>,

    pub wrapper_nodes: Vec<NodeId>,
    pub item_signals: Vec<Signal<T>>,
    pub index_signals: Vec<Signal<usize>>,

    // Xavfsiz, Multi-threaded kesh
    pub heights_cache: Arc<Mutex<HashMap<usize, f32>>>,
    pub force_recalc: Signal<u32>,

    pub taffy_node: Option<TaffyNode>,
    my_id: Option<NodeId>,
}

impl<T: Clone + PartialEq + Default + Send + Sync + 'static> VirtualList<T> {
    // API dan estimated_height butunlay olib tashlandi!
    pub fn new<F>(items: Signal<Vec<T>>, scroll_y: Signal<f32>, builder: F) -> Self
    where
        F: FnMut(Signal<T>) -> Box<dyn Widget> + Send + 'static,
    {
        Self {
            items,
            scroll_y,
            buffer_size: 30,
            builder: Box::new(builder),
            wrapper_nodes: Vec::new(),
            item_signals: Vec::new(),
            index_signals: Vec::new(),
            heights_cache: Arc::new(Mutex::new(HashMap::new())),
            force_recalc: Signal::new(0),
            taffy_node: None,
            my_id: None,
        }
    }

    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}

impl<T: Clone + PartialEq + Default + Send + Sync + 'static> Widget for VirtualList<T> {
    fn type_name(&self) -> &'static str {
        "VirtualList"
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let mut t_children = Vec::new();

        for _ in 0..self.buffer_size {
            let sig = Signal::new(T::default());
            let idx_sig = Signal::new(0);
            self.item_signals.push(sig);
            self.index_signals.push(idx_sig);

            let child_widget = (self.builder)(sig);
            let (_, child_id) =
                crate::reactive::signals::create_scope(|| child_widget.build(arena, engine, ctx));

            let mut wrap_style = Style::default();
            wrap_style.position = Position::Absolute;
            wrap_style.inset.top = Val::Px(-10000.0);
            wrap_style.inset.left = Val::Px(0.0);
            wrap_style.width = Val::Percent(100.0);
            wrap_style.height = Val::Auto; // Real-time measurement uchun Auto!

            let wrap_taffy = engine.new_node(wrap_style, &[arena.taffy_map[&child_id]]);
            let wrap_id = arena.allocate_node();
            arena.taffy_map.insert(wrap_id, wrap_taffy);
            arena.node_map.insert(wrap_taffy, wrap_id);

            arena.widgets[wrap_id.0 as usize] = Some(Box::new(VirtualListWrapper {
                child_id,
                index_signal: idx_sig,
                heights_cache: self.heights_cache.clone(),
                force_recalc: self.force_recalc,
            }));

            t_children.push(wrap_taffy);
            self.wrapper_nodes.push(wrap_id);
        }

        let mut container_style = Style::default();
        container_style.width = Val::Percent(100.0);
        // Boshida nol bo'lib turadi, yadro uni avtomat o'zi topadi
        container_style.height = Val::Px(0.0);

        let taffy_node = engine.new_node(container_style, &t_children);
        let my_id = arena.allocate_node();
        self.my_id = Some(my_id);
        self.taffy_node = Some(taffy_node);
        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let sig_scroll = self.scroll_y;
        let sig_items = self.items;
        let item_signals = self.item_signals.clone();
        let index_signals = self.index_signals.clone();
        let wrapper_nodes = self.wrapper_nodes.clone();
        let buffer_size = self.buffer_size;
        let container_id = my_id;

        let last_total_height = core::cell::Cell::new(0.0);
        let force_recalc = self.force_recalc;
        let cache_arc = self.heights_cache.clone();

        // INQILOB: SELF-LEARNING HEURISTICS (O'zini o'zi o'rgatish)
        crate::reactive::signals::create_effect(move || {
            let _ = force_recalc.get();
            let sy = sig_scroll.get();
            let items = sig_items.get();
            let cache = cache_arc.lock().unwrap();

            // 1. Dinamik O'rtacha Qiymatni (Dynamic Average) hisoblash
            let mut total_known_height = 0.0;
            let mut known_count = 0;
            for &h in cache.values() {
                total_known_height += h;
                known_count += 1;
            }

            // Yadro ekranda hech bo'lmasa 1 ta element ko'ringuncha 40px ni ushlab turadi,
            // keyin esa 100% haqiqiy, real-time matematik o'rtacha qiymatga o'tadi!
            let dynamic_average = if known_count > 0 {
                total_known_height / known_count as f32
            } else {
                40.0
            };

            // 2. Haqiqiy balandliklar prefiksini (Offset) hisoblaymiz
            let mut total_height = 0.0;
            let mut offsets = Vec::with_capacity(items.len());
            for i in 0..items.len() {
                offsets.push(total_height);
                total_height += cache.get(&i).copied().unwrap_or(dynamic_average);
            }

            // 3. Arvoh oyna balandligini moslash
            if (total_height - last_total_height.get()).abs() > 1.0 {
                last_total_height.set(total_height);
                let mut s = Style::default();
                s.width = Val::Percent(100.0);
                s.height = Val::Px(total_height);
                CommandQueue::send(UICommand::UpdateStyle(container_id, s));
            }

            // 4. Scroll pozitsiyasiga mos start indeksni topish
            let start_idx = match offsets.binary_search_by(|probe| probe.partial_cmp(&sy).unwrap())
            {
                Ok(idx) => idx,
                Err(idx) => idx.saturating_sub(1),
            };

            // 5. Ekranga sig'adigan qutilarni joyiga qo'yish
            for i in 0..buffer_size {
                let idx = start_idx + i;
                let wrap_id = wrapper_nodes[i];

                let mut s = Style::default();
                s.position = Position::Absolute;
                s.inset.left = Val::Px(0.0);
                s.width = Val::Percent(100.0);
                s.height = Val::Auto;

                if idx < items.len() {
                    item_signals[i].set(items[idx].clone());
                    index_signals[i].set(idx);
                    s.inset.top = Val::Px(offsets[idx]);
                } else {
                    s.inset.top = Val::Px(-10000.0);
                }

                CommandQueue::send(UICommand::UpdateStyle(wrap_id, s));
            }
        });

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);

        for (i, &wrap_id) in self.wrapper_nodes.iter().enumerate() {
            if let Some(widget_ref) = state.arena.widgets[wrap_id.0 as usize].take() {
                if let Some(&wrap_node) = state.arena.taffy_map.get(&wrap_id) {
                    let child_output = widget_ref.render(
                        engine,
                        state,
                        wrap_node,
                        Vec2::new(layout.x, layout.y),
                        clip_rect,
                        format!("{}_virt_{}", path, i),
                    );
                    output.extend(child_output);
                }
                state.arena.widgets[wrap_id.0 as usize] = Some(widget_ref);
            }
        }
        output
    }
}

// =====================================================================
// Ayg'oqchi (Spy) Qobiq - Taffy'ning hisobini o'g'irlaydi
// =====================================================================
struct VirtualListWrapper {
    child_id: NodeId,
    index_signal: Signal<usize>,
    heights_cache: Arc<Mutex<HashMap<usize, f32>>>,
    force_recalc: Signal<u32>,
}

impl Widget for VirtualListWrapper {
    fn type_name(&self) -> &'static str {
        "VirtualListWrapper"
    }
    fn build(self: Box<Self>, _: &mut UiArena, _: &mut LayoutEngine, _: &BuildContext) -> NodeId {
        unreachable!()
    }
    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);

        let current_idx = self.index_signal.get_untracked();

        let should_update = {
            let cache = self.heights_cache.lock().unwrap();
            let old_h = cache.get(&current_idx).copied().unwrap_or(0.0);
            (layout.height - old_h).abs() > 0.5 && layout.height > 0.0
        };

        if should_update {
            self.heights_cache
                .lock()
                .unwrap()
                .insert(current_idx, layout.height);
            self.force_recalc.set(self.force_recalc.get_untracked() + 1);
        }

        if let Some(widget_ref) = state.arena.widgets[self.child_id.0 as usize].take() {
            if let Some(&child_node) = state.arena.taffy_map.get(&self.child_id) {
                let child_output = widget_ref.render(
                    engine,
                    state,
                    child_node,
                    Vec2::new(layout.x, layout.y),
                    clip_rect,
                    format!("{}_wrap", path),
                );
                output.extend(child_output);
            }
            state.arena.widgets[self.child_id.0 as usize] = Some(widget_ref);
        }
        output
    }
}
