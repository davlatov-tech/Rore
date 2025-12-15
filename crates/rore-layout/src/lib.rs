pub mod convert;

use taffy::prelude::*;
use taffy::style::Style;

// NodeId ni eksport qilamiz
pub type LayoutNode = NodeId;

// Asosiy turlarni eksport qilamiz
pub use taffy::geometry::{Size, Point, Rect};
pub use taffy::style::{AvailableSpace, Dimension, LengthPercentage};
pub use taffy::Layout; 

pub struct LayoutEngine {
    // 6-MODUL HOLATI: Hech qanday murakkab Context yoki Callback yo'q.
    // Oddiy Taffy daraxti.
    taffy: TaffyTree<()>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
        }
    }

    pub fn new_node(&mut self, style: &Style, children: &[LayoutNode]) -> LayoutNode {
        self.taffy.new_with_children(style.clone(), children).unwrap()
    }

    // Measure funksiyasini VAQTINCHA OLIB TASHLAYMIZ.
    // Biz avval oddiy qutilar ishlashini test qilamiz.

    pub fn compute(&mut self, root: LayoutNode, width: f32, height: f32) {
        let available_space = Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        };
        self.taffy.compute_layout(root, available_space).unwrap();
    }

    pub fn get_layout(&self, node: LayoutNode) -> Layout {
        *self.taffy.layout(node).unwrap()
    }
}

// --- UNIT TEST (ISBOT) ---
// Kodni yozdikmi, darrov tekshiramiz.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_layout_logic() {
        let mut engine = LayoutEngine::new();

        // 1. Ota konteyner (500x500)
        let root_style = Style {
            size: Size { width: Dimension::Length(500.0), height: Dimension::Length(500.0) },
            display: taffy::style::Display::Flex,
            ..Default::default()
        };

        // 2. Bola (50% x 50%)
        let child_style = Style {
            size: Size { width: Dimension::Percent(0.5), height: Dimension::Percent(0.5) },
            ..Default::default()
        };

        let child = engine.new_node(&child_style, &[]);
        let root = engine.new_node(&root_style, &[child]);

        // 3. Hisoblash
        engine.compute(root, 1000.0, 1000.0);

        // 4. Tekshirish
        let root_layout = engine.get_layout(root);
        let child_layout = engine.get_layout(child);

        // Ota 500px bo'lishi kerak
        assert_eq!(root_layout.size.width, 500.0);
        // Bola otasining 50% i (250px) bo'lishi kerak
        assert_eq!(child_layout.size.width, 250.0);
        
        println!("Layout Test: Muvaffaqiyatli o'tdi!");
    }
}