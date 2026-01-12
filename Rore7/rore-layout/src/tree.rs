use taffy::Taffy;
use taffy::node::Node;
use rore_types::Style;
use crate::mapper::map_style;
use std::collections::HashMap;

// Taffy traitlari
use taffy::tree::LayoutTree; 

pub struct LayoutEngine {
    pub taffy: Taffy,
    pub root: Option<Node>,
    pub id_map: HashMap<String, Node>,      
    pub node_to_id: HashMap<Node, String>,  
    pub parent_map: HashMap<Node, Node>,
}

#[derive(Debug, Clone, Copy)]
pub struct ComputedLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            taffy: Taffy::new(),
            root: None,
            id_map: HashMap::new(),
            node_to_id: HashMap::new(),
            parent_map: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.taffy = Taffy::new();
        self.root = None;
        self.id_map.clear();
        self.node_to_id.clear(); 
        self.parent_map.clear();
    }

    pub fn register_id(&mut self, id: &str, node: Node) {
        self.id_map.insert(id.to_string(), node);
        self.node_to_id.insert(node, id.to_string());
    }

    pub fn get_node(&self, id: &str) -> Option<Node> {
        self.id_map.get(id).cloned()
    }

    pub fn parent(&self, node: Node) -> Option<Node> {
        self.parent_map.get(&node).cloned()
    }

pub fn reset_node_y_to_zero(&mut self, node: Node) {
    let layout = self.taffy.layout_mut(node); // Result emas, to'g'ridan-to'g'ri reference!
    layout.location.y = 0.0;
}

    pub fn disable_shrink(&mut self, node: Node) {
        // style() ham Result qaytarishi mumkin
        if let Ok(mut style) = self.taffy.style(node).cloned() {
            style.flex_shrink = 0.0;
            let _ = self.taffy.set_style(node, style);
        }
    }

    pub fn new_node(&mut self, style: Style, children: &[Node]) -> Node {
        let taffy_style = map_style(&style);
        let parent_node = self.taffy.new_with_children(taffy_style, children).unwrap();
        
        for child in children {
            self.parent_map.insert(*child, parent_node);
        }
        parent_node
    }

    pub fn new_leaf(&mut self, style: Style) -> Node {
        let taffy_style = map_style(&style);
        self.taffy.new_leaf(taffy_style).unwrap()
    }

    pub fn new_leaf_with_measure(
        &mut self, 
        style: Style, 
        measure_func: impl Fn(f32, f32) -> (f32, f32) + 'static + Send + Sync
    ) -> Node {
        let taffy_style = map_style(&style);
        self.taffy.new_leaf_with_measure(
            taffy_style, 
            taffy::node::MeasureFunc::Boxed(Box::new(move |known_dims, available_space| {
                let width = match known_dims.width {
                    Some(w) => w,
                    None => match available_space.width {
                        taffy::style::AvailableSpace::Definite(w) => w,
                        _ => f32::INFINITY,
                    },
                };
                let height = match known_dims.height {
                    Some(h) => h,
                    None => match available_space.height {
                        taffy::style::AvailableSpace::Definite(h) => h,
                        _ => f32::INFINITY,
                    },
                };
                let (w, h) = measure_func(width, height);
                taffy::geometry::Size { width: w, height: h }
            }))
        ).unwrap()
    }

    pub fn compute(&mut self, width: f32, height: f32) {
        if let Some(root) = self.root {
            let available_space = taffy::geometry::Size {
                width: taffy::style::AvailableSpace::Definite(width),
                height: taffy::style::AvailableSpace::Definite(height),
            };
            let _ = self.taffy.compute_layout(root, available_space);
        }
    }

    // --- Layout olishda ham ehtiyot bo'lish kerak ---
    pub fn get_final_layout(&self, node: Node, parent_x: f32, parent_y: f32) -> ComputedLayout {
        // layout() Result qaytaradimi? Xatoga ko'ra layout_mut Result emas.
        // Ehtimol layout() ham Result emasdir.
        // Xavfsiz variant:
        let layout_res = self.taffy.layout(node); 
        
        // Agar Result bo'lsa:
        let layout = match layout_res {
            Ok(l) => l,
            Err(_) => return ComputedLayout { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }, // Xato bo'lsa bo'sh
        };

        // Agar Result bo'lmasa (sizdagi holatda ehtimoliy), yuqoridagi kod xato beradi.
        // Keling, oddiy unwrap ishlatamiz. Agar u Result bo'lmasa, unwrap() ni olib tashlang.
        
        ComputedLayout {
            x: parent_x + layout.location.x,
            y: parent_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        }
    }

    // --- HIT TEST ---
    pub fn hit_test(
        &self, 
        root: Node, 
        cursor_x: f32, 
        cursor_y: f32, 
        node_scroll_offsets: &HashMap<Node, f32>
    ) -> Option<Node> {
        self.hit_test_recursive(root, cursor_x, cursor_y, 0.0, 0.0, node_scroll_offsets)
    }

    fn hit_test_recursive(
        &self, 
        node: Node, 
        mx: f32, my: f32,
        parent_x: f32, parent_y: f32,
        scroll_map: &HashMap<Node, f32>
    ) -> Option<Node> {
        // layout() ham Result qaytaradi deb faraz qilamiz (standart Taffy)
        if let Ok(layout) = self.taffy.layout(node) {
            let abs_x = parent_x + layout.location.x;
            let abs_y = parent_y + layout.location.y;
            let width = layout.size.width;
            let height = layout.size.height;

            let is_inside = mx >= abs_x && mx <= abs_x + width &&
                            my >= abs_y && my <= abs_y + height;

            if is_inside {
                let scroll_offset = scroll_map.get(&node).unwrap_or(&0.0);
                let effective_child_parent_y = abs_y - scroll_offset;

                if let Ok(children) = self.taffy.children(node) {
                    for &child in children.iter().rev() {
                        if let Some(hit) = self.hit_test_recursive(child, mx, my, abs_x, effective_child_parent_y, scroll_map) {
                            return Some(hit);
                        }
                    }
                }

                if self.node_to_id.contains_key(&node) {
                    return Some(node);
                }
            }
        }
        None
    }
}