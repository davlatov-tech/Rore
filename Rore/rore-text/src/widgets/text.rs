use glam::Vec2;
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{
    BuildContext, DisplayCommand, IntoProp, Prop, RenderOutput, Widget,
};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::text::TextMeasurer;
use rore_types::{Color, Style};
use std::sync::{Arc, Mutex};

use crate::text::get_measurer;

pub struct Text {
    pub id: Option<String>,
    pub text: Prop<String>,
    pub color: Prop<Color>,
    pub font_size: Prop<f32>,
    pub style: Prop<Style>,
    pub live_text: Arc<Mutex<String>>,
    pub live_color: Option<Arc<Mutex<Color>>>,
}

impl Text {
    pub fn new(text: impl IntoProp<String>) -> Self {
        Self {
            id: None,
            text: text.into_prop(),
            color: Prop::Static(Color::WHITE),
            font_size: Prop::Static(16.0),
            style: Prop::Static(Style::default()),
            live_text: Arc::new(Mutex::new(String::new())),
            live_color: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    pub fn color(mut self, color: impl IntoProp<Color>) -> Self {
        self.color = color.into_prop();
        self
    }
    pub fn size(mut self, size: impl IntoProp<f32>) -> Self {
        self.font_size = size.into_prop();
        self
    }
    pub fn style(mut self, style: impl IntoProp<Style>) -> Self {
        self.style = style.into_prop();
        self
    }
}

impl Widget for Text {
    fn type_name(&self) -> &'static str {
        "Text"
    }

    fn is_interactive(&self) -> bool {
        self.id.is_some()
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        _ctx: &BuildContext,
    ) -> NodeId {
        let my_id = arena.allocate_node();

        // 1. MATN REAKTIVLIGI
        let text_prop = std::mem::replace(&mut self.text, Prop::Static(String::new()));
        match text_prop {
            Prop::Static(s) => *self.live_text.lock().unwrap() = s,
            Prop::Dynamic(mut f) => {
                let initial = f();
                *self.live_text.lock().unwrap() = initial.clone();
                let lt = self.live_text.clone();
                rore_core::reactive::signals::create_effect(move || {
                    let new_text = f();
                    *lt.lock().unwrap() = new_text.clone();
                    rore_core::reactive::command::CommandQueue::send(
                        rore_core::reactive::command::UICommand::UpdateText(my_id, new_text),
                    );
                });
            }
        }

        // 2. RANG REAKTIVLIGI
        let color_prop = std::mem::replace(&mut self.color, Prop::Static(Color::WHITE));
        match color_prop {
            Prop::Static(c) => arena.colors[my_id.0 as usize] = [c.r, c.g, c.b, c.a],
            Prop::Dynamic(mut f) => {
                let initial = f();
                arena.colors[my_id.0 as usize] = [initial.r, initial.g, initial.b, initial.a];
                let lc = Arc::new(Mutex::new(initial));
                self.live_color = Some(lc.clone());
                rore_core::reactive::signals::create_effect(move || {
                    let new_c = f();
                    *lc.lock().unwrap() = new_c;
                    rore_core::reactive::command::CommandQueue::send(
                        rore_core::reactive::command::UICommand::MarkDirty(
                            my_id,
                            rore_core::state::DIRTY_COLOR,
                        ),
                    );
                });
            }
        }

        let font_size = match &self.font_size {
            Prop::Static(v) => *v,
            _ => 16.0,
        };

        let live_text_for_layout = self.live_text.clone();
        let fm_arc = get_measurer();

        let taffy_node = engine.new_leaf_with_measure(
            match &self.style {
                Prop::Static(s) => s.clone(),
                _ => Style::default(),
            },
            move |known_w, _known_h| {
                let mut fm = fm_arc.lock().unwrap();
                let max_w = if known_w < f32::INFINITY {
                    Some(known_w)
                } else {
                    None
                };
                let current_text = live_text_for_layout.lock().unwrap().clone();
                fm.measure(&current_text, font_size, max_w)
            },
        );

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        if let Some(id_str) = &self.id {
            arena.register_id(id_str, my_id);
            engine.register_id(id_str, taffy_node);
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
        _path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);
        let my_id = *state.arena.node_map.get(&taffy_node).unwrap();

        let mut current_color = state.arena.colors[my_id.0 as usize];
        if let Some(live_c) = &self.live_color {
            let c = *live_c.lock().unwrap();
            current_color = [c.r, c.g, c.b, c.a];
        }

        let display_text = self.live_text.lock().unwrap().clone();
        let font_size = match &self.font_size {
            Prop::Static(v) => *v,
            _ => 16.0,
        };

        // INQILOB: WGPU qaramligi uzildi
        let cmd = DisplayCommand::DrawText {
            text: display_text,
            pos: Vec2::new(layout.x, layout.y),
            font_size,
            color: current_color,
            clip: clip_rect,
            width_limit: layout.width,
        };
        output.node_commands.push((my_id.0, vec![cmd]));

        output
    }
}
