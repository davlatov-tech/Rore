use crate::mapper::map_style;
use rore_types::Style;
use taffy::node::Node;
use taffy::tree::LayoutTree;
use taffy::Taffy;

pub struct LayoutEngine {
    pub taffy: Taffy,
    pub root: Option<Node>,
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
        }
    }

    pub fn clear(&mut self) {
        self.taffy = Taffy::new();
        self.root = None;
    }

    pub fn mark_interactive(&mut self, _node: Node) {}
    pub fn register_id(&mut self, _id: &str, _node: Node) {}
    pub fn add_logical_parent(&mut self, _node: Node, _parent: Node) {}
    pub fn add_logical_parent_id(&mut self, _node: Node, _target_id: &str) {}
    pub fn get_node(&self, _id: &str) -> Option<Node> {
        None
    }

    pub fn reset_node_y_to_zero(&mut self, node: Node) {
        let layout = self.taffy.layout_mut(node);
        layout.location.y = 0.0;
    }

    pub fn disable_shrink(&mut self, node: Node) {
        if let Ok(mut style) = self.taffy.style(node).cloned() {
            style.flex_shrink = 0.0;
            let _ = self.taffy.set_style(node, style);
        }
    }
    pub fn update_style(&mut self, node: Node, style: Style) {
        // Taffy tushunadigan tilga o'giramiz
        let taffy_style = map_style(&style);
        let _ = self.taffy.set_style(node, taffy_style);
    }

    pub fn new_node(&mut self, style: Style, children: &[Node]) -> Node {
        let taffy_style = map_style(&style);
        self.taffy.new_with_children(taffy_style, children).unwrap()
    }

    pub fn new_leaf(&mut self, style: Style) -> Node {
        let taffy_style = map_style(&style);
        self.taffy.new_leaf(taffy_style).unwrap()
    }

    pub fn new_leaf_with_measure(
        &mut self,
        style: Style,
        measure_func: impl Fn(f32, f32) -> (f32, f32) + 'static + Send + Sync,
    ) -> Node {
        let taffy_style = map_style(&style);
        self.taffy
            .new_leaf_with_measure(
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
                    taffy::geometry::Size {
                        width: w,
                        height: h,
                    }
                })),
            )
            .unwrap()
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

    pub fn get_final_layout(&self, node: Node, parent_x: f32, parent_y: f32) -> ComputedLayout {
        let layout_res = self.taffy.layout(node);
        let layout = match layout_res {
            Ok(l) => l,
            Err(_) => {
                return ComputedLayout {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                }
            }
        };

        ComputedLayout {
            x: parent_x + layout.location.x,
            y: parent_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        }
    }
}
