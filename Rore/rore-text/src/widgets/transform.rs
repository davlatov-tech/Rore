use glam::Vec2;
use rore_core::reactive::command::{CommandQueue, UICommand};
use rore_core::reactive::signals::{create_effect, Signal};
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, RenderOutput, Widget};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;
use std::cell::Cell;

pub struct TransformBox {
    pub offset_x: Signal<f32>,
    pub offset_y: Signal<f32>,

    pub on_size_signal: Option<Signal<Vec2>>,
    pub child: Option<Box<dyn Widget>>,
    pub style: Style,
    my_id: Option<NodeId>,
    last_size: Cell<Vec2>, // Oldingi o'lchamni eslab qolish uchun
}

impl TransformBox {
    pub fn new(offset_x: Signal<f32>, offset_y: Signal<f32>) -> Self {
        Self {
            offset_x,
            offset_y,
            on_size_signal: None,
            child: None,
            style: Style::default(),
            my_id: None,
            last_size: Cell::new(Vec2::ZERO),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(w));
        self
    }

    // YANGI: Vidjet o'lchami o'zgarganda xabar beriladigan signal
    pub fn on_size(mut self, sig: Signal<Vec2>) -> Self {
        self.on_size_signal = Some(sig);
        self
    }
}

impl Widget for TransformBox {
    fn type_name(&self) -> &'static str {
        "TransformBox"
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
        self.my_id = Some(my_id);
        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let sig_x = self.offset_x;
        let sig_y = self.offset_y;
        let id_clone = my_id;

        create_effect(move || {
            let dx = sig_x.get();
            let dy = sig_y.get();
            CommandQueue::send(UICommand::UpdateTransform(id_clone, dx, dy));
        });

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        _parent_pos: Vec2,
        clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();

        let dx = self.offset_x.get_untracked();
        let dy = self.offset_y.get_untracked();

        // Taffy o'lchab bo'lgan haqiqiy layout
        let layout = engine.get_final_layout(taffy_node, 0.0, 0.0);

        let current_size = Vec2::new(layout.width, layout.height);
        let last = self.last_size.get();

        if (last.x - current_size.x).abs() > 0.5 || (last.y - current_size.y).abs() > 0.5 {
            self.last_size.set(current_size);
            if let Some(sig) = self.on_size_signal {
                // Universal UpdateResource orqali o'lchamni Fizikaga yuboramiz!
                CommandQueue::send(UICommand::UpdateResource(sig.id.0, Box::new(current_size)));
            }
        }

        let new_parent_pos = Vec2::new(dx, dy);

        if let Ok(children) = engine.taffy.children(taffy_node) {
            for (i, &child_node) in children.iter().enumerate() {
                if let Some(&child_id) = state.arena.node_map.get(&child_node) {
                    if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                        let child_output = widget_ref.render(
                            engine,
                            state,
                            child_node,
                            new_parent_pos,
                            clip_rect,
                            format!("{}_{}", path, i),
                        );
                        output.extend(child_output);
                        state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
                    }
                }
            }
        }
        output
    }
}
