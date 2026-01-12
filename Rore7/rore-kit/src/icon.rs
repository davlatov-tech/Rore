use rore_core::widgets::{Widget, RenderOutput, BuildContext};
use rore_core::widgets::base::TextureSource; 
use rore_core::state::FrameworkState;
use rore_layout::{LayoutEngine, Node};
use rore_render::text::FontManager;
use rore_render::Instance;
use rore_types::*;
use glam::Vec2;
use std::sync::{Arc, Mutex};
use std::fs;

use resvg::usvg::{Options, Tree};
use tiny_skia::{Pixmap, Transform};

pub struct Icon {
    pub id: Option<String>,
    pub path: String,
    pub style: Style,
    pub color: Option<Color>, 
}

impl Icon {
    pub fn load(path: &str) -> Self {
        Self {
            id: None,
            path: path.to_string(),
            style: Style {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                ..Default::default()
            },
            color: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self { self.id = Some(id.to_string()); self }

    pub fn size(mut self, size: f32) -> Self {
        self.style.width = Val::Px(size);
        self.style.height = Val::Px(size);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl Widget for Icon {
    fn type_name(&self) -> &'static str { "Icon" }

    fn build(&self, engine: &mut LayoutEngine, _ctx: &BuildContext) -> Node {
        let node = engine.new_leaf(self.style.clone());
        if let Some(id) = &self.id { engine.register_id(id, node); }
        node
    }

    fn render(
        &self, 
        engine: &LayoutEngine, 
        _state: &mut FrameworkState,
        node: Node, 
        parent_pos: Vec2, 
        _fm: &Arc<Mutex<FontManager>>, 
        clip_rect: Option<[f32; 4]>,
        _path: String // <--- YANGI ARGUMENT (Trait talabi)
    ) -> RenderOutput {
        let mut out = RenderOutput::new();
        let layout = engine.get_final_layout(node, parent_pos.x, parent_pos.y);
        let pos = Vec2::new(layout.x, layout.y);
        let size = Vec2::new(layout.width, layout.height);
        let effective_clip = clip_rect.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]);

        let color_hex = if let Some(c) = self.color {
            format!("{:x}{:x}{:x}", (c.r*255.0) as u8, (c.g*255.0) as u8, (c.b*255.0) as u8)
        } else {
            "original".to_string()
        };
        
        let texture_id = format!("{}::{}", self.path, color_hex);

        // SVG Rasterization
        if let Ok(svg_data) = fs::read_to_string(&self.path) {
            let opt = Options::default();
            if let Ok(tree) = Tree::from_str(&svg_data, &opt) {
                let scale = 2.0; 
                let render_w = (layout.width * scale) as u32;
                let render_h = (layout.height * scale) as u32;

                if render_w > 0 && render_h > 0 {
                    let mut pixmap = Pixmap::new(render_w, render_h).unwrap();
                    let svg_size = tree.size();
                    
                    let sx = render_w as f32 / svg_size.width();
                    let sy = render_h as f32 / svg_size.height();
                    let transform = Transform::from_scale(sx, sy);

                    resvg::render(&tree, transform, &mut pixmap.as_mut());
                    
                    if let Ok(png_bytes) = pixmap.encode_png() {
                        out.texture_loads.push((texture_id.clone(), TextureSource::Bytes(png_bytes)));
                    }
                }
            }
        }

        let tint = if let Some(c) = self.color {
            [c.r, c.g, c.b, c.a]
        } else {
            [1.0; 4] 
        };

        let instance = Instance {
            position: pos,
            size: size,
            color_start: tint, 
            color_end: tint,
            gradient_angle: 0.0,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: [0.0; 4],
            shadow_color: [0.0; 4],
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            clip_rect: effective_clip,
        };

        out.images.entry(texture_id).or_insert_with(Vec::new).push(instance);

        out
    }
}