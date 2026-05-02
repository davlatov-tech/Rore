use glam::Vec2;
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, IntoProp, Prop, RenderOutput, Widget};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_render::Instance;
use rore_types::{Color, Style};
use std::sync::{Arc, Mutex};

pub struct UiBox {
    pub id: Option<String>,
    pub style: Prop<Style>,
    pub bg_color: Prop<Color>,
    pub border_radius: f32, // Oddiylik uchun statik
    pub children: Vec<Box<dyn Widget>>,
    pub live_bg: Option<Arc<Mutex<Color>>>,
}

// UiBox ichidagi o'zgarishlar (box_widget.rs)
impl UiBox {
    pub fn new() -> Self {
        Self {
            id: None,
            style: Prop::Static(Style::default()),
            bg_color: Prop::Static(Color::TRANSPARENT), // Default shaffof
            border_radius: 0.0,
            children: vec![],
            live_bg: None,
        }
    }

    pub fn background(mut self) -> Self {
        self.bg_color = Prop::Dynamic(Box::new(|| {
            if let Some(t) = rore_core::reactive::signals::use_context::<
                rore_core::reactive::signals::Signal<crate::widgets::theme::Theme>,
            >() {
                t.get().background
            } else {
                Color::TRANSPARENT
            }
        }));
        self
    }
    pub fn surface(mut self) -> Self {
        self.bg_color = Prop::Dynamic(Box::new(|| {
            if let Some(t) = rore_core::reactive::signals::use_context::<
                rore_core::reactive::signals::Signal<crate::widgets::theme::Theme>,
            >() {
                t.get().surface
            } else {
                Color::TRANSPARENT
            }
        }));
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    pub fn style(mut self, style: impl IntoProp<Style>) -> Self {
        self.style = style.into_prop();
        self
    }
    pub fn bg_color(mut self, color: impl IntoProp<Color>) -> Self {
        self.bg_color = color.into_prop();
        self
    }
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.children.push(Box::new(w));
        self
    }
}

impl Widget for UiBox {
    fn type_name(&self) -> &'static str {
        "UiBox"
    }
    fn is_interactive(&self) -> bool {
        self.id.is_some()
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let mut child_nodes = Vec::new();
        for child in self.children.drain(..) {
            let child_id = child.build(arena, engine, ctx);
            if let Some(&t_node) = arena.taffy_map.get(&child_id) {
                child_nodes.push(t_node);
            }
        }

        let base_style = match &self.style {
            Prop::Static(s) => s.clone(),
            _ => Style::default(),
        };
        let taffy_node = engine.new_node(base_style, &child_nodes);
        let my_id = arena.allocate_node();

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let bg_prop = std::mem::replace(&mut self.bg_color, Prop::Static(Color::TRANSPARENT));
        match bg_prop {
            Prop::Static(c) => {
                arena.colors[my_id.0 as usize] = [c.r, c.g, c.b, c.a];
            }
            Prop::Dynamic(mut f) => {
                let initial = f();
                arena.colors[my_id.0 as usize] = [initial.r, initial.g, initial.b, initial.a];
                let live_c = Arc::new(Mutex::new(initial));
                self.live_bg = Some(live_c.clone());

                rore_core::reactive::signals::create_effect(move || {
                    let new_c = f();
                    *live_c.lock().unwrap() = new_c;
                    rore_core::reactive::command::CommandQueue::send(
                        rore_core::reactive::command::UICommand::MarkDirty(
                            my_id,
                            rore_core::state::DIRTY_COLOR,
                        ),
                    );
                });
            }
        }

        if let Some(id_str) = &self.id {
            arena.register_id(id_str, my_id);
            engine.register_id(id_str, taffy_node);
        }

        if self.is_interactive() {
            engine.mark_interactive(taffy_node);
        }

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
        let my_id = state.arena.node_map.get(&taffy_node).unwrap();

        let mut current_color = state.arena.colors[my_id.0 as usize];
        if let Some(live_bg) = &self.live_bg {
            let c = *live_bg.lock().unwrap();
            current_color = [c.r, c.g, c.b, c.a];
        }

        let inst = Instance {
            position: Vec2::new(layout.x, layout.y),
            size: Vec2::new(layout.width, layout.height),
            color_start: current_color,
            color_end: current_color,
            target_color_start: current_color,
            target_color_end: current_color,
            gradient_angle: 0.0,
            border_radius: self.border_radius,
            border_width: 0.0,
            border_color: [0.0; 4],
            target_border_color: [0.0; 4],
            shadow_color: [0.0; 4],
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            clip_rect: clip_rect.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]),
            anim_start_time: 0.0,
            anim_duration: 0.0,
        };

        output.sparse_instances.push((my_id.0, inst));

        if let Ok(children) = engine.taffy.children(taffy_node) {
            let mut child_i = 0;
            for child_node in children {
                if let Some(&child_id) = state.arena.node_map.get(&child_node) {
                    if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                        let child_layout = engine.get_final_layout(child_node, layout.x, layout.y);

                        let is_visible = if let Some(clip) = clip_rect {
                            let c_left = clip[0];
                            let c_right = clip[0] + clip[2];
                            let c_top = clip[1];
                            let c_bottom = clip[1] + clip[3];

                            let b_left = child_layout.x;
                            let b_right = child_layout.x + child_layout.width;
                            let b_top = child_layout.y;
                            let b_bottom = child_layout.y + child_layout.height;

                            !(b_left >= c_right
                                || b_right <= c_left
                                || b_top >= c_bottom
                                || b_bottom <= c_top)
                        } else {
                            true
                        };

                        if is_visible {
                            let child_path = format!("{}_{}", path, child_i);
                            let child_output = widget_ref.render(
                                engine,
                                state,
                                child_node,
                                Vec2::new(layout.x, layout.y),
                                clip_rect,
                                child_path,
                            );
                            output.extend(child_output);
                        }

                        state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
                    }
                }
                child_i += 1;
            }
        }
        output
    }

    fn visual_overflow(&self) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}

pub struct VirtualList {
    pub id: Option<String>,
    pub style: Prop<Style>,
    pub bg_color: Prop<Color>,
    pub item_height: f32,
    pub children: Vec<Box<dyn Widget>>,
}

impl VirtualList {
    pub fn new(item_height: f32) -> Self {
        Self {
            id: None,
            style: Prop::Static(Style::default()),
            bg_color: Prop::Static(Color::TRANSPARENT),
            item_height,
            children: vec![],
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    pub fn style(mut self, style: impl IntoProp<Style>) -> Self {
        self.style = style.into_prop();
        self
    }
    pub fn bg_color(mut self, color: impl IntoProp<Color>) -> Self {
        self.bg_color = color.into_prop();
        self
    }
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.children.push(Box::new(w));
        self
    }
}

impl Widget for VirtualList {
    fn type_name(&self) -> &'static str {
        "VirtualList"
    }
    fn is_interactive(&self) -> bool {
        self.id.is_some()
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let mut child_nodes = Vec::new();
        for child in self.children.drain(..) {
            let child_id = child.build(arena, engine, ctx);
            if let Some(&t_node) = arena.taffy_map.get(&child_id) {
                child_nodes.push(t_node);
            }
        }

        let base_style = match &self.style {
            Prop::Static(s) => s.clone(),
            _ => Style::default(),
        };
        let taffy_node = engine.new_node(base_style, &child_nodes);
        let my_id = arena.allocate_node();

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let bg_prop = std::mem::replace(&mut self.bg_color, Prop::Static(Color::TRANSPARENT));
        let initial_color = match bg_prop {
            Prop::Static(c) => c,
            Prop::Dynamic(mut f) => f(),
        };

        let idx = my_id.0 as usize;
        if idx < arena.colors.len() {
            arena.colors[idx] = [
                initial_color.r,
                initial_color.g,
                initial_color.b,
                initial_color.a,
            ];
        }

        if let Some(id_str) = &self.id {
            arena.register_id(id_str, my_id);
            engine.register_id(id_str, taffy_node);
        }

        if self.is_interactive() {
            engine.mark_interactive(taffy_node);
        }

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
        let my_id = state.arena.node_map.get(&taffy_node).unwrap();

        let current_color = state.arena.colors[my_id.0 as usize];

        let inst = Instance {
            position: Vec2::new(layout.x, layout.y),
            size: Vec2::new(layout.width, layout.height),
            color_start: current_color,
            color_end: current_color,
            target_color_start: current_color,
            target_color_end: current_color,
            gradient_angle: 0.0,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: [0.0; 4],
            target_border_color: [0.0; 4],
            shadow_color: [0.0; 4],
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            clip_rect: clip_rect.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]),
            anim_start_time: 0.0,
            anim_duration: 0.0,
        };

        output.sparse_instances.push((my_id.0, inst));

        if let Ok(children) = engine.taffy.children(taffy_node) {
            let mut start_idx = 0;
            let mut end_idx = children.len();

            if self.item_height > 0.0 {
                if let Some(clip) = clip_rect {
                    let local_clip_top = clip[1] - layout.y;
                    let local_clip_bottom = (clip[1] + clip[3]) - layout.y;

                    let s = (local_clip_top / self.item_height).floor() as isize;
                    let e = (local_clip_bottom / self.item_height).ceil() as isize;

                    start_idx = s.max(0) as usize;
                    end_idx = (e + 2).max(0) as usize;
                    end_idx = end_idx.min(children.len());
                    if start_idx > end_idx {
                        start_idx = end_idx;
                    }
                }
            }

            for i in start_idx..end_idx {
                let child_node = children[i];
                if let Some(&child_id) = state.arena.node_map.get(&child_node) {
                    if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                        let child_path = format!("{}_{}", path, i);
                        let child_output = widget_ref.render(
                            engine,
                            state,
                            child_node,
                            Vec2::new(layout.x, layout.y),
                            clip_rect,
                            child_path,
                        );
                        output.extend(child_output);
                        state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
                    }
                }
            }
        }
        output
    }

    fn visual_overflow(&self) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
