use crate::widgets::shader_box::ShaderBox;
use glam::Vec2;
use rore_core::reactive::signals::Signal;
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, Prop, RenderOutput, Widget};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;

// GPU Tushunadigan C-Xotira Strukturasi
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AppleUniforms {
    pub time: f32,
    pub _padding: [f32; 3], // 16 bayt alignment (WGSL vec4 qoidasi)
}

// GPU da Suyuq Shisha matematikasi
const APPLE_GLASS_WGSL: &str = r#"
struct CameraUniform { view_proj: mat4x4<f32>, };
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct Builtins { rect: vec4<f32>, clip: vec4<f32>, };
@group(1) @binding(0) var<uniform> builtins: Builtins;

struct CustomData {
    time: f32,
    _padding: vec3<f32>,
};
@group(1) @binding(1) var<uniform> custom: CustomData;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOutput {
    let uv = vec2<f32>(f32(v_idx & 1u), f32(v_idx >> 1u));
    let pos = builtins.rect.xy + uv * builtins.rect.zw;

    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(pos, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = custom.time;

    // Apple Shisha fon
    var final_col = vec3<f32>(0.2, 0.5, 0.9);

    // Suyuq yorug'lik effektlari
    let p = uv * 3.0;
    let wave = sin(p.x * 10.0 + t * 2.0) * sin(p.y * 10.0 + t * 2.0) * 0.5 + 0.5;
    final_col += vec3<f32>(0.2, 0.4, 0.6) * wave;

    // Shisha Yaltirashi
    let spec = exp(-pow(uv.y - 0.5 + sin(uv.x * 10.0 + t * 0.5) * 0.1, 2.0) * 80.0);
    final_col += vec3<f32>(spec * 0.3);

    // Burchaklarni silliqlash
    let edge_x = smoothstep(0.0, 0.02, uv.x) * smoothstep(1.0, 0.98, uv.x);
    let edge_y = smoothstep(0.0, 0.02, uv.y) * smoothstep(1.0, 0.98, uv.y);
    let edge_factor = edge_x * edge_y;

    // 85% Shaffof suyuq oyna
    return vec4<f32>(final_col, 0.85 * edge_factor);
}
"#;

pub struct AppleGlassBox {
    pub style: Style,
    pub time_signal: Signal<f32>,
    pub child: Option<Box<dyn Widget>>,
}

impl AppleGlassBox {
    pub fn new(time_signal: Signal<f32>) -> Self {
        Self {
            style: Style::default(),
            time_signal,
            child: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(w));
        self
    }
}

impl Widget for AppleGlassBox {
    fn type_name(&self) -> &'static str {
        "AppleGlassBox"
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
        let t_sig = self.time_signal;

        let uniform_prop = Prop::Dynamic(Box::new(move || {
            let data = AppleUniforms {
                time: t_sig.get(),
                _padding: [0.0; 3],
            };
            bytemuck::bytes_of(&data).to_vec()
        }));

        let mut s_box = ShaderBox::new("apple_glass_v1", APPLE_GLASS_WGSL)
            .style(self.style.clone())
            .uniforms(uniform_prop);

        if let Some(child) = self.child.take() {
            s_box.children.push(child);
        }

        Box::new(s_box).build(arena, engine, ctx)
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
