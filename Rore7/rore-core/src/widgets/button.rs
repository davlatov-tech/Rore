use super::base::{Widget, RenderOutput, BuildContext};
use crate::state::FrameworkState; 
use super::view::View;
use rore_types::{Color, Style, Val, Display, Align, Thickness};
use rore_layout::{LayoutEngine, Node};
use rore_render::{Instance, text::FontManager};
use glam::Vec2;
use std::sync::{Arc, Mutex};

pub struct Button {
    pub id: Option<String>,
    view: View,
    pub bg_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub active_color: [f32; 4],
}

impl Button {
    pub fn new() -> Self {
        let style = Style {
            padding: Thickness { top: Val::Px(12.0), bottom: Val::Px(12.0), left: Val::Px(24.0), right: Val::Px(24.0) },
            display: Display::Flex, justify_content: Align::Center, align_items: Align::Center, 
            margin: Thickness { bottom: Val::Px(8.0), ..Default::default() },
            ..Default::default()
        };
        let bg = Color::hex("#313244");
        let hover = Color::hex("#45475a");
        let active = Color::hex("#585b70");

        let view = View::new(style).bg([bg.r, bg.g, bg.b, bg.a]).rounded(8.0);
        Self { 
            id: None, view, 
            bg_color: [bg.r, bg.g, bg.b, bg.a],
            hover_color: [hover.r, hover.g, hover.b, hover.a], 
            active_color: [active.r, active.g, active.b, active.a] 
        }
    }
    pub fn id(mut self, id: &str) -> Self { self.id = Some(id.to_string()); self }
    pub fn bg(mut self, c: [f32; 4]) -> Self { self.bg_color = c; self.view.background = c; self }
    pub fn child(mut self, w: impl Widget + 'static) -> Self { self.view = self.view.child(w); self }
}

impl Widget for Button {
    fn type_name(&self) -> &'static str { "Button" }
    
    fn build(&self, engine: &mut LayoutEngine, ctx: &BuildContext) -> Node { 
        let node = self.view.build(engine, ctx);
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
        let is_hovered = state.hovered_node == Some(node);
        let is_active = state.active_node == Some(node);
        
        let target_bg = if is_active { self.active_color } 
                        else if is_hovered { self.hover_color } 
                        else { self.bg_color };
        
        let offset = if is_active { Vec2::new(0.0, 1.0) } else { Vec2::ZERO };

        // Barqaror ID
        let anim_id = self.id.clone().unwrap_or(path.clone());
        let final_bg = state.get_animated_color(&anim_id, target_bg, 0.15);

        let mut out = RenderOutput::new();
        let l = engine.get_final_layout(node, p.x, p.y);
        let pos = Vec2::new(l.x, l.y) + offset;
        let effective_clip = clip_rect.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]);

        out.instances.push(Instance {
            position: pos, size: Vec2::new(l.width, l.height),
            color_start: final_bg, color_end: final_bg, gradient_angle: 0.0,
            border_radius: self.view.border_radius, border_width: self.view.border_width, border_color: self.view.border_color,
            shadow_color: self.view.shadow_color, shadow_offset: self.view.shadow_offset, shadow_blur: self.view.shadow_blur, shadow_spread: 0.0,
            clip_rect: effective_clip,
        });

        let ids = engine.taffy.children(node).unwrap();
        for (i, w) in self.view.children.iter().enumerate() {
            if let Some(id) = ids.get(i) { 
                let child_path = format!("{}/{}", path, i);
                out.extend(w.render(engine, state, *id, pos, fm, clip_rect, child_path)); 
            }
        }
        out
    }
}