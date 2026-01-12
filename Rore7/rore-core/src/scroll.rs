use rore_types::{Style, Val, Display, FlexDirection, Align}; 
use rore_layout::{LayoutEngine, Node};
use rore_render::text::FontManager;
use crate::widgets::{Widget, RenderOutput, BuildContext};
use glam::Vec2;
use std::sync::{Arc, Mutex};
use crate::state::FrameworkState; 

pub struct ScrollView {
    pub id: String,
    pub content: Box<dyn Widget>,
    pub style: Style,
}

impl ScrollView {
    pub fn new(id: &str, content: impl Widget + 'static) -> Self {
        Self {
            id: id.to_string(),
            content: Box::new(content),
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: Align::Start, 
                align_items: Align::Start,
                align_content: Align::Start,
                ..Default::default()
            },
        }
    }

    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
}

impl Widget for ScrollView {
    fn type_name(&self) -> &'static str { "ScrollView" }

    fn build(&self, engine: &mut LayoutEngine, ctx: &BuildContext) -> Node {
        let content_node = self.content.build(engine, ctx);
        let mut forced_style = self.style.clone();
        forced_style.justify_content = Align::Start; 
        forced_style.align_items = Align::Start; 
        let node = engine.new_node(forced_style, &[content_node]);
        engine.register_id(&self.id, node);
        node
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        node: Node,
        parent_pos: Vec2,
        font_manager: &Arc<Mutex<FontManager>>,
        parent_clip: Option<[f32; 4]>,
        path: String, // <--- YANGI
    ) -> RenderOutput {
        let mut out = RenderOutput::new();
        
        let layout = engine.get_final_layout(node, parent_pos.x, parent_pos.y);
        let my_pos = Vec2::new(layout.x, layout.y);
        let my_clip = [layout.x, layout.y, layout.width, layout.height];
        let final_clip = if let Some(parent) = parent_clip {
            let x = my_clip[0].max(parent[0]);
            let y = my_clip[1].max(parent[1]);
            let r = (my_clip[0] + my_clip[2]).min(parent[0] + parent[2]);
            let b = (my_clip[1] + my_clip[3]).min(parent[1] + parent[3]);
            [x, y, (r - x).max(0.0), (b - y).max(0.0)]
        } else {
            my_clip
        };

        let raw_offset_y = *state.scroll_offsets.get(&self.id).unwrap_or(&0.0);
        
        let ids = engine.taffy.children(node).unwrap();
        if let Some(child_node) = ids.get(0) {
            let child_layout = engine.taffy.layout(*child_node).unwrap();
            let clamped_offset = raw_offset_y.clamp(0.0, (child_layout.size.height - layout.height).max(0.0));
            let child_pos = my_pos + Vec2::new(child_layout.location.x, child_layout.location.y) - Vec2::new(0.0, clamped_offset);

            // Zanjir davom ettiriladi
            let content_path = format!("{}/content", path);
            out.extend(self.content.render(engine, state, *child_node, child_pos, font_manager, Some(final_clip), content_path));
        }
        out
    }
}