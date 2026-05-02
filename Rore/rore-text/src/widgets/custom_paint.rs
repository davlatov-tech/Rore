use glam::Vec2;
use rore_core::reactive::command::{CommandQueue, UICommand};
use rore_core::reactive::signals::{create_effect, Signal};
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{
    BuildContext, DisplayCommand, EventResult, RenderOutput, Widget, WidgetEvent,
};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;
use std::sync::{Arc, Mutex};

pub struct CustomPaint {
    pub id: Option<String>,
    pub style: Style,
    // Dasturchi yozadigan matematik chizish funksiyasi
    pub painter: Arc<Mutex<Box<dyn FnMut(Vec2, Vec2) -> Vec<DisplayCommand> + Send>>>,
    // Reaktivlikni ta'minlash uchun kuzatiladigan signallar ro'yxati
    pub triggers: Vec<Box<dyn FnOnce(NodeId) + Send>>,
    node_id: Option<NodeId>,
}

impl CustomPaint {
    pub fn new<F>(painter: F) -> Self
    where
        F: FnMut(Vec2, Vec2) -> Vec<DisplayCommand> + Send + 'static,
    {
        Self {
            id: None,
            style: Style::default(),
            painter: Arc::new(Mutex::new(Box::new(painter))),
            triggers: Vec::new(),
            node_id: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn watch<T: Clone + Send + 'static>(mut self, sig: Signal<T>) -> Self {
        self.triggers.push(Box::new(move |my_id| {
            create_effect(move || {
                let _val = sig.get(); // Signalni o'qiymiz, Effect zanjirini ulash uchun

                // Signal o'zgarganda faqat Paint (Chizish) yangilanadi, Layout emas!
                CommandQueue::send(UICommand::MarkDirty(my_id, rore_core::state::DIRTY_COLOR));
            });
        }));
        self
    }
}

impl Widget for CustomPaint {
    fn type_name(&self) -> &'static str {
        "CustomPaint"
    }

    fn is_interactive(&self) -> bool {
        // ID berilgan bo'lsa, sichqoncha hodisalarini ushlashga ruxsat beramiz
        self.id.is_some()
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        _ctx: &BuildContext,
    ) -> NodeId {
        // Taffy uchun bu shunchaki bo'sh quti (Leaf Node)
        let taffy_node = engine.new_leaf(self.style.clone());
        let my_id = arena.allocate_node();
        self.node_id = Some(my_id);

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        if let Some(id_str) = &self.id {
            arena.register_id(id_str, my_id);
            engine.register_id(id_str, taffy_node);
            engine.mark_interactive(taffy_node);
        }

        // Barcha bog'langan signallar uchun Reaktiv Effectlarni ishga tushiramiz
        for trigger_setup in self.triggers.drain(..) {
            trigger_setup(my_id);
        }

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn handle_event(&mut self, state: &mut FrameworkState, event: &WidgetEvent) -> EventResult {
        if self.id.is_some() {
            match event {
                WidgetEvent::HoverEnter => {
                    state.current_cursor_icon = winit::window::CursorIcon::Crosshair;
                    return EventResult::Consumed;
                }
                WidgetEvent::HoverLeave => {
                    state.current_cursor_icon = winit::window::CursorIcon::Default;
                    return EventResult::Consumed;
                }
                _ => {}
            }
        }
        EventResult::Ignored
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        _state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        _clip_rect: Option<[f32; 4]>,
        _path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();

        // Taffy qutining ramkasini (Joylashuv va O'lcham) hisoblab beradi
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);
        let my_id = self.node_id.unwrap();

        let pos = Vec2::new(layout.x, layout.y);
        let size = Vec2::new(layout.width, layout.height);

        if size.x > 0.0 && size.y > 0.0 {
            let mut painter_fn = self.painter.lock().unwrap();
            let commands = painter_fn(pos, size);

            if !commands.is_empty() {
                // Minglab komandalar yig'indisini Yadroga WGPU ga jo'natish uchun uzatamiz
                output.node_commands.push((my_id.0, commands));
            }
        }

        output
    }
}
