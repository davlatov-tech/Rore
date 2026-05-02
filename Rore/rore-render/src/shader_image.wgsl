// 0. Kamera (Global)
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// 1. Texture va Sampler
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

// 2. SSBO Uslublar Lug'ati
struct StyleRaw {
    color_start: vec4<f32>,
    color_end: vec4<f32>,
    target_color_start: vec4<f32>,
    target_color_end: vec4<f32>,
    border_color: vec4<f32>,
    target_border_color: vec4<f32>,
    shadow_color: vec4<f32>,
    shadow_data: vec4<f32>,
    properties: vec4<f32>,
    anim_data: vec4<f32>,
};
@group(2) @binding(0) var<storage, read> styles: array<StyleRaw>;

struct TimeUniform {
    current_time: f32,
};
@group(3) @binding(0) var<uniform> global_time: TimeUniform;

struct VertexInput {
    @location(0) position: vec2<f32>, // 0..1
};

struct InstanceInput {
    @location(1) model_pos: vec2<f32>,
    @location(2) model_size: vec2<f32>,
    @location(3) clip_rect: vec4<f32>,
    @location(4) style_index: u32,
    @location(5) z_index: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) world_pos: vec2<f32>,
    @location(4) clip_rect: vec4<f32>,
    @interpolate(flat) @location(5) style_index: u32,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let size = instance.model_size;
    let world_pos = instance.model_pos + (model.position * size);

    // Z-Buffer
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, instance.z_index, 1.0);

    out.tex_coords = model.position;
    out.local_pos = (model.position - 0.5) * size;
    out.half_size = size * 0.5;
    out.world_pos = world_pos;
    out.clip_rect = instance.clip_rect;
    out.style_index = instance.style_index;

    return out;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cx = in.clip_rect.x; let cy = in.clip_rect.y;
    let cw = in.clip_rect.z; let ch = in.clip_rect.w;

    if (in.world_pos.x < cx || in.world_pos.x > cx + cw ||
        in.world_pos.y < cy || in.world_pos.y > cy + ch) {
        discard;
    }

    let style = styles[in.style_index];
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    if (style.properties.w == 1.0) {
        // MSDF Median formulasi
        let median = max(min(tex_color.r, tex_color.g), min(max(tex_color.r, tex_color.g), tex_color.b));

        // Hardware fwidth() orqali istalgan masshtabda mutlaq tiniqlik
        let w = max(fwidth(median), 0.01);
        let alpha = smoothstep(0.5 - w, 0.5 + w, median);

        if (alpha <= 0.01) { discard; }

        // Vektor rangini beramiz
        return vec4<f32>(style.color_start.rgb, style.color_start.a * alpha);
    }

    let radius = style.properties.x;
    let r = min(radius, min(in.half_size.x, in.half_size.y));
    let dist = sd_rounded_box(in.local_pos, in.half_size, r);

    let smoothing = max(fwidth(dist), 0.5);
    let alpha_shape = 1.0 - smoothstep(-smoothing, smoothing, dist);

    if (alpha_shape <= 0.0) {
        discard;
    }

    return vec4<f32>(tex_color.rgb, tex_color.a * alpha_shape);
}
