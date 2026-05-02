use glam::Vec2;
use rore_core::reactive::command::{CommandQueue, UICommand};
use rore_core::reactive::signals::{create_effect, Signal};
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, RenderOutput, Widget};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_render::Instance;
use rore_types::{Align, Color, Style, Val};

pub struct AnimatedBox {
    pub id: String,
    pub width_sig: Signal<f32>,
    pub color_sig: Signal<Color>,
    pub child: Option<Box<dyn Widget>>,
    node_id: Option<NodeId>,
}

impl AnimatedBox {
    pub fn new(id: &str, width_sig: Signal<f32>, color_sig: Signal<Color>) -> Self {
        Self {
            id: id.to_string(),
            width_sig,
            color_sig,
            child: None,
            node_id: None,
        }
    }

    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(w));
        self
    }
}

impl Widget for AnimatedBox {
    fn type_name(&self) -> &'static str {
        "AnimatedBox"
    }
    fn is_interactive(&self) -> bool {
        false
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

        let mut style = Style::default();
        style.width = Val::Px(self.width_sig.get_untracked());
        style.height = Val::Px(150.0);
        style.justify_content = Align::Center;
        style.align_items = Align::Center;

        let taffy_node = engine.new_node(style, &child_nodes);
        let my_id = arena.allocate_node();
        self.node_id = Some(my_id);
        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let c = self.color_sig.get_untracked();
        arena.colors[my_id.0 as usize] = [c.r, c.g, c.b, c.a];

        let id_str = format!("{}_{}", self.id, my_id.0);
        arena.register_id(&id_str, my_id);

        let w_sig = self.width_sig;
        let c_sig = self.color_sig;
        let id_clone = id_str.clone();

        create_effect(move || {
            let mut new_style = Style::default();
            new_style.width = Val::Px(w_sig.get());
            new_style.height = Val::Px(150.0);
            new_style.justify_content = Align::Center;
            new_style.align_items = Align::Center;

            let col = c_sig.get();
            CommandQueue::send(UICommand::UpdateStyle(my_id, new_style));
            CommandQueue::send(UICommand::SetColor(
                id_clone.clone(),
                [col.r, col.g, col.b, col.a],
            ));
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
        let my_id = self.node_id.unwrap();
        let color = state.arena.colors[my_id.0 as usize];

        let inst = Instance {
            position: Vec2::new(layout.x, layout.y),
            size: Vec2::new(layout.width, layout.height),
            color_start: color,
            color_end: color,
            target_color_start: color,
            target_color_end: color,
            gradient_angle: 0.0,
            border_radius: 20.0,
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
            for (i, &child_node) in children.iter().enumerate() {
                if let Some(&child_id) = state.arena.node_map.get(&child_node) {
                    if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                        output.extend(widget_ref.render(
                            engine,
                            state,
                            child_node,
                            Vec2::new(layout.x, layout.y),
                            clip_rect,
                            format!("{}_{}", path, i),
                        ));
                        state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
                    }
                }
            }
        }
        output
    }
}
