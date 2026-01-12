use super::base::{Widget, RenderOutput, BuildContext};
use rore_types::{Color, Style};
use rore_layout::{LayoutEngine, Node};
use rore_render::text::FontManager;
use glam::Vec2;
use std::sync::{Arc, Mutex};
use crate::state::FrameworkState;

pub struct Text {
    pub id: Option<String>,
    pub content: String,
    pub size: f32,
    pub color: Color,
    pub style: Style,
}

impl Text {
    pub fn new(content: &str) -> Self { 
        Self { 
            id: None, 
            content: content.to_string(), 
            size: 16.0, 
            color: Color::hex("#cdd6f4"), 
            style: Style::default() 
        } 
    }
    pub fn id(mut self, id: &str) -> Self { self.id = Some(id.to_string()); self }
    pub fn size(mut self, s: f32) -> Self { self.size = s; self }
    pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
}

impl Widget for Text {
    fn type_name(&self) -> &'static str { "Text" }
    
    fn build(&self, engine: &mut LayoutEngine, ctx: &BuildContext) -> Node {
        let content = self.content.clone();
        let size = self.size;
        let fm = ctx.font_manager.clone();
        
        // --- TUZATILDI: Tuple (f32, f32) qaytaradi ---
        engine.new_leaf_with_measure(self.style.clone(), move |width_constraint, _height_constraint| {
            // Taffy bizga f32 beradi (width constraint)
            // Agar cheksiz (INFINITY) bo'lsa, wrapping kerak emas (None).
            let max_width = if width_constraint.is_infinite() {
                None
            } else {
                Some(width_constraint)
            };

            // FontManager orqali o'lchaymiz
            let (w, h) = fm.lock().unwrap().measure(&content, size, max_width);
            
            // XATO SHU YERDA EDI: Size struct emas, Tuple qaytarish kerak
            (w, h) 
        })
    }
    
    fn render(
        &self, 
        engine: &LayoutEngine, 
        _state: &mut FrameworkState, 
        node: Node, 
        p: Vec2, 
        _fm: &Arc<Mutex<FontManager>>, 
        clip_rect: Option<[f32; 4]>,
        _path: String
    ) -> RenderOutput {
        let l = engine.get_final_layout(node, p.x, p.y);
        let mut out = RenderOutput::new();
        
        out.texts.push((
            self.content.clone(), 
            self.color, 
            self.size, 
            Vec2::new(l.x, l.y), 
            clip_rect,
            l.width // Layout hisoblagan kenglik
        ));
        out
    }
}