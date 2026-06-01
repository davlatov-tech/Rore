use glam::Vec2;
use rore_core::reactive::command::{CommandQueue, UICommand};
use rore_core::reactive::signals::{
    create_effect, create_scope, create_signal_untracked, get_signal_untyped, set_signal_untyped,
    SignalId,
};
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, RenderOutput, Widget};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub static CURRENT_ROUTE: RefCell<Option<SignalId>> = RefCell::new(None);
}

pub fn navigate(path: &str) {
    CURRENT_ROUTE.with(|cr| {
        if let Some(sig_id) = *cr.borrow() {
            set_signal_untyped(sig_id, path.to_string());
        }
    });
}

pub struct Router {
    pub routes: HashMap<String, Box<dyn FnMut() -> Box<dyn Widget> + Send>>,
    pub current_child_id: Option<NodeId>,
    pub taffy_node: Option<TaffyNode>,
    pub current_path: String,
    my_id: Option<NodeId>,
}

impl Router {
    pub fn new(initial_path: &str) -> Self {
        // Dastlabki yo'l (path) uchun signal yaratamiz
        let sig = create_signal_untracked(initial_path.to_string());
        CURRENT_ROUTE.with(|cr| *cr.borrow_mut() = Some(sig.id));

        Self {
            routes: HashMap::new(),
            current_child_id: None,
            taffy_node: None,
            current_path: initial_path.to_string(),
            my_id: None,
        }
    }

    pub fn route<F>(mut self, path: &str, builder: F) -> Self
    where
        F: FnMut() -> Box<dyn Widget> + Send + 'static,
    {
        self.routes.insert(path.to_string(), Box::new(builder));
        self
    }
}

impl Widget for Router {
    fn type_name(&self) -> &'static str {
        "Router"
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let sig_id = CURRENT_ROUTE.with(|cr| cr.borrow().unwrap());

        let child_widget = if let Some(builder) = self.routes.get_mut(&self.current_path) {
            builder()
        } else {
            // Topilmasa, birinchisini ochamiz
            (self.routes.values_mut().next().unwrap())()
        };

        // YANGI SAHIFA UCHUN TOZA SCOPE
        let (_, child_id) = create_scope(|| child_widget.build(arena, engine, ctx));
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

        // YO'L (PATH) O'ZGARISHINI KUZATUVCHI EFFEKT
        create_effect(move || {
            let _new_path: String = get_signal_untyped(sig_id).unwrap();
            // Dvigatelga Router o'zgarganini aytamiz (0 harakati bilan)
            CommandQueue::send(UICommand::RebuildNode(my_id, 0));
        });

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn rebuild(&mut self, state: &mut FrameworkState, engine: &mut LayoutEngine, _action: u32) {
        let sig_id = CURRENT_ROUTE.with(|cr| cr.borrow().unwrap());
        let new_path: String = get_signal_untyped(sig_id).unwrap_or_default();

        if self.current_path == new_path {
            return;
        }
        self.current_path = new_path.clone();

        // 1. ESKI SAHIFANI O'LDIRISH (Garbage Collect)
        if let Some(old_id) = self.current_child_id {
            state.drop_queue.borrow_mut().push(old_id);
        }

        // 2. YANGI SAHIFANI DANGASA (Lazy) YASASH
        let child_widget = if let Some(builder) = self.routes.get_mut(&self.current_path) {
            builder()
        } else if let Some(builder) = self.routes.values_mut().next() {
            builder()
        } else {
            return;
        };

        let ctx = BuildContext {};
        let (_, child_id) = create_scope(|| child_widget.build(&mut state.arena, engine, &ctx));
        self.current_child_id = Some(child_id);

        // 3. YANGI SAHIFANI TAFFY'GA ULASH
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
                        format!("{}_router", path),
                    );
                    output.extend(child_output);
                }
                state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
            }
        }
        output
    }
}
