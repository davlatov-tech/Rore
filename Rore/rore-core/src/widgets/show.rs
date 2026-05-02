use crate::state::{FrameworkState, NodeId, UiArena};
use crate::widgets::base::{BuildContext, IntoProp, Prop, RenderOutput, Widget};
use glam::Vec2;
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;

pub struct Show {
    pub condition: Prop<bool>,
    pub true_builder: Box<dyn FnMut() -> Box<dyn Widget> + Send>,
    pub false_builder: Box<dyn FnMut() -> Box<dyn Widget> + Send>,

    pub current_child_id: Option<NodeId>,
    pub taffy_node: Option<TaffyNode>,
    pub is_currently_true: bool,
    my_id: Option<NodeId>,
}

impl Show {
    pub fn new<T, F>(condition: impl IntoProp<bool>, true_fn: T, false_fn: F) -> Self
    where
        T: FnMut() -> Box<dyn Widget> + Send + 'static,
        F: FnMut() -> Box<dyn Widget> + Send + 'static,
    {
        Self {
            condition: condition.into_prop(),
            true_builder: Box::new(true_fn),
            false_builder: Box::new(false_fn),
            current_child_id: None,
            taffy_node: None,
            is_currently_true: false,
            my_id: None,
        }
    }
}

impl Widget for Show {
    fn type_name(&self) -> &'static str {
        "Show"
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let condition_val = match &self.condition {
            Prop::Static(b) => *b,
            Prop::Dynamic(f) => {
                let ptr = f.as_ref() as *const _ as *mut dyn FnMut() -> bool;
                unsafe { (*ptr)() }
            }
        };

        self.is_currently_true = condition_val;

        let child_widget = if condition_val {
            (self.true_builder)()
        } else {
            (self.false_builder)()
        };

        // ========================================================
        // YANGI: Bolani xavfsiz Scope bilan quramiz
        // ========================================================
        let (_, child_id) =
            crate::reactive::signals::create_scope(|| child_widget.build(arena, engine, ctx));
        self.current_child_id = Some(child_id);

        let mut child_nodes = Vec::new();
        if let Some(&t_node) = arena.taffy_map.get(&child_id) {
            child_nodes.push(t_node);
        }

        let taffy_node = engine.new_node(Style::default(), &child_nodes);
        let my_id = arena.allocate_node();
        self.my_id = Some(my_id);
        self.taffy_node = Some(taffy_node);

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        if let Prop::Dynamic(mut f) = self.condition {
            crate::reactive::signals::create_effect(move || {
                let new_val = f();
                let action = if new_val { 1 } else { 0 };
                crate::reactive::command::CommandQueue::send(
                    crate::reactive::command::UICommand::RebuildNode(my_id, action),
                );
            });
            self.condition = Prop::Static(condition_val);
        }

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn rebuild(&mut self, state: &mut FrameworkState, engine: &mut LayoutEngine, action: u32) {
        let new_is_true = action == 1;
        if self.is_currently_true == new_is_true {
            return;
        }

        self.is_currently_true = new_is_true;

        if let Some(old_id) = self.current_child_id {
            state.drop_queue.borrow_mut().push(old_id);
            // Drop bo'lganda Arena o'zi Memory'ni tozalaydi!
        }

        let child_widget = if new_is_true {
            (self.true_builder)()
        } else {
            (self.false_builder)()
        };

        let ctx = BuildContext {};

        // YANGI: Bolani xavfsiz Scope bilan quramiz
        let (_, child_id) = crate::reactive::signals::create_scope(|| {
            child_widget.build(&mut state.arena, engine, &ctx)
        });
        self.current_child_id = Some(child_id);

        if let Some(parent_taffy) = self.taffy_node {
            if let Some(&child_taffy) = state.arena.taffy_map.get(&child_id) {
                let _ = engine.taffy.set_children(parent_taffy, &[child_taffy]);
                let _ = engine.taffy.dirty(parent_taffy);
            }
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

        if let Some(child_id) = self.current_child_id {
            if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                if let Some(&child_node) = state.arena.taffy_map.get(&child_id) {
                    let child_output = widget_ref.render(
                        engine,
                        state,
                        child_node,
                        Vec2::new(layout.x, layout.y),
                        clip_rect,
                        format!("{}_show", path),
                    );
                    output.extend(child_output);
                }
                state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
            }
        }
        output
    }
}
