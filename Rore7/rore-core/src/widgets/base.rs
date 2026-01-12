use rore_types::Color;
use rore_layout::{LayoutEngine, Node};
use rore_render::{Instance, text::FontManager};
use glam::Vec2;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::state::FrameworkState; 

pub struct BuildContext {
    pub font_manager: Arc<Mutex<FontManager>>,
}

#[derive(Debug, Clone)]
pub enum TextureSource {
    Path(String),
    Bytes(Vec<u8>),
}

pub struct RenderOutput {
    pub instances: Vec<Instance>,
    // YANGI: width (f32) qo'shildi
    pub texts: Vec<(String, Color, f32, Vec2, Option<[f32; 4]>, f32)>, 
    pub images: HashMap<String, Vec<Instance>>,
    pub texture_loads: Vec<(String, TextureSource)>,
}

impl RenderOutput {
    pub fn new() -> Self { 
        Self { 
            instances: Vec::new(), 
            texts: Vec::new(),
            images: HashMap::new(),
            texture_loads: Vec::new(),
        } 
    }
    
    pub fn extend(&mut self, other: RenderOutput) {
        self.instances.extend(other.instances);
        self.texts.extend(other.texts);
        for (id, list) in other.images {
            self.images.entry(id).or_insert_with(Vec::new).extend(list);
        }
        self.texture_loads.extend(other.texture_loads);
    }
}

pub trait Widget {
    fn build(&self, engine: &mut LayoutEngine, ctx: &BuildContext) -> Node;
    
    fn render(
        &self, 
        engine: &LayoutEngine, 
        state: &mut FrameworkState, 
        node: Node, 
        parent_pos: Vec2, 
        font_manager: &Arc<Mutex<FontManager>>, 
        clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput;
    
    fn type_name(&self) -> &'static str;
}