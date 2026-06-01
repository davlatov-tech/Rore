use crate::mapper::map_style;
use rore_types::Style;
use std::collections::HashMap;
use taffy::NodeId as Node;
use taffy::TaffyTree;

type MeasureFuncType = Box<dyn Fn(f32, f32) -> (f32, f32) + 'static + Send + Sync>;

pub struct LayoutEngine {
    pub taffy: TaffyTree,
    pub root: Option<Node>,
    pub measure_funcs: HashMap<Node, MeasureFuncType>,
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
            taffy: TaffyTree::new(),
            root: None,
            measure_funcs: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.taffy = TaffyTree::new();
        self.root = None;
        self.measure_funcs.clear();
    }

    pub fn mark_interactive(&mut self, _node: Node) {}
    pub fn register_id(&mut self, _id: &str, _node: Node) {}
    pub fn add_logical_parent(&mut self, _node: Node, _parent: Node) {}
    pub fn add_logical_parent_id(&mut self, _node: Node, _target_id: &str) {}
    pub fn get_node(&self, _id: &str) -> Option<Node> {
        None
    }

    pub fn reset_node_y_to_zero(&mut self, _node: Node) {}

    pub fn disable_shrink(&mut self, node: Node) {
        if let Ok(mut style) = self.taffy.style(node).cloned() {
            style.flex_shrink = 0.0;
            let _ = self.taffy.set_style(node, style);
        }
    }

    pub fn update_style(&mut self, node: Node, style: Style) {
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
        let node = self.taffy.new_leaf(taffy_style).unwrap();
        self.measure_funcs.insert(node, Box::new(measure_func));
        node
    }

    pub fn compute(&mut self, width: f32, height: f32) {
        if let Some(root) = self.root {
            let available_space = taffy::geometry::Size {
                width: taffy::style::AvailableSpace::Definite(width),
                height: taffy::style::AvailableSpace::Definite(height),
            };

            let funcs = &self.measure_funcs;
            let _ = self.taffy.compute_layout_with_measure(
                root,
                available_space,
                |known_dims, avail_space, node_id, _ctx, _style| {
                    if let Some(f) = funcs.get(&node_id) {
                        let w = match known_dims.width {
                            Some(w) => w,
                            None => match avail_space.width {
                                taffy::style::AvailableSpace::Definite(w) => w,
                                taffy::style::AvailableSpace::MinContent => 0.0,
                                taffy::style::AvailableSpace::MaxContent => f32::INFINITY,
                            },
                        };
                        let h = match known_dims.height {
                            Some(h) => h,
                            None => match avail_space.height {
                                taffy::style::AvailableSpace::Definite(h) => h,
                                taffy::style::AvailableSpace::MinContent => 0.0,
                                taffy::style::AvailableSpace::MaxContent => f32::INFINITY,
                            },
                        };

                        let (mw, mh) = f(w, h);

                        taffy::geometry::Size {
                            width: (mw + 0.5).ceil(),
                            height: (mh + 0.5).ceil(),
                        }
                    } else {
                        taffy::geometry::Size::ZERO
                    }
                },
            );
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
            x: (parent_x + layout.location.x).round(),
            y: (parent_y + layout.location.y).round(),
            width: layout.size.width.round(),
            height: layout.size.height.round(),
        }
    }

    // INQILOB: Taffy 0.10 dan "Haqiqiy Skroll O'lchamini" so'rab olish
    pub fn get_scroll_size(&self, node: Node) -> (f32, f32) {
        if let Ok(layout) = self.taffy.layout(node) {
            (layout.content_size.width, layout.content_size.height)
        } else {
            (0.0, 0.0)
        }
    }
}
