use super::base::{Widget, RenderOutput, BuildContext};
use rore_types::Style;
use rore_layout::{LayoutEngine, Node};
use rore_render::{Instance, text::FontManager};
use glam::Vec2;
use std::sync::{Arc, Mutex};
use crate::state::FrameworkState; 

pub struct View {
    pub id: Option<String>,
    pub style: Style,
    pub children: Vec<Box<dyn Widget>>,
    pub background: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
    pub shadow_color: [f32; 4],
    pub shadow_offset: Vec2,
    pub shadow_blur: f32,
}

impl View {
    pub fn new(style: Style) -> Self { 
        Self { 
            id: None, style, children: vec![], 
            background: [0.0;4], border_radius: 0.0, 
            border_width: 0.0, border_color: [0.0;4], 
            shadow_color: [0.0;4], shadow_offset: Vec2::ZERO, shadow_blur: 0.0 
        } 
    }
    pub fn id(mut self, id: &str) -> Self { self.id = Some(id.to_string()); self }
    pub fn child(mut self, w: impl Widget + 'static) -> Self { self.children.push(Box::new(w)); self }
    pub fn bg(mut self, c: [f32; 4]) -> Self { self.background = c; self }
    pub fn rounded(mut self, r: f32) -> Self { self.border_radius = r; self }
    pub fn border(mut self, w: f32, c: [f32; 4]) -> Self { self.border_width = w; self.border_color = c; self }
    pub fn shadow(mut self, c: [f32; 4], o: Vec2, b: f32) -> Self { self.shadow_color = c; self.shadow_offset = o; self.shadow_blur = b; self }
}

impl Widget for View {
    fn type_name(&self) -> &'static str { "View" }
    
    fn build(&self, engine: &mut LayoutEngine, ctx: &BuildContext) -> Node {
        let children: Vec<Node> = self.children.iter().map(|c| c.build(engine, ctx)).collect();
        let node = engine.new_node(self.style.clone(), &children);
        if let Some(id) = &self.id { engine.register_id(id, node); }
        node
    }
    
    fn render(
        &self, 
        engine: &LayoutEngine, 
        state: &mut FrameworkState, 
        node: Node, 
        p: Vec2, 
        fm: &Arc<Mutex<FontManager>>, 
        clip_rect: Option<[f32; 4]>,
        path: String, // <--- YANGI
    ) -> RenderOutput {
        let mut out = RenderOutput::new();
        let l = engine.get_final_layout(node, p.x, p.y);
        let pos = Vec2::new(l.x, l.y);
        let effective_clip = clip_rect.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]);

        if self.background[3] > 0.0 || self.border_width > 0.0 {
            // Animatsiya: ID yoki Path orqali
            let anim_id = self.id.clone().unwrap_or(path.clone());
            let final_bg = state.get_animated_color(&anim_id, self.background, 0.2);

            out.instances.push(Instance {
                position: pos, size: Vec2::new(l.width, l.height),
                color_start: final_bg, color_end: final_bg, gradient_angle: 0.0,
                border_radius: self.border_radius, border_width: self.border_width, border_color: self.border_color,
                shadow_color: self.shadow_color, shadow_offset: self.shadow_offset, shadow_blur: self.shadow_blur, shadow_spread: 0.0,
                clip_rect: effective_clip,
            });
        }
        
        let ids = engine.taffy.children(node).unwrap();
        for (i, w) in self.children.iter().enumerate() {
            if let Some(id) = ids.get(i) { 
                // Zanjir yaratish
                let child_path = format!("{}/{}", path, i);
                out.extend(w.render(engine, state, *id, pos, fm, clip_rect, child_path)); 
            }
        }
        out
    }
}