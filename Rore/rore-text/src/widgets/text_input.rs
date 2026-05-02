use glam::Vec2;
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, EventResult, RenderOutput, Widget, WidgetEvent};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_render::Instance;
use rore_types::{Color, Style};
use std::cell::Cell;
use winit::keyboard::{Key, NamedKey};

use crate::text::get_measurer;

pub struct TextInput {
    pub id: String,
    pub on_input: Option<Box<dyn FnMut(String) + Send + 'static>>,
    pub style: Style,
    pub bg_color: Color,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub font_size: f32,
    pub placeholder: String,
    pub border_radius: f32,

    pub lines: Vec<String>,

    pub cursor_row: usize,
    pub cursor_char: usize,
    pub cursor_byte: usize,

    pub is_dirty: Cell<bool>,
    pub cached_cursor: Cell<(f32, f32, f32, bool)>,
    pub last_width: Cell<f32>,
    pub last_input_time: Cell<f32>,

    node_id: Option<NodeId>,
}

impl TextInput {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            on_input: None,
            style: Style::default(),
            bg_color: Color::hex("#0f172a"),
            text_color: Color::WHITE,
            placeholder_color: Color::hex("#64748b"),
            font_size: 16.0,
            placeholder: "".to_string(),
            border_radius: 8.0,

            lines: vec![String::new()],
            cursor_row: 0,
            cursor_char: 0,
            cursor_byte: 0,

            is_dirty: Cell::new(true),
            cached_cursor: Cell::new((0.0, 0.0, 0.0, false)),
            last_width: Cell::new(0.0),
            last_input_time: Cell::new(0.0),

            node_id: None,
        }
    }

    pub fn on_input<F: FnMut(String) + Send + 'static>(mut self, f: F) -> Self {
        self.on_input = Some(Box::new(f));
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    pub fn bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    pub fn placeholder(mut self, text: &str) -> Self {
        self.placeholder = text.to_string();
        self
    }
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.border_radius = r;
        self
    }

    pub fn get_full_text(&self) -> String {
        self.lines.join("\n")
    }
}

impl Widget for TextInput {
    fn type_name(&self) -> &'static str {
        "TextInput"
    }
    fn is_interactive(&self) -> bool {
        true
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        _ctx: &BuildContext,
    ) -> NodeId {
        let taffy_node = engine.new_leaf(self.style.clone());
        let my_id = arena.allocate_node();
        self.node_id = Some(my_id);
        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);
        arena.colors[my_id.0 as usize] = [
            self.bg_color.r,
            self.bg_color.g,
            self.bg_color.b,
            self.bg_color.a,
        ];
        arena.register_id(&self.id, my_id);
        engine.register_id(&self.id, taffy_node);
        engine.mark_interactive(taffy_node);
        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn handle_event(&mut self, state: &mut FrameworkState, event: &WidgetEvent) -> EventResult {
        if let Some(id) = self.node_id {
            let mut changed = false;

            match event {
                WidgetEvent::HoverEnter => {
                    state.current_cursor_icon = cursor_icon::CursorIcon::Text;
                    return EventResult::Consumed;
                }
                WidgetEvent::HoverLeave => {
                    state.current_cursor_icon = cursor_icon::CursorIcon::Default;
                    return EventResult::Consumed;
                }
                WidgetEvent::MouseDown => {
                    let last_row = self.lines.len() - 1;
                    if self.cursor_row != last_row || self.cursor_byte != self.lines[last_row].len()
                    {
                        self.cursor_row = last_row;
                        self.cursor_char = self.lines[last_row].chars().count();
                        self.cursor_byte = self.lines[last_row].len();
                        changed = true;
                    }
                    self.last_input_time.set(state.global_time);
                }
                WidgetEvent::TextInput(input_str) => {
                    self.lines[self.cursor_row].insert_str(self.cursor_byte, input_str);
                    self.cursor_char += input_str.chars().count();
                    self.cursor_byte += input_str.len();
                    changed = true;
                    self.last_input_time.set(state.global_time);
                }
                WidgetEvent::KeyPress(key) => {
                    self.last_input_time.set(state.global_time);
                    match key {
                        Key::Named(NamedKey::Backspace) => {
                            if self.cursor_byte > 0 {
                                let last_char = self.lines[self.cursor_row][..self.cursor_byte]
                                    .chars()
                                    .next_back()
                                    .unwrap();
                                let ch_len = last_char.len_utf8();
                                self.cursor_byte -= ch_len;
                                self.cursor_char -= 1;
                                self.lines[self.cursor_row]
                                    .drain(self.cursor_byte..self.cursor_byte + ch_len);
                                changed = true;
                            } else if self.cursor_row > 0 {
                                let current_line = self.lines.remove(self.cursor_row);
                                self.cursor_row -= 1;
                                self.cursor_char = self.lines[self.cursor_row].chars().count();
                                self.cursor_byte = self.lines[self.cursor_row].len();
                                self.lines[self.cursor_row].push_str(&current_line);
                                changed = true;
                            }
                        }
                        Key::Named(NamedKey::Enter) => {
                            let remainder = self.lines[self.cursor_row].split_off(self.cursor_byte);
                            self.cursor_row += 1;
                            self.lines.insert(self.cursor_row, remainder);
                            self.cursor_char = 0;
                            self.cursor_byte = 0;
                            changed = true;
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            if self.cursor_byte > 0 {
                                let last_char = self.lines[self.cursor_row][..self.cursor_byte]
                                    .chars()
                                    .next_back()
                                    .unwrap();
                                self.cursor_byte -= last_char.len_utf8();
                                self.cursor_char -= 1;
                                changed = true;
                            } else if self.cursor_row > 0 {
                                self.cursor_row -= 1;
                                self.cursor_char = self.lines[self.cursor_row].chars().count();
                                self.cursor_byte = self.lines[self.cursor_row].len();
                                changed = true;
                            }
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            if self.cursor_byte < self.lines[self.cursor_row].len() {
                                let next_char = self.lines[self.cursor_row][self.cursor_byte..]
                                    .chars()
                                    .next()
                                    .unwrap();
                                self.cursor_byte += next_char.len_utf8();
                                self.cursor_char += 1;
                                changed = true;
                            } else if self.cursor_row < self.lines.len() - 1 {
                                self.cursor_row += 1;
                                self.cursor_char = 0;
                                self.cursor_byte = 0;
                                changed = true;
                            }
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            if self.cursor_row > 0 {
                                self.cursor_row -= 1;
                                let char_count = self.lines[self.cursor_row].chars().count();
                                self.cursor_char = self.cursor_char.min(char_count);
                                self.cursor_byte = self.lines[self.cursor_row]
                                    .char_indices()
                                    .nth(self.cursor_char)
                                    .map(|(i, _)| i)
                                    .unwrap_or(self.lines[self.cursor_row].len());
                                changed = true;
                            }
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            if self.cursor_row < self.lines.len() - 1 {
                                self.cursor_row += 1;
                                let char_count = self.lines[self.cursor_row].chars().count();
                                self.cursor_char = self.cursor_char.min(char_count);
                                self.cursor_byte = self.lines[self.cursor_row]
                                    .char_indices()
                                    .nth(self.cursor_char)
                                    .map(|(i, _)| i)
                                    .unwrap_or(self.lines[self.cursor_row].len());
                                changed = true;
                            }
                        }
                        _ => {}
                    }
                }
                _ => return EventResult::Ignored,
            }

            if changed {
                self.is_dirty.set(true);

                // MUKAMMAL INTEGRATSIYA: Borrow checker xatosini chetlab o'tish
                let current_text = self.get_full_text(); // Avval o'qiymiz
                if let Some(cb) = &mut self.on_input {
                    // Keyin o'zgartirishga ochamiz
                    cb(current_text);
                }

                if !state.sparse_update_queue.contains(&id) {
                    state.sparse_update_queue.push(id);
                }
                return EventResult::Consumed;
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
        clip_rect: Option<[f32; 4]>,
        _path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);
        let my_id = self.node_id.unwrap();

        let is_focused = state.focused_node == Some(taffy_node);
        let current_bg = state.arena.colors[my_id.0 as usize];
        let border_c = if is_focused {
            [0.2, 0.6, 1.0, 1.0]
        } else {
            [0.3, 0.4, 0.5, 1.0]
        };

        let box_inst = Instance {
            position: Vec2::new(layout.x, layout.y),
            size: Vec2::new(layout.width, layout.height),
            color_start: current_bg,
            color_end: current_bg,
            target_color_start: current_bg,
            target_color_end: current_bg,
            gradient_angle: 0.0,
            border_radius: self.border_radius,
            border_width: 1.0,
            border_color: border_c,
            target_border_color: border_c,
            shadow_color: [0.0; 4],
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
            anim_start_time: 0.0,
            anim_duration: 0.0,
        };
        output.sparse_instances.push((my_id.0, box_inst));

        let safe_padding_bottom = self.font_size * 0.4;
        let my_bounds = [
            layout.x,
            layout.y,
            layout.width,
            layout.height + safe_padding_bottom,
        ];

        let strict_clip = if let Some(clip) = clip_rect {
            let min_x = clip[0].max(my_bounds[0]);
            let min_y = clip[1].max(my_bounds[1]);
            let max_x = (clip[0] + clip[2]).min(my_bounds[0] + my_bounds[2]);
            let max_y = (clip[1] + clip[3]).min(my_bounds[1] + my_bounds[3]);
            Some([
                min_x,
                min_y,
                (max_x - min_x).max(0.0),
                (max_y - min_y).max(0.0),
            ])
        } else {
            Some(my_bounds)
        };

        let is_empty = self.lines.len() == 1 && self.lines[0].is_empty();
        let line_height = self.font_size * 1.2;
        let pad_x = 12.0;
        let pad_y = 12.0;
        let inner_width = layout.width - pad_x * 2.0;

        let mut start_row = 0;
        let mut end_row = self.lines.len();

        if let Some(clip) = strict_clip {
            let widget_y = layout.y + pad_y;
            let visible_top = clip[1] - widget_y;
            let visible_bottom = (clip[1] + clip[3]) - widget_y;

            let s = (visible_top / line_height).floor() as isize;
            let e = (visible_bottom / line_height).ceil() as isize;

            start_row = s.max(0) as usize;
            end_row = (e + 2).max(0) as usize;
            end_row = end_row.min(self.lines.len());
            if start_row > end_row {
                start_row = end_row;
            }
        }

        let display_text = if is_empty {
            self.placeholder.clone()
        } else {
            self.lines[start_row..end_row].join("\n")
        };

        let final_text_color = if is_empty {
            self.placeholder_color
        } else {
            self.text_color
        };
        let text_pos = Vec2::new(
            layout.x + pad_x,
            layout.y + pad_y + (start_row as f32 * line_height),
        );

        output.sparse_texts.push((
            my_id.0,
            display_text.clone(),
            final_text_color,
            self.font_size,
            text_pos,
            strict_clip,
            inner_width,
        ));

        if is_focused {
            if self.is_dirty.get() || (self.last_width.get() - inner_width).abs() > 0.5 {
                let measurer_arc = get_measurer();
                let mut fm = measurer_arc.lock().unwrap();

                let mut visible_cursor_byte = 0;
                let mut cursor_is_visible = true;

                if self.cursor_row >= start_row && self.cursor_row < end_row && !is_empty {
                    for r in start_row..self.cursor_row {
                        visible_cursor_byte += self.lines[r].len() + 1;
                    }
                    visible_cursor_byte += self.cursor_byte;
                } else if is_empty {
                    visible_cursor_byte = 0;
                } else {
                    cursor_is_visible = false;
                }

                let (cx, local_cy, ch) = fm.get_cursor_pos(
                    &display_text,
                    self.font_size,
                    Some(inner_width),
                    visible_cursor_byte,
                );

                let global_cy = local_cy + (start_row as f32 * line_height);

                self.cached_cursor
                    .set((cx, global_cy, ch, cursor_is_visible));
                self.last_width.set(inner_width);
                self.is_dirty.set(false);
            }

            let (cx, cy, ch, is_in_view) = self.cached_cursor.get();
            let time_since_input = state.global_time - self.last_input_time.get();

            let blink_on = time_since_input < 0.1 || ((time_since_input / 0.50) as u32 % 2) == 0;

            if is_in_view {
                let caret_x = layout.x + pad_x + cx;
                let caret_y = layout.y + pad_y + cy;

                let caret_color = if blink_on {
                    [self.text_color.r, self.text_color.g, self.text_color.b, 1.0]
                } else {
                    [0.0, 0.0, 0.0, 0.0]
                };

                let caret_inst = Instance {
                    position: Vec2::new(caret_x, caret_y),
                    size: Vec2::new(2.0, ch),
                    color_start: caret_color,
                    color_end: caret_color,
                    target_color_start: caret_color,
                    target_color_end: caret_color,
                    gradient_angle: 0.0,
                    border_radius: 1.0,
                    border_width: 0.0,
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

                output.sparse_instances.push((my_id.0 + 10000, caret_inst));
            }
        }

        output
    }

    fn visual_overflow(&self) -> [f32; 4] {
        [1.0, 1.0, 1.0 + (self.font_size * 0.4), 1.0]
    }
}
