use glam::Vec2;
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{
    BuildContext, EventResult, IntoProp, Prop, RenderOutput, Widget, WidgetEvent,
};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_render::Instance;
use rore_types::{Color, Style};
use std::cell::Cell;
use winit::keyboard::{Key, NamedKey};

use crate::text::get_measurer;
use rore_types::{impl_layout_modifiers, LayoutModifiers};

pub struct TextInput {
    pub id: String,
    pub on_input: Option<Box<dyn FnMut(String) + Send + 'static>>,
    pub style: Prop<Style>,
    pub bg_color: Color,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub font_size: f32,
    pub placeholder: String,
    pub border_radius: f32,

    pub multiline: bool,

    // Endi faqat lines bilan emas, global byte mantiqi orqali ishlaymiz!
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_char: usize,
    pub cursor_byte: usize,

    pub is_dirty: Cell<bool>,
    pub cached_cursor: Cell<(f32, f32, f32, bool)>,
    pub last_width: Cell<f32>,
    pub last_input_time: Cell<f32>,

    pub scroll_x: Cell<f32>,
    pub scroll_y: Cell<f32>,
    pub was_focused: Cell<bool>,
    pub focus_anim_start: Cell<f32>,

    // INQILOB: Selection Anchor va Koordinatalar
    pub selection_anchor: Cell<Option<usize>>,
    pub last_layout_x: Cell<f32>,
    pub last_layout_y: Cell<f32>,

    node_id: Option<NodeId>,
}

impl_layout_modifiers!(TextInput);

impl TextInput {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            on_input: None,
            style: Prop::Static(Style::default()),
            bg_color: Color::hex("#0f172a"),
            text_color: Color::WHITE,
            placeholder_color: Color::hex("#64748b"),
            font_size: 16.0,
            placeholder: "".to_string(),
            border_radius: 8.0,

            multiline: false,

            lines: vec![String::new()],
            cursor_row: 0,
            cursor_char: 0,
            cursor_byte: 0,

            is_dirty: Cell::new(true),
            cached_cursor: Cell::new((0.0, 0.0, 0.0, false)),
            last_width: Cell::new(0.0),
            last_input_time: Cell::new(0.0),

            scroll_x: Cell::new(0.0),
            scroll_y: Cell::new(0.0),
            was_focused: Cell::new(false),
            focus_anim_start: Cell::new(0.0),

            selection_anchor: Cell::new(None),
            last_layout_x: Cell::new(0.0),
            last_layout_y: Cell::new(0.0),

            node_id: None,
        }
    }

    pub fn on_input<F: FnMut(String) + Send + 'static>(mut self, f: F) -> Self {
        self.on_input = Some(Box::new(f));
        self
    }
    pub fn multiline(mut self, is_multiline: bool) -> Self {
        self.multiline = is_multiline;
        self
    }
    pub fn style(mut self, style: impl IntoProp<Style>) -> Self {
        self.style = style.into_prop();
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

    // Yordamchi logikalar (Global matn manipulyatsiyasi - juda xavfsiz va xatosiz!)
    pub fn get_full_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_full_text(&mut self, text: &str) {
        self.lines = text.split('\n').map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }

    pub fn get_global_byte(&self) -> usize {
        let mut b = 0;
        for r in 0..self.cursor_row {
            b += self.lines[r].len() + 1; // +1 \n uchun
        }
        b += self.cursor_byte;
        b
    }

    pub fn set_cursor_global_byte(&mut self, mut global_byte: usize) {
        let mut r = 0;
        while r < self.lines.len() {
            let len = self.lines[r].len();
            if global_byte <= len {
                self.cursor_row = r;
                self.cursor_byte = global_byte;
                self.cursor_char = self.lines[r][..global_byte].chars().count();
                return;
            }
            global_byte -= len + 1;
            r += 1;
        }
        self.cursor_row = self.lines.len() - 1;
        self.cursor_byte = self.lines[self.cursor_row].len();
        self.cursor_char = self.lines[self.cursor_row].chars().count();
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some(anchor) = self.selection_anchor.get() {
            let cursor = self.get_global_byte();
            if anchor != cursor {
                let start = anchor.min(cursor);
                let end = anchor.max(cursor);
                let mut full_text = self.get_full_text();
                if start < full_text.len() && end <= full_text.len() {
                    full_text.replace_range(start..end, "");
                    self.set_full_text(&full_text);
                    self.set_cursor_global_byte(start);
                    self.selection_anchor.set(None);
                    return true;
                }
            }
            self.selection_anchor.set(None);
        }
        false
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
        let base_style = match &self.style {
            Prop::Static(s) => s.clone(),
            _ => Style::default(),
        };

        let taffy_node = engine.new_leaf(base_style);
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
            let is_shift = state.modifiers.shift_key();
            let is_ctrl = state.modifiers.control_key() || state.modifiers.super_key();

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
                    let lx =
                        state.cursor_pos.x - self.last_layout_x.get() - 12.0 + self.scroll_x.get();
                    let ly =
                        state.cursor_pos.y - self.last_layout_y.get() - 12.0 + self.scroll_y.get();

                    let measurer_arc = get_measurer();
                    let mut fm = measurer_arc.lock().unwrap();
                    let gb = fm.get_byte_at_pos(
                        &self.get_full_text(),
                        self.font_size,
                        if self.multiline {
                            Some(self.last_width.get())
                        } else {
                            None
                        },
                        lx,
                        ly,
                    );

                    if is_shift {
                        if self.selection_anchor.get().is_none() {
                            self.selection_anchor.set(Some(self.get_global_byte()));
                        }
                    } else {
                        self.selection_anchor.set(Some(gb)); // Drag boshlash uchun
                    }

                    self.set_cursor_global_byte(gb);
                    changed = true;
                    self.last_input_time.set(state.global_time);
                }
                WidgetEvent::MouseDrag { .. } => {
                    let lx =
                        state.cursor_pos.x - self.last_layout_x.get() - 12.0 + self.scroll_x.get();
                    let ly =
                        state.cursor_pos.y - self.last_layout_y.get() - 12.0 + self.scroll_y.get();

                    let measurer_arc = get_measurer();
                    let mut fm = measurer_arc.lock().unwrap();
                    let gb = fm.get_byte_at_pos(
                        &self.get_full_text(),
                        self.font_size,
                        if self.multiline {
                            Some(self.last_width.get())
                        } else {
                            None
                        },
                        lx,
                        ly,
                    );

                    self.set_cursor_global_byte(gb);
                    changed = true;
                }
                WidgetEvent::MouseUp => {
                    let anchor = self.selection_anchor.get();
                    let gb = self.get_global_byte();
                    if anchor == Some(gb) {
                        self.selection_anchor.set(None); // Shunchaki bosilgan bo'lsa langarni olib tashlaymiz
                    }
                }
                WidgetEvent::TextInput(input_str) => {
                    if !is_ctrl {
                        self.delete_selection();
                        let mut full = self.get_full_text();
                        let gb = self.get_global_byte();
                        full.insert_str(gb, input_str);
                        self.set_full_text(&full);
                        self.set_cursor_global_byte(gb + input_str.len());
                        changed = true;
                        self.last_input_time.set(state.global_time);
                    }
                }
                WidgetEvent::KeyPress(key) => {
                    self.last_input_time.set(state.global_time);

                    // OS Clipboard yorliqlari (Hotkeys)
                    if is_ctrl {
                        if let Key::Character(c) = key {
                            if c.eq_ignore_ascii_case("a") {
                                self.selection_anchor.set(Some(0));
                                self.set_cursor_global_byte(self.get_full_text().len());
                                changed = true;
                            } else if c.eq_ignore_ascii_case("c") || c.eq_ignore_ascii_case("x") {
                                if let Some(anchor) = self.selection_anchor.get() {
                                    let cursor = self.get_global_byte();
                                    let start = anchor.min(cursor);
                                    let end = anchor.max(cursor);
                                    if start < end {
                                        let full = self.get_full_text();
                                        if let Some(cb_mutex) = &state.clipboard {
                                            if let Ok(mut cb) = cb_mutex.lock() {
                                                let _ = cb.set_text(full[start..end].to_string());
                                            }
                                        }
                                        if c.eq_ignore_ascii_case("x") {
                                            self.delete_selection();
                                            changed = true;
                                        }
                                    }
                                }
                            } else if c.eq_ignore_ascii_case("v") {
                                if let Some(cb_mutex) = &state.clipboard {
                                    if let Ok(mut cb) = cb_mutex.lock() {
                                        if let Ok(pasted) = cb.get_text() {
                                            let pasted = if self.multiline {
                                                pasted
                                            } else {
                                                pasted.replace('\n', " ")
                                            };
                                            self.delete_selection();
                                            let mut full = self.get_full_text();
                                            let gb = self.get_global_byte();
                                            full.insert_str(gb, &pasted);
                                            self.set_full_text(&full);
                                            self.set_cursor_global_byte(gb + pasted.len());
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    match key {
                        Key::Named(NamedKey::Backspace) => {
                            if self.delete_selection() {
                                changed = true;
                            } else {
                                let gb = self.get_global_byte();
                                if gb > 0 {
                                    let mut full = self.get_full_text();
                                    let mut prev = gb - 1;
                                    while prev > 0 && !full.is_char_boundary(prev) {
                                        prev -= 1;
                                    }
                                    full.replace_range(prev..gb, "");
                                    self.set_full_text(&full);
                                    self.set_cursor_global_byte(prev);
                                    changed = true;
                                }
                            }
                        }
                        Key::Named(NamedKey::Enter) => {
                            if self.multiline {
                                self.delete_selection();
                                let mut full = self.get_full_text();
                                let gb = self.get_global_byte();
                                full.insert_str(gb, "\n");
                                self.set_full_text(&full);
                                self.set_cursor_global_byte(gb + 1);
                                changed = true;
                            }
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            let gb = self.get_global_byte();
                            if is_shift {
                                if self.selection_anchor.get().is_none() {
                                    self.selection_anchor.set(Some(gb));
                                }
                            } else {
                                self.selection_anchor.set(None);
                            }

                            if gb > 0 {
                                let full = self.get_full_text();
                                let mut prev = gb - 1;
                                while prev > 0 && !full.is_char_boundary(prev) {
                                    prev -= 1;
                                }
                                self.set_cursor_global_byte(prev);
                                changed = true;
                            }
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            let gb = self.get_global_byte();
                            let full = self.get_full_text();
                            if is_shift {
                                if self.selection_anchor.get().is_none() {
                                    self.selection_anchor.set(Some(gb));
                                }
                            } else {
                                self.selection_anchor.set(None);
                            }

                            if gb < full.len() {
                                let mut next = gb + 1;
                                while next < full.len() && !full.is_char_boundary(next) {
                                    next += 1;
                                }
                                self.set_cursor_global_byte(next);
                                changed = true;
                            }
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            let gb = self.get_global_byte();
                            if is_shift {
                                if self.selection_anchor.get().is_none() {
                                    self.selection_anchor.set(Some(gb));
                                }
                            } else {
                                self.selection_anchor.set(None);
                            }

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
                            let gb = self.get_global_byte();
                            if is_shift {
                                if self.selection_anchor.get().is_none() {
                                    self.selection_anchor.set(Some(gb));
                                }
                            } else {
                                self.selection_anchor.set(None);
                            }

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

                let current_text = self.get_full_text();
                if let Some(cb) = &mut self.on_input {
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

        self.last_layout_x.set(layout.x);
        self.last_layout_y.set(layout.y);

        let is_focused = state.focused_node == Some(taffy_node);
        let was_focused = self.was_focused.get();

        if is_focused != was_focused {
            self.focus_anim_start.set(state.global_time);
            self.was_focused.set(is_focused);
        }

        let current_bg = state.arena.colors[my_id.0 as usize];
        let border_focus = [0.2, 0.6, 1.0, 1.0];
        let border_normal = [0.3, 0.4, 0.5, 1.0];

        let (c_start, c_target) = if is_focused {
            (border_normal, border_focus)
        } else {
            (border_focus, border_normal)
        };

        let my_clip = [layout.x, layout.y, layout.width, layout.height];
        let combined_clip = if let Some(parent_clip) = clip_rect {
            let x1 = parent_clip[0].max(my_clip[0]);
            let y1 = parent_clip[1].max(my_clip[1]);
            let x2 = (parent_clip[0] + parent_clip[2]).min(my_clip[0] + my_clip[2]);
            let y2 = (parent_clip[1] + parent_clip[3]).min(my_clip[1] + my_clip[3]);
            Some([x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0)])
        } else {
            Some(my_clip)
        };

        let box_inst = Instance {
            position: Vec2::new(layout.x, layout.y),
            size: Vec2::new(layout.width, layout.height),
            color_start: current_bg,
            color_end: current_bg,
            target_color_start: current_bg,
            target_color_end: current_bg,
            gradient_angle: 0.0,
            border_radius: [self.border_radius; 4],
            border_width: [1.0; 4],
            border_color: c_start,
            target_border_color: c_target,
            shadow_color: [0.0; 4],
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            clip_rect: clip_rect.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]),
            anim_start_time: self.focus_anim_start.get(),
            anim_duration: 0.15,
        };
        output.sparse_instances.push((my_id.0, box_inst));

        let pad_x = 12.0;
        let pad_y = 12.0;
        let inner_width = layout.width - pad_x * 2.0;
        let inner_height = layout.height - pad_y * 2.0;
        let line_height = self.font_size * 1.2;

        let display_text = if self.lines.len() == 1 && self.lines[0].is_empty() {
            self.placeholder.clone()
        } else {
            self.lines.join("\n")
        };

        if is_focused {
            if self.is_dirty.get() || (self.last_width.get() - inner_width).abs() > 0.5 {
                let measurer_arc = get_measurer();
                let mut fm = measurer_arc.lock().unwrap();
                let visible_cursor_byte = self.get_global_byte();

                let (cx, local_cy, ch) = fm.get_cursor_pos(
                    &display_text,
                    self.font_size,
                    if self.multiline {
                        Some(inner_width)
                    } else {
                        None
                    },
                    visible_cursor_byte,
                );

                let global_cy = local_cy;
                let mut sx = self.scroll_x.get();
                let mut sy = self.scroll_y.get();

                if cx < sx {
                    sx = cx;
                } else if cx > sx + inner_width - 2.0 {
                    sx = cx - inner_width + 2.0;
                }
                if self.multiline {
                    if global_cy < sy {
                        sy = global_cy;
                    } else if global_cy + line_height > sy + inner_height {
                        sy = global_cy + line_height - inner_height;
                    }
                } else {
                    sy = 0.0;
                }

                self.scroll_x.set(sx);
                self.scroll_y.set(sy);
                self.cached_cursor.set((cx, global_cy, ch, true));
                self.last_width.set(inner_width);
                self.is_dirty.set(false);
            }
        }

        let sx = self.scroll_x.get();
        let sy = self.scroll_y.get();
        let final_text_color = if self.lines.len() == 1 && self.lines[0].is_empty() {
            self.placeholder_color
        } else {
            self.text_color
        };

        // 1. CHIZISH: SELECTION BOX (MATNNI ORQASIDAN)
        if is_focused && !(self.lines.len() == 1 && self.lines[0].is_empty()) {
            if let Some(anchor) = self.selection_anchor.get() {
                let gb = self.get_global_byte();
                if anchor != gb {
                    let start = anchor.min(gb);
                    let end = anchor.max(gb);
                    let measurer_arc = get_measurer();
                    let mut fm = measurer_arc.lock().unwrap();
                    let rects = fm.get_selection_rects(
                        &display_text,
                        self.font_size,
                        if self.multiline {
                            Some(inner_width)
                        } else {
                            None
                        },
                        start,
                        end,
                    );

                    for (i, r) in rects.iter().enumerate() {
                        let sel_inst = Instance {
                            position: Vec2::new(
                                layout.x + pad_x - sx + r[0],
                                layout.y + pad_y - sy + r[1],
                            ),
                            size: Vec2::new(r[2], r[3]),
                            color_start: [0.2, 0.4, 0.8, 0.5], // Shaffof Ko'k
                            color_end: [0.2, 0.4, 0.8, 0.5],
                            target_color_start: [0.2, 0.4, 0.8, 0.5],
                            target_color_end: [0.2, 0.4, 0.8, 0.5],
                            gradient_angle: 0.0,
                            border_radius: [2.0; 4],
                            border_width: [0.0; 4],
                            border_color: [0.0; 4],
                            target_border_color: [0.0; 4],
                            shadow_color: [0.0; 4],
                            shadow_offset: Vec2::ZERO,
                            shadow_blur: 0.0,
                            shadow_spread: 0.0,
                            clip_rect: combined_clip
                                .unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]),
                            anim_start_time: 0.0,
                            anim_duration: 0.0,
                        };
                        output
                            .sparse_instances
                            .push((my_id.0 + 30000 + i as u32, sel_inst));
                    }
                }
            }
        }

        // 2. CHIZISH: MATNNING O'ZI
        let text_pos = Vec2::new(layout.x + pad_x - sx, layout.y + pad_y - sy);
        output.sparse_texts.push((
            my_id.0,
            display_text,
            final_text_color,
            self.font_size,
            text_pos,
            combined_clip,
            if self.multiline { inner_width } else { 0.0 },
        ));

        // 3. CHIZISH: KURSOR (CARET)
        if is_focused {
            let (cx, cy, ch, is_in_view) = self.cached_cursor.get();
            let time_since_input = state.global_time - self.last_input_time.get();
            let blink_on = time_since_input < 0.1 || ((time_since_input / 0.50) as u32 % 2) == 0;

            if is_in_view {
                let caret_x = layout.x + pad_x + cx - sx;
                let caret_y = layout.y + pad_y + cy - sy;

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
                    border_radius: [1.0; 4],
                    border_width: [0.0; 4],
                    border_color: [0.0; 4],
                    target_border_color: [0.0; 4],
                    shadow_color: [0.0; 4],
                    shadow_offset: Vec2::ZERO,
                    shadow_blur: 0.0,
                    shadow_spread: 0.0,
                    clip_rect: combined_clip.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]),
                    anim_start_time: 0.0,
                    anim_duration: 0.0,
                };
                output.sparse_instances.push((my_id.0 + 10000, caret_inst));
            }
        }

        output
    }

    fn visual_overflow(&self) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
