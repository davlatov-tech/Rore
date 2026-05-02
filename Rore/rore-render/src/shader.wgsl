struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

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
@group(1) @binding(0) var<storage, read> styles: array<StyleRaw>;

struct TimeUniform {
    current_time: f32,
    grid_width: f32,
    grid_height: f32,
    is_full_redraw: f32,
    clear_color: vec4<f32>,
};
@group(2) @binding(0) var<uniform> time: TimeUniform;
@group(2) @binding(1) var<storage, read> tile_mask: array<u32>;

struct VertexInput {
    @location(0) pos: vec2<f32>,
};

struct InstanceInput {
    @location(1) model_pos: vec2<f32>,
    @location(2) model_size: vec2<f32>,
    @location(3) clip_rect: vec4<f32>,
    @location(4) style_index: u32,
    @location(5) z_index: f32,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color_start: vec4<f32>,
    @location(3) color_end: vec4<f32>,
    @location(4) target_color_start: vec4<f32>,
    @location(5) target_color_end: vec4<f32>,
    @location(6) border_color: vec4<f32>,
    @location(7) target_border_color: vec4<f32>,
    @location(8) shadow_color: vec4<f32>,
    @location(9) shadow_data: vec4<f32>,
    @location(10) properties: vec4<f32>,
    @location(11) clip_rect: vec4<f32>,
    @location(12) anim_data: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput, inst: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let style = styles[inst.style_index];

    let w_pos = inst.model_pos + (model.pos * inst.model_size);
    out.clip_pos = camera.view_proj * vec4<f32>(w_pos, 0.0, 1.0);

    out.uv = model.pos;
    out.size = inst.model_size;
    out.clip_rect = inst.clip_rect;
    out.color_start = style.color_start;
    out.color_end = style.color_end;
    out.target_color_start = style.target_color_start;
    out.target_color_end = style.target_color_end;
    out.border_color = style.border_color;
    out.target_border_color = style.target_border_color;
    out.shadow_color = style.shadow_color;
    out.shadow_data = style.shadow_data;
    out.properties = style.properties;
    out.anim_data = style.anim_data;

    return out;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    if (in.anim_data.y < 0.0) {
        let elapsed = time.current_time - in.anim_data.x;
        if (fract(elapsed) >= 0.5) {
            discard;
        }
    }

        let screen_pos = in.clip_pos.xy;
        if (screen_pos.x < in.clip_rect.x ||
            screen_pos.y < in.clip_rect.y ||
            screen_pos.x > in.clip_rect.x + in.clip_rect.z ||
            screen_pos.y > in.clip_rect.y + in.clip_rect.w) {
            discard;
        }


    var current_color = in.color_start;
    var current_border_color = in.border_color;

    if (in.anim_data.y > 0.0) {
        let elapsed_anim = max(0.0, time.current_time - in.anim_data.x);
        let t = saturate(elapsed_anim / in.anim_data.y);
        let ease_out_t = 1.0 - pow(1.0 - t, 3.0);

        current_color = mix(in.color_start, in.target_color_start, ease_out_t);
        current_border_color = mix(in.border_color, in.target_border_color, ease_out_t);
    }

    let border_radius = in.properties.x;
    let border_width = in.properties.y;

    let p = (in.uv - 0.5) * in.size;
    let b = in.size * 0.5;

    let d = sd_rounded_box(p, b, border_radius);

    let smoothed_d = smoothstep(-0.5, 0.5, -d);

    var out_color = current_color;
    out_color.a = out_color.a * smoothed_d;

    if (border_width > 0.0) {
        let border_d = sd_rounded_box(p, b - vec2<f32>(border_width), max(0.0, border_radius - border_width));
        let border_alpha = smoothstep(-0.5, 0.5, border_d) * smoothed_d;
        out_color = mix(out_color, current_border_color, border_alpha);
    }

    if (in.shadow_color.a > 0.0) {
        let shadow_offset = in.shadow_data.xy;
        let shadow_blur = in.shadow_data.z;
        let shadow_spread = in.shadow_data.w;
        let shadow_p = p - shadow_offset;
        let shadow_d = sd_rounded_box(shadow_p, b + vec2<f32>(shadow_spread), border_radius + shadow_spread);
        let shadow_alpha = 1.0 - smoothstep(-shadow_blur, shadow_blur, shadow_d);
        let shadow_final = in.shadow_color * shadow_alpha;

        out_color = vec4<f32>(
            mix(shadow_final.rgb, out_color.rgb, out_color.a),
            shadow_final.a + out_color.a * (1.0 - shadow_final.a)
        );
    }

    if (out_color.a < 0.001) {
        discard;
    }

    return out_color;
}
