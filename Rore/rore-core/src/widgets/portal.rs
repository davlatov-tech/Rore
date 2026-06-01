use crate::state::{FrameworkState, NodeId, UiArena};
use crate::widgets::base::{BuildContext, EventResult, RenderOutput, Widget, WidgetEvent};
use glam::Vec2;
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;

pub struct Portal {
    pub anchor_id: String,
    pub child: Option<Box<dyn Widget>>,
    pub current_child_id: Option<NodeId>,
    pub on_close: Option<Box<dyn FnMut() + Send + 'static>>,
    pub backdrop_color: Option<[f32; 4]>,
    node_id: Option<NodeId>,
}

impl Portal {
    pub fn new(anchor_id: &str) -> Self {
        Self {
            anchor_id: anchor_id.to_string(),
            child: None,
            current_child_id: None,
            on_close: None,
            backdrop_color: Some([0.0, 0.0, 0.0, 0.2]), // Yengil shaffof qora fon (Backdrop)
            node_id: None,
        }
    }

    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(w));
        self
    }

    pub fn on_close<F: FnMut() + Send + 'static>(mut self, f: F) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }

    pub fn transparent_backdrop(mut self) -> Self {
        self.backdrop_color = None;
        self
    }
}

impl Widget for Portal {
    fn type_name(&self) -> &'static str {
        "Portal"
    }

    // Portalning o'zi (Backdrop) interaktiv qobiq!
    fn is_interactive(&self) -> bool {
        true
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let style = Style::default();
        let taffy_node = engine.new_leaf(style);
        let my_id = arena.allocate_node();
        self.node_id = Some(my_id);

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let mut child_id_opt = None;
        if let Some(child) = self.child.take() {
            let child_id = child.build(arena, engine, ctx);
            child_id_opt = Some(child_id);

            if let Some(&child_taffy) = arena.taffy_map.get(&child_id) {
                let _ = engine.taffy.set_children(taffy_node, &[child_taffy]);
            }
        }
        self.current_child_id = child_id_opt;

        // INQILOB: Endi Portalning O'ZI Overlay ro'yxatiga tushadi!
        if !arena.overlays.contains(&taffy_node) {
            arena.overlays.push(taffy_node);
        }
        arena.anchors.insert(taffy_node, self.anchor_id.clone());

        engine.mark_interactive(taffy_node);
        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn handle_event(&mut self, _state: &mut FrameworkState, event: &WidgetEvent) -> EventResult {
        match event {
            WidgetEvent::MouseDown => {
                // Foydalanuvchi Backdrop ga (Dropdown dan tashqariga) bosdi!
                if let Some(cb) = &mut self.on_close {
                    cb();
                    return EventResult::Consumed;
                }
            }
            _ => {}
        }
        EventResult::Ignored
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        _parent_pos: Vec2,
        _clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let my_id = self.node_id.unwrap();

        // 1. BACKDROP CHIZISH
        if let Some(color) = self.backdrop_color {
            let backdrop = rore_render::Instance {
                position: Vec2::ZERO,
                size: state.screen_size,
                color_start: color,
                color_end: color,
                target_color_start: color,
                target_color_end: color,
                gradient_angle: 0.0,
                border_radius: [0.0; 4],
                border_width: [0.0; 4],
                border_color: [0.0; 4],
                target_border_color: [0.0; 4],
                shadow_color: [0.0; 4],
                shadow_offset: Vec2::ZERO,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
                anim_start_time: 0.0,
                anim_duration: 0.0,
            };
            output.sparse_instances.push((my_id.0, backdrop));
        }

        // 2. AQLIY POZITSIYANI QABUL QILISH VA CHILD CHIZISH
        let smart_pos = if let Some(bounds) = state.node_bounds.get(&taffy_node) {
            Vec2::new(bounds[0], bounds[1])
        } else {
            Vec2::ZERO
        };

        if let Some(child_id) = self.current_child_id {
            if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                if let Some(&child_node) = state.arena.taffy_map.get(&child_id) {
                    let child_output = widget_ref.render(
                        engine,
                        state,
                        child_node,
                        smart_pos, // INQILOB: state.rs da hisoblangan mukammal pozitsiya!
                        None,
                        format!("{}_portal", path),
                    );
                    output.extend(child_output);
                }
                state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
            }
        }

        output
    }
}
