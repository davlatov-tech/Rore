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
    corner_radii: vec4<f32>,
    border_widths: vec4<f32>,
    extra_props: vec4<f32>,
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
    @location(2) clip_rect: vec4<f32>,
    @location(3) @interpolate(flat) style_index: u32,
};

@vertex
fn vs_main(model: VertexInput, inst: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let w_pos = inst.model_pos + (model.pos * inst.model_size);
    out.clip_pos = camera.view_proj * vec4<f32>(w_pos, 0.0, 1.0);
    out.uv = model.pos;
    out.size = inst.model_size;
    out.clip_rect = inst.clip_rect;
    out.style_index = inst.style_index;
    return out;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let style = styles[in.style_index];

    if (style.anim_data.y < 0.0) {
        let elapsed = time.current_time - style.anim_data.x;
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

    var current_color_start = style.color_start;
    var current_color_end = style.color_end;
    var current_border_color = style.border_color;

    if (style.anim_data.y > 0.0) {
        let elapsed_anim = max(0.0, time.current_time - style.anim_data.x);
        let t = saturate(elapsed_anim / style.anim_data.y);
        let ease_out_t = 1.0 - pow(1.0 - t, 3.0);

        current_color_start = mix(style.color_start, style.target_color_start, ease_out_t);
        current_color_end = mix(style.color_end, style.target_color_end, ease_out_t);
        current_border_color = mix(style.border_color, style.target_border_color, ease_out_t);
    }

    // INQILOB: Haqiqiy Linear Gradient
    let angle = style.extra_props.x;
    let dir = vec2<f32>(cos(angle), sin(angle));
    let centered_uv = in.uv - 0.5;
    let max_proj = abs(dir.x) * 0.5 + abs(dir.y) * 0.5;
    let proj = dot(centered_uv, dir);
    let grad_t = (proj / max(max_proj, 0.001)) * 0.5 + 0.5;
    let current_color = mix(current_color_start, current_color_end, saturate(grad_t));

    let p = (in.uv - 0.5) * in.size;
    let b = in.size * 0.5;

    // INQILOB: 4 xil radiusni 1 ta SDF orqali ifodalash (IQ math)
    let rs = select(style.corner_radii.xw, style.corner_radii.yz, p.x > 0.0);
    let border_radius = select(rs.x, rs.y, p.y > 0.0);

    let d = sd_rounded_box(p, b, border_radius);
    let smoothed_d = smoothstep(-0.5, 0.5, -d);

    var out_color = current_color;
    out_color.a = out_color.a * smoothed_d;

    // INQILOB: 4 xil tomonga alohida qalinlikdagi Border
    let d_top = p.y - (-b.y);
    let d_right = b.x - p.x;
    let d_bottom = b.y - p.y;
    let d_left = p.x - (-b.x);

    var border_alpha = 0.0;
    let bw = style.border_widths;
    if (bw.x > 0.0) { border_alpha = max(border_alpha, 1.0 - smoothstep(bw.x - 1.0, bw.x, d_top)); }
    if (bw.y > 0.0) { border_alpha = max(border_alpha, 1.0 - smoothstep(bw.y - 1.0, bw.y, d_right)); }
    if (bw.z > 0.0) { border_alpha = max(border_alpha, 1.0 - smoothstep(bw.z - 1.0, bw.z, d_bottom)); }
    if (bw.w > 0.0) { border_alpha = max(border_alpha, 1.0 - smoothstep(bw.w - 1.0, bw.w, d_left)); }

    border_alpha = border_alpha * smoothed_d;
    out_color = mix(out_color, current_border_color, border_alpha);

    // Soya qismi (Shadow)
    if (style.shadow_color.a > 0.0) {
        let shadow_offset = style.shadow_data.xy;
        let shadow_blur = style.shadow_data.z;
        let shadow_spread = style.shadow_data.w;
        let shadow_p = p - shadow_offset;

        let shadow_rs = select(style.corner_radii.xw, style.corner_radii.yz, shadow_p.x > 0.0);
        let shadow_radius = select(shadow_rs.x, shadow_rs.y, shadow_p.y > 0.0);

        let shadow_d = sd_rounded_box(shadow_p, b + vec2<f32>(shadow_spread), shadow_radius + shadow_spread);
        let shadow_alpha = 1.0 - smoothstep(-shadow_blur, shadow_blur, shadow_d);
        let shadow_final = style.shadow_color * shadow_alpha;

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
