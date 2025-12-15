use crate::convert;
use rore_primitives::layout::Style as RoreStyle; // Aniq import
use taffy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutNode(pub NodeId);

pub struct LayoutEngine {
    taffy: TaffyTree<()>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self { taffy: TaffyTree::new() }
    }

    pub fn new_node(&mut self, style: &RoreStyle, children: &[LayoutNode]) -> LayoutNode {
        let taffy_style = convert::to_taffy_style(style);
        let child_ids: Vec<NodeId> = children.iter().map(|n| n.0).collect();
        LayoutNode(self.taffy.new_with_children(taffy_style, &child_ids).unwrap())
    }

    pub fn compute(&mut self, root: LayoutNode, width: f32, height: f32) {
        let space = Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        };
        self.taffy.compute_layout(root.0, space).unwrap();
    }

    pub fn get_layout(&self, node: LayoutNode) -> rore_primitives::geometry::Rect {
        let l = self.taffy.layout(node.0).unwrap();
        rore_primitives::geometry::Rect::new(l.location.x, l.location.y, l.size.width, l.size.height)
    }
}