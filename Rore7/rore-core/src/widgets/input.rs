use super::base::{Widget, RenderOutput, BuildContext};
use rore_types::{Color, Style, Val, Display, Align, Thickness};
use rore_layout::{LayoutEngine, Node};
use rore_render::{Instance, text::FontManager};
use glam::Vec2;
use std::sync::{Arc, Mutex};
use crate::state::FrameworkState;

pub struct TextInput {
    pub id: Option<String>,
    pub value: String,
    pub placeholder: String,
    pub style: Style,
    pub bg_color: [f32; 4],
    pub text_color: Color,
}

impl TextInput {
    pub fn new(value: &str) -> Self {
        let bg = Color::hex("#313244");
        let text = Color::hex("#cdd6f4");
        Self {
            id: None,
            value: value.to_string(), placeholder: "Type here...".to_string(),
            style: Style { 
                width: Val::Px(200.0), 
                height: Val::Px(45.0), 
                display: Display::Flex, 
                align_items: Align::Center, 
                padding: Thickness { left: Val::Px(10.0), right: Val::Px(10.0), ..Default::default() }, 
                margin: Thickness { bottom: Val::Px(10.0), ..Default::default() }, 
                ..Default::default() 
            },
            bg_color: [bg.r, bg.g, bg.b, bg.a], 
            text_color: text,
        }
    }
    pub fn id(mut self, id: &str) -> Self { self.id = Some(id.to_string()); self }
    pub fn placeholder(mut self, text: &str) -> Self { self.placeholder = text.to_string(); self }
}

impl Widget for TextInput {
    fn type_name(&self) -> &'static str { "TextInput" }
    
    fn build(&self, engine: &mut LayoutEngine, _ctx: &BuildContext) -> Node { 
        let node = engine.new_leaf(self.style.clone());
        if let Some(id) = &self.id { engine.register_id(id, node); }
        node
    }
    
    fn render(
        &self, 
        engine: &LayoutEngine, 
        state: &mut FrameworkState, 
        node: Node, 
        parent_pos: Vec2, 
        fm: &Arc<Mutex<FontManager>>, 
        clip_rect: Option<[f32; 4]>,
        _path: String
    ) -> RenderOutput {
        let is_focused_node = state.focused_node == Some(node);
        let is_input_active = self.id.as_ref() == state.focused_input_id.as_ref() && self.id.is_some();
        let is_active = is_focused_node || is_input_active;

        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(node, parent_pos.x, parent_pos.y);
        let my_pos = Vec2::new(layout.x, layout.y);
        
        // Background
        let bw = if is_active { 2.0 } else { 1.0 };
        let bc = if is_active { [0.53, 0.70, 0.98, 1.0] } else { [0.35, 0.35, 0.45, 1.0] };
        let effective_clip = clip_rect.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]);

        output.instances.push(Instance {
            position: my_pos, size: Vec2::new(layout.width, layout.height),
            color_start: self.bg_color, color_end: self.bg_color, gradient_angle: 0.0,
            border_radius: 8.0, border_width: bw, border_color: bc,
            shadow_color: [0.0; 4], shadow_offset: Vec2::ZERO, shadow_blur: 0.0, shadow_spread: 0.0,
            clip_rect: effective_clip,
        });

        // Text & Layout
        let show_ph = self.value.is_empty();
        let txt = if show_ph { &self.placeholder } else { &self.value };
        let col = if show_ph { Color::hex("#a6adc8") } else { self.text_color };
        let fs = 18.0;
        
        let padding_x = 10.0; 
        let available_width = layout.width - (padding_x * 2.0);

        // --- YANGI: ENGINE CALL ---
        let text_layout = fm.lock().unwrap().layout_text(txt, fs, None);
        
        let text_width = text_layout.total_size.0;
        let mut scroll_x = 0.0;
        if !show_ph && text_width > available_width {
            scroll_x = available_width - text_width - 2.0;
        }
        let t_pos = my_pos + Vec2::new(padding_x + scroll_x, 12.0); // 12.0 = vertikal padding

        let input_inner_clip = [layout.x + 2.0, layout.y + 2.0, layout.width - 4.0, layout.height - 4.0];
        let final_text_clip = if let Some(parent) = clip_rect {
             let x = input_inner_clip[0].max(parent[0]);
             let y = input_inner_clip[1].max(parent[1]);
             let r = (input_inner_clip[0] + input_inner_clip[2]).min(parent[0] + parent[2]);
             let b = (input_inner_clip[1] + input_inner_clip[3]).min(parent[1] + parent[3]);
             Some([x, y, (r - x).max(0.0), (b - y).max(0.0)])
        } else {
             Some(input_inner_clip)
        };

        // HIT TESTING & DRAG
        if state.active_node == Some(node) {
            let local_mouse_x = state.cursor_pos.x - t_pos.x;
            let local_mouse_y = state.cursor_pos.y - t_pos.y;
            let idx = text_layout.hit_test(local_mouse_x, local_mouse_y);
            
            if state.drag_start_idx.is_none() {
                state.drag_start_idx = Some(idx);
                state.input_selection = None; 
            }
            
            if let Some(start) = state.drag_start_idx {
                if start != idx {
                    state.input_selection = Some((start, idx));
                } else {
                    state.input_selection = None;
                }
            }

            state.input_cursor_idx = idx;
            state.focused_input_id = self.id.clone();
        }

        // --- SELECTION RENDER (Fix #3) ---
        if let Some((start, end)) = state.get_normalized_selection() {
            let rects = text_layout.get_selection_rects(start, end);
            
            for (rx, ry, rw, rh) in rects {
                let sel_pos = t_pos + Vec2::new(rx, ry);
                output.instances.push(Instance {
                    position: sel_pos, size: Vec2::new(rw, rh),
                    color_start: [0.2, 0.4, 0.8, 0.4], color_end: [0.2, 0.4, 0.8, 0.4], gradient_angle: 0.0,
                    border_radius: 0.0, border_width: 0.0, border_color: [0.0;4],
                    shadow_color: [0.0;4], shadow_offset: Vec2::ZERO, shadow_blur: 0.0, shadow_spread: 0.0,
                    clip_rect: final_text_clip.unwrap_or(effective_clip),
                });
            }
        }

        // TEXT RENDER
        output.texts.push((txt.clone(), col, fs, t_pos, final_text_clip, f32::INFINITY));

        // --- CURSOR RENDER (Fix #2) ---
        if is_active && !show_ph {
            let safe_idx = state.input_cursor_idx.min(self.value.len());
            if let Some((cx, cy, ch)) = text_layout.get_cursor_pos(safe_idx) {
                // Kursor balandligi va markazlashuvi
                let cursor_h = 20.0; 
                let offset_y = (ch - cursor_h) / 2.0; 
                let cursor_draw_pos = t_pos + Vec2::new(cx, cy + offset_y);
                
                output.instances.push(Instance {
                    position: cursor_draw_pos, size: Vec2::new(2.0, cursor_h),
                    color_start: [0.8, 0.8, 0.9, 1.0], color_end: [0.8, 0.8, 0.9, 1.0], gradient_angle: 0.0,
                    border_radius: 0.0, border_width: 0.0, border_color: [0.0; 4], 
                    shadow_color: [0.0; 4], shadow_offset: Vec2::ZERO, shadow_blur: 0.0, shadow_spread: 0.0,
                    clip_rect: final_text_clip.unwrap_or(effective_clip),
                });
            }
        } else if is_active && show_ph {
             let cursor_h = 20.0;
             let line_h = text_layout.line_height;
             let offset_y = (line_h - cursor_h) / 2.0;

             output.instances.push(Instance {
                position: t_pos + Vec2::new(0.0, offset_y), 
                size: Vec2::new(2.0, cursor_h),
                color_start: [0.8, 0.8, 0.9, 1.0], color_end: [0.8, 0.8, 0.9, 1.0], gradient_angle: 0.0,
                border_radius: 0.0, border_width: 0.0, border_color: [0.0; 4], 
                shadow_color: [0.0; 4], shadow_offset: Vec2::ZERO, shadow_blur: 0.0, shadow_spread: 0.0,
                clip_rect: final_text_clip.unwrap_or(effective_clip),
            });
        }

        output
    }
}