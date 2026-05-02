use glam::Vec2;
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{
    BuildContext, DisplayCommand, EventResult, IntoProp, Prop, RenderOutput, Widget, WidgetEvent,
};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::{Color, Style};
use std::sync::{Arc, Mutex};

pub struct Button {
    pub id: String,
    pub style: Style,

    pub normal_color: Prop<Color>,
    pub hover_color: Prop<Color>,
    pub click_color: Prop<Color>,

    pub live_normal: Arc<Mutex<Color>>,
    pub live_hover: Arc<Mutex<Color>>,
    pub live_click: Arc<Mutex<Color>>,

    pub border_radius: f32,
    pub prev_color: [f32; 4],
    pub target_color: [f32; 4],
    pub anim_start_time: f32,
    pub on_click_action: Option<Box<dyn FnMut() + Send>>,

    pub child: Option<Box<dyn Widget>>,
    node_id: Option<NodeId>,
}

impl Button {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            style: Style::default(),

            normal_color: Prop::Dynamic(Box::new(|| {
                if let Some(t) = rore_core::reactive::signals::use_context::<
                    rore_core::reactive::signals::Signal<crate::widgets::theme::Theme>,
                >() {
                    t.get().primary
                } else {
                    Color::hex("#3b82f6")
                }
            })),
            hover_color: Prop::Dynamic(Box::new(|| {
                if let Some(t) = rore_core::reactive::signals::use_context::<
                    rore_core::reactive::signals::Signal<crate::widgets::theme::Theme>,
                >() {
                    t.get().primary_hover
                } else {
                    Color::hex("#60a5fa")
                }
            })),
            click_color: Prop::Dynamic(Box::new(|| {
                if let Some(t) = rore_core::reactive::signals::use_context::<
                    rore_core::reactive::signals::Signal<crate::widgets::theme::Theme>,
                >() {
                    t.get().primary_click
                } else {
                    Color::hex("#2563eb")
                }
            })),

            live_normal: Arc::new(Mutex::new(Color::TRANSPARENT)),
            live_hover: Arc::new(Mutex::new(Color::TRANSPARENT)),
            live_click: Arc::new(Mutex::new(Color::TRANSPARENT)),

            border_radius: 8.0,
            prev_color: [0.0; 4],
            target_color: [0.0; 4],
            anim_start_time: 0.0,
            on_click_action: None,
            child: None,
            node_id: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn colors(
        mut self,
        normal: impl IntoProp<Color>,
        hover: impl IntoProp<Color>,
        click: impl IntoProp<Color>,
    ) -> Self {
        self.normal_color = normal.into_prop();
        self.hover_color = hover.into_prop();
        self.click_color = click.into_prop();
        self
    }

    pub fn corner_radius(mut self, r: f32) -> Self {
        self.border_radius = r;
        self
    }

    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(w));
        self
    }

    pub fn on_click<F: FnMut() + Send + 'static>(mut self, f: F) -> Self {
        self.on_click_action = Some(Box::new(f));
        self
    }
}

impl Widget for Button {
    fn type_name(&self) -> &'static str {
        "Button"
    }
    fn is_interactive(&self) -> bool {
        true
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let mut child_nodes = Vec::new();
        if let Some(child) = self.child.take() {
            let child_id = child.build(arena, engine, ctx);
            if let Some(&t_node) = arena.taffy_map.get(&child_id) {
                child_nodes.push(t_node);
            }
        }

        let taffy_node = engine.new_node(self.style.clone(), &child_nodes);
        let my_id = arena.allocate_node();
        self.node_id = Some(my_id);

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let norm_prop = std::mem::replace(&mut self.normal_color, Prop::Static(Color::TRANSPARENT));
        match norm_prop {
            Prop::Static(c) => *self.live_normal.lock().unwrap() = c,
            Prop::Dynamic(mut f) => {
                let initial = f();
                *self.live_normal.lock().unwrap() = initial;
                let ln = self.live_normal.clone();
                rore_core::reactive::signals::create_effect(move || {
                    let new_c = f();
                    *ln.lock().unwrap() = new_c;
                    rore_core::reactive::command::CommandQueue::send(
                        rore_core::reactive::command::UICommand::MarkDirty(
                            my_id,
                            rore_core::state::DIRTY_COLOR,
                        ),
                    );
                });
            }
        }

        let hover_prop = std::mem::replace(&mut self.hover_color, Prop::Static(Color::TRANSPARENT));
        match hover_prop {
            Prop::Static(c) => *self.live_hover.lock().unwrap() = c,
            Prop::Dynamic(mut f) => {
                let initial = f();
                *self.live_hover.lock().unwrap() = initial;
                let lh = self.live_hover.clone();
                rore_core::reactive::signals::create_effect(move || {
                    *lh.lock().unwrap() = f();
                });
            }
        }

        let click_prop = std::mem::replace(&mut self.click_color, Prop::Static(Color::TRANSPARENT));
        match click_prop {
            Prop::Static(c) => *self.live_click.lock().unwrap() = c,
            Prop::Dynamic(mut f) => {
                let initial = f();
                *self.live_click.lock().unwrap() = initial;
                let lc = self.live_click.clone();
                rore_core::reactive::signals::create_effect(move || {
                    *lc.lock().unwrap() = f();
                });
            }
        }

        let n_col = {
            let c = self.live_normal.lock().unwrap();
            [c.r, c.g, c.b, c.a]
        };
        self.prev_color = n_col;
        self.target_color = n_col;
        arena.colors[my_id.0 as usize] = n_col;

        arena.register_id(&self.id, my_id);
        engine.register_id(&self.id, taffy_node);
        engine.mark_interactive(taffy_node);

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn handle_event(&mut self, state: &mut FrameworkState, event: &WidgetEvent) -> EventResult {
        if let Some(id) = self.node_id {
            let new_target = match event {
                WidgetEvent::HoverEnter => {
                    state.current_cursor_icon = winit::window::CursorIcon::Pointer;
                    let c = self.live_hover.lock().unwrap();
                    Some([c.r, c.g, c.b, c.a])
                }
                WidgetEvent::HoverLeave => {
                    state.current_cursor_icon = winit::window::CursorIcon::Default;
                    let c = self.live_normal.lock().unwrap();
                    Some([c.r, c.g, c.b, c.a])
                }
                WidgetEvent::MouseDown => {
                    let c = self.live_click.lock().unwrap();
                    Some([c.r, c.g, c.b, c.a])
                }
                WidgetEvent::MouseUp | WidgetEvent::Click => {
                    if let Some(cb) = &mut self.on_click_action {
                        cb();
                    }
                    let c = self.live_hover.lock().unwrap();
                    Some([c.r, c.g, c.b, c.a])
                }
                _ => None,
            };

            if let Some(t_color) = new_target {
                if self.target_color != t_color {
                    self.prev_color = self.target_color;
                    self.target_color = t_color;
                    self.anim_start_time = state.global_time;

                    if !state.sparse_update_queue.contains(&id) {
                        state.sparse_update_queue.push(id);
                    }
                    return EventResult::Consumed;
                }
            }
        }
        EventResult::Ignored
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        _clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);
        let my_id = self.node_id.unwrap();

        // INQILOB: WGPU qaramligi uzildi
        let cmd = DisplayCommand::DrawQuad {
            rect: [layout.x, layout.y, layout.width, layout.height],
            color: self.target_color,
            border_radius: self.border_radius,
            border_width: 0.0,
            border_color: [0.0; 4],
            anim_start_time: self.anim_start_time,
            anim_duration: 0.08,
        };
        output.node_commands.push((my_id.0, vec![cmd]));

        if let Ok(children) = engine.taffy.children(taffy_node) {
            if let Some(&child_node) = children.first() {
                if let Some(&child_id) = state.arena.node_map.get(&child_node) {
                    if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                        let child_output = widget_ref.render(
                            engine,
                            state,
                            child_node,
                            Vec2::new(layout.x, layout.y),
                            None,
                            format!("{}_child", path),
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
