use crate::state::{FrameworkState, NodeId, UiArena};
use glam::Vec2;
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_render::Instance;
use rore_types::{Color, Style};
use std::collections::{HashMap, HashSet};
use winit::keyboard::Key;

// =====================================================================
// INQILOB: CPU va Hodisalar uchun "AQL" (Phase 2)
// =====================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyState {
    Clean,
    PaintOnly, // Taffy uxlaydi, faqat GPU buffer yangilanadi
    Layout,    // Taffy o'lchamlarni hisoblaydi
}

const CELL_SIZE: f32 = 128.0;

#[derive(Clone, Debug)]
pub struct GridItem {
    pub node_id: NodeId,
    pub z_index: i32,
    pub rect: [f32; 4],
}

pub struct SpatialHashGrid {
    cells: HashMap<(i32, i32), Vec<GridItem>>,
}

impl SpatialHashGrid {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, node_id: NodeId, rect: [f32; 4], z_index: i32) {
        let min_x = (rect[0] / CELL_SIZE).floor() as i32;
        let min_y = (rect[1] / CELL_SIZE).floor() as i32;
        let max_x = ((rect[0] + rect[2]) / CELL_SIZE).floor() as i32;
        let max_y = ((rect[1] + rect[3]) / CELL_SIZE).floor() as i32;

        let item = GridItem {
            node_id,
            z_index,
            rect,
        };
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                self.cells
                    .entry((x, y))
                    .or_insert_with(Vec::new)
                    .push(item.clone());
            }
        }
    }

    pub fn query_point(&self, x: f32, y: f32) -> Vec<GridItem> {
        let cell_x = (x / CELL_SIZE).floor() as i32;
        let cell_y = (y / CELL_SIZE).floor() as i32;
        if let Some(items) = self.cells.get(&(cell_x, cell_y)) {
            let mut found = Vec::new();
            let mut seen = HashSet::new();
            for item in items {
                if x >= item.rect[0]
                    && x <= item.rect[0] + item.rect[2]
                    && y >= item.rect[1]
                    && y <= item.rect[1] + item.rect[3]
                {
                    if seen.insert(item.node_id) {
                        found.push(item.clone());
                    }
                }
            }
            // Z-index bo'yicha yuqoridan pastga saralash
            found.sort_by(|a, b| b.z_index.cmp(&a.z_index));
            return found;
        }
        Vec::new()
    }
}

pub struct BuildContext {}

#[derive(Debug, Clone)]
pub enum TextureSource {
    Path(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum WidgetEvent {
    HoverEnter,
    HoverLeave,
    MouseDown,
    MouseUp,
    Click,
    TextInput(String),
    KeyPress(Key),
    // =========================================================================
    // YANGI HODISALAR: Kursor va G'ildirakcha (Scroll) uchun Universal Interfeys
    // =========================================================================
    MouseMove { x: f32, y: f32 },
    MouseDrag { dx: f32, dy: f32 },
    MouseScroll { delta_x: f32, delta_y: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    Consumed,
    Ignored,
}

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    PushClip {
        rect: [f32; 4],
    },
    PopClip,
    PushTransform {
        offset: Vec2,
    },
    PopTransform,
    DrawQuad {
        rect: [f32; 4],
        color: [f32; 4],
        border_radius: f32,
        border_width: f32,
        border_color: [f32; 4],
        anim_start_time: f32,
        anim_duration: f32,
    },
    DrawText {
        text: String,
        pos: Vec2,
        font_size: f32,
        color: [f32; 4],
        clip: Option<[f32; 4]>,
        width_limit: f32,
    },
    DrawCustomShader {
        shader_id: String,
        wgsl_code: Option<String>,
        rect: [f32; 4],
        uniforms: Vec<u8>,
    },
}

pub struct RenderOutput {
    pub node_commands: Vec<(u32, Vec<DisplayCommand>)>,
    pub texture_loads: Vec<(String, TextureSource)>,
    pub sparse_instances: Vec<(u32, Instance)>,
    pub sparse_texts: Vec<rore_types::text::SparseTextItem>,
    pub images: HashMap<String, Vec<Instance>>,
}

impl RenderOutput {
    pub fn new() -> Self {
        Self {
            node_commands: Vec::new(),
            texture_loads: Vec::new(),
            sparse_instances: Vec::new(),
            sparse_texts: Vec::new(),
            images: HashMap::new(),
        }
    }
    pub fn extend(&mut self, other: RenderOutput) {
        self.node_commands.extend(other.node_commands);
        self.sparse_instances.extend(other.sparse_instances);
        self.sparse_texts.extend(other.sparse_texts);
        for (id, list) in other.images {
            self.images.entry(id).or_insert_with(Vec::new).extend(list);
        }
        self.texture_loads.extend(other.texture_loads);
    }
}

pub enum Prop<T> {
    Static(T),
    Dynamic(Box<dyn FnMut() -> T + Send>),
}

pub trait IntoProp<T> {
    fn into_prop(self) -> Prop<T>;
}

macro_rules! impl_into_prop {
    ($t:ty) => {
        impl IntoProp<$t> for $t {
            fn into_prop(self) -> Prop<$t> {
                Prop::Static(self)
            }
        }
        impl<F> IntoProp<$t> for F
        where
            F: FnMut() -> $t + Send + 'static,
        {
            fn into_prop(self) -> Prop<$t> {
                Prop::Dynamic(Box::new(self))
            }
        }
    };
}
impl_into_prop!(f32);
impl_into_prop!(u32);
impl_into_prop!(i32);
impl_into_prop!(bool);
impl_into_prop!(Color);
impl_into_prop!(Style);
impl_into_prop!(String);

impl<T: Clone + Send + 'static> IntoProp<T> for crate::reactive::signals::Signal<T> {
    fn into_prop(self) -> Prop<T> {
        Prop::Dynamic(Box::new(move || self.get()))
    }
}
impl<T: Clone + PartialEq + Send + 'static> IntoProp<T> for crate::reactive::memo::Memo<T> {
    fn into_prop(self) -> Prop<T> {
        Prop::Dynamic(Box::new(move || self.get()))
    }
}
impl IntoProp<String> for &str {
    fn into_prop(self) -> Prop<String> {
        Prop::Static(self.to_string())
    }
}

pub trait Widget: Send + 'static {
    fn build(
        self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId;
    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput;
    fn handle_event(&mut self, _state: &mut FrameworkState, _event: &WidgetEvent) -> EventResult {
        EventResult::Ignored
    }
    fn rebuild(&mut self, _state: &mut FrameworkState, _engine: &mut LayoutEngine, _action: u32) {}
    fn is_interactive(&self) -> bool {
        false
    }
    fn type_name(&self) -> &'static str;
    fn visual_overflow(&self) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
