use rore_core::widgets::{Widget, RenderOutput, BuildContext};
use rore_core::state::FrameworkState; 
use rore_core::widgets::base::TextureSource;
use rore_layout::{LayoutEngine, Node};
use rore_render::{Instance, text::FontManager};
use rore_types::*;
use glam::Vec2;
use std::sync::{Arc, Mutex};

pub struct Image {
    pub id: Option<String>,
    pub path: String,
    pub style: Style,
    pub border_radius: f32,
}

impl Image {
    pub fn load(path: &str) -> Self {
        Self {
            id: None,
            path: path.to_string(),
            style: Style::default(),
            border_radius: 0.0,
        }
    }

    pub fn id(mut self, id: &str) -> Self { self.id = Some(id.to_string()); self }
    
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.style.width = Val::Px(width);
        self.style.height = Val::Px(height);
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }
}

impl Widget for Image {
    fn type_name(&self) -> &'static str { "Image" }

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

        // Texture Load Request
        out.texture_loads.push((self.path.clone(), TextureSource::Path(self.path.clone())));

        // Image Instance
        let instance = Instance {
            position: pos,
            size: size,
            color_start: [1.0; 4], 
            color_end: [1.0; 4],
            gradient_angle: 0.0,
            border_radius: self.border_radius,
            border_width: 0.0,
            border_color: [0.0; 4],
            shadow_color: [0.0; 4],
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            clip_rect: effective_clip,
        };

        // Batch Rendering
        out.images.entry(self.path.clone()).or_insert_with(Vec::new).push(instance);

        out
    }
}