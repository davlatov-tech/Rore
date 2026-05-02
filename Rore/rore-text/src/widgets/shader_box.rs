use glam::Vec2;
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, DisplayCommand, Prop, RenderOutput, Widget};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;
use std::sync::{Arc, Mutex};

pub struct ShaderBox {
    pub id: Option<String>,
    pub style: Style,
    pub shader_id: String,
    pub wgsl_code: String,
    pub uniforms: Prop<Vec<u8>>,
    pub children: Vec<Box<dyn Widget>>,

    live_uniforms: Arc<Mutex<Vec<u8>>>,
    is_first_render: std::cell::Cell<bool>,
}

impl ShaderBox {
    pub fn new(shader_id: &str, wgsl_code: &str) -> Self {
        Self {
            id: None,
            style: Style::default(),
            shader_id: shader_id.to_string(),
            wgsl_code: wgsl_code.to_string(),
            uniforms: Prop::Static(vec![]),
            children: vec![],
            live_uniforms: Arc::new(Mutex::new(vec![])),
            is_first_render: std::cell::Cell::new(true),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    pub fn uniforms(mut self, bytes: Prop<Vec<u8>>) -> Self {
        self.uniforms = bytes;
        self
    }
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.children.push(Box::new(w));
        self
    }
}

impl Widget for ShaderBox {
    fn type_name(&self) -> &'static str {
        "ShaderBox"
    }
    fn is_interactive(&self) -> bool {
        false
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let mut child_nodes = Vec::new();
        for child in self.children.drain(..) {
            let child_id = child.build(arena, engine, ctx);
            if let Some(&t_node) = arena.taffy_map.get(&child_id) {
                child_nodes.push(t_node);
            }
        }
        let taffy_node = engine.new_node(self.style.clone(), &child_nodes);
        let my_id = arena.allocate_node();

        let u_prop = std::mem::replace(&mut self.uniforms, Prop::Static(vec![]));
        match u_prop {
            Prop::Static(b) => *self.live_uniforms.lock().unwrap() = b,
            Prop::Dynamic(mut f) => {
                let initial = f();
                *self.live_uniforms.lock().unwrap() = initial;
                let lu = self.live_uniforms.clone();
                rore_core::reactive::signals::create_effect(move || {
                    *lu.lock().unwrap() = f();
                    rore_core::reactive::command::CommandQueue::send(
                        rore_core::reactive::command::UICommand::MarkDirty(
                            my_id,
                            rore_core::state::DIRTY_COLOR,
                        ),
                    );
                });
            }
        }

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);
        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        _clip_rect: Option<[f32; 4]>,
        _path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);
        let my_id = *state.arena.node_map.get(&taffy_node).unwrap();

        let wgsl = if self.is_first_render.get() {
            self.is_first_render.set(false);
            Some(self.wgsl_code.clone())
        } else {
            None
        };

        let current_uniforms = self.live_uniforms.lock().unwrap().clone();

        let cmd = DisplayCommand::DrawCustomShader {
            shader_id: self.shader_id.clone(),
            wgsl_code: wgsl,
            rect: [layout.x, layout.y, layout.width, layout.height],
            uniforms: current_uniforms,
        };

        output.node_commands.push((my_id.0, vec![cmd]));
        output
    }
}
