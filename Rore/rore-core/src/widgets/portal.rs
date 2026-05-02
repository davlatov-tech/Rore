use glam::Vec2;
// TASHQI EMAS, ICHKI IMPORTLAR (crate::) o'z joyiga qaytarildi!
use crate::state::{FrameworkState, NodeId, UiArena};
use crate::widgets::base::{BuildContext, RenderOutput, Widget};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;

pub struct Portal {
    pub anchor_id: String, // Portal qaysi vidjet (Tugma) tagida ochilishini bilishi kerak
    pub child: Option<Box<dyn Widget>>,
    pub current_child_id: Option<NodeId>, // YANGI: O'z farzandini bilib turadi (Xotirani tozalash uchun)
}

impl Portal {
    pub fn new(anchor_id: &str) -> Self {
        Self {
            anchor_id: anchor_id.to_string(),
            child: None,
            current_child_id: None,
        }
    }

    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(w));
        self
    }
}

impl Widget for Portal {
    fn type_name(&self) -> &'static str {
        "Portal"
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        // 1. Portalning o'zi uchun ko'rinmas, bo'sh (0x0) qobiq yaratamiz.
        // U daraxtda joy egallamaydi.
        let style = Style::default();
        let taffy_node = engine.new_leaf(style);
        let my_id = arena.allocate_node();

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        // 2. Farzandni (masalan Dropdown) quramiz...
        let mut child_id_opt = None;
        if let Some(child) = self.child.take() {
            let child_id = child.build(arena, engine, ctx);
            child_id_opt = Some(child_id);

            if let Some(&child_taffy) = arena.taffy_map.get(&child_id) {
                if !arena.overlays.contains(&child_taffy) {
                    arena.overlays.push(child_taffy);
                }
                arena.anchors.insert(child_taffy, self.anchor_id.clone());

                arena
                    .logical_children
                    .entry(taffy_node)
                    .or_default()
                    .push(child_taffy);
            }
        }
        self.current_child_id = child_id_opt;

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn render(
        &self,
        _engine: &LayoutEngine,
        _state: &mut FrameworkState,
        _taffy_node: TaffyNode,
        _parent_pos: Vec2,
        _clip_rect: Option<[f32; 4]>,
        _path: String,
    ) -> RenderOutput {
        RenderOutput::new()
    }
}
