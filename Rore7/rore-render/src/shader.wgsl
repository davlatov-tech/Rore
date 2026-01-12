struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>, // 0..1 quad
};

struct InstanceInput {
    @location(1) model_pos: vec2<f32>,
    @location(2) model_size: vec2<f32>,
    @location(3) color_start: vec4<f32>,
    @location(4) color_end: vec4<f32>,
    @location(5) border_color: vec4<f32>,
    @location(6) shadow_color: vec4<f32>,
    @location(7) shadow_data: vec4<f32>, // x, y, blur, spread
    @location(8) properties: vec4<f32>,  // radius, border_width, angle, padding
    // YANGI: Clipping
    @location(9) clip_rect: vec4<f32>,   // x, y, w, h
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,   // Pikseldagi koordinata (center relative)
    @location(1) half_size: vec2<f32>,
    @location(2) color_start: vec4<f32>,
    @location(3) color_end: vec4<f32>,
    @location(4) border_color: vec4<f32>,
    @location(5) shadow_color: vec4<f32>,
    @location(6) shadow_data: vec4<f32>,
    @location(7) radius: f32,
    @location(8) border_width: f32,
    @location(9) angle: f32,
    // YANGI: Dunyo koordinatasi (Clipping uchun)
    @location(10) world_pos: vec2<f32>,
    @location(11) clip_rect: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    // 1. Soyani hisobga olish uchun Quadni kengaytiramiz
    // Soya qancha uzoqqa ketsa (offset + blur + spread), bizga shuncha joy kerak
    // Xavfsizlik uchun har tomondan 50px yoki shadow_size qadar kengaytiramiz
    let extra_margin = max(instance.shadow_data.z + instance.shadow_data.w + length(instance.shadow_data.xy), 0.0) + 2.0;
    
    // Asosiy o'lcham
    let size = instance.model_size;
    
    // Vertex pozitsiyasini kengaytiramiz
    // model.position 0..1 oralig'ida. Biz uni -margin .. size+margin oralig'iga cho'zamiz
    let raw_pos = model.position * size; // 0..width, 0..height
    let expanded_pos = raw_pos + (model.position - 0.5) * 2.0 * extra_margin;

    // Dunyo koordinatasi
    let world_pos = instance.model_pos + expanded_pos;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);

    // Fragment shader uchun ma'lumotlar
    out.local_pos = expanded_pos - (size * 0.5); // Markaz (0,0) da bo'lishi uchun
    out.half_size = size * 0.5;
    
    out.color_start = instance.color_start;
    out.color_end = instance.color_end;
    out.border_color = instance.border_color;
    out.shadow_color = instance.shadow_color;
    out.shadow_data = instance.shadow_data;
    out.radius = instance.properties.x;
    out.border_width = instance.properties.y;
    out.angle = instance.properties.z;
    
    // YANGI
    out.world_pos = world_pos;
    out.clip_rect = instance.clip_rect;

    return out;
}

// Rounded Box SDF
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // --- 0. CLIPPING LOGIC ---
    // Agar bizga clip_rect berilgan bo'lsa (width > 0 va height > 0)
    // Va piksel shu to'rtburchakdan tashqarida bo'lsa, uni chizmaymiz.
    let cx = in.clip_rect.x;
    let cy = in.clip_rect.y;
    let cw = in.clip_rect.z;
    let ch = in.clip_rect.w;

    // Kichik optimizatsiya: Agar w yoki h juda katta bo'lsa, tekshirmasa ham bo'ladi,
    // lekin hozircha oddiy if ishlatamiz.
    if (in.world_pos.x < cx || in.world_pos.x > cx + cw ||
        in.world_pos.y < cy || in.world_pos.y > cy + ch) {
        discard;
    }

    // 1. Asosiy shakl masofasi (Distance)
    // Radiusni o'lchamga moslashtiramiz (xatolik bo'lmasligi uchun)
    let r = min(in.radius, min(in.half_size.x, in.half_size.y));
    let dist_box = sd_rounded_box(in.local_pos, in.half_size, r);

    // 2. Soya (Shadow) chizish
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    // Agar soya rangi shaffof bo'lmasa
    if (in.shadow_color.a > 0.0) {
        let shadow_offset = in.shadow_data.xy;
        let shadow_blur = in.shadow_data.z;
        let shadow_spread = in.shadow_data.w;
        
        // Soya uchun SDF (joylashuvi siljigan)
        let dist_shadow = sd_rounded_box(in.local_pos - shadow_offset, in.half_size + shadow_spread, r);
        
        // Yumshoqlik (Blur)
        // smoothstep yordamida soyani eritamiz
        let shadow_alpha = 1.0 - smoothstep(-shadow_blur, 0.0, dist_shadow);
        
        // Soya rangi
        let shadow = vec4<f32>(in.shadow_color.rgb, in.shadow_color.a * shadow_alpha);
        
        // Soyani asosiy fonga yozamiz
        final_color = shadow;
    }

    // 3. Asosiy Shakl (Main Shape)
    // Anti-aliasing (1px yumshoqlik)
    let smoothing = 1.0; 
    let alpha_box = 1.0 - smoothstep(0.0, smoothing, dist_box);

    if (alpha_box > 0.0) {
        // Gradient hisoblash
        // Burish (Rotation)
        let s = sin(-in.angle);
        let c = cos(-in.angle);
        let rotated_pos = vec2<f32>(
            in.local_pos.x * c - in.local_pos.y * s,
            in.local_pos.x * s + in.local_pos.y * c
        );
        
        // Gradient factor (-size..size oralig'idan 0..1 ga o'tkazish)
        let gradient_factor = clamp((rotated_pos.y + in.half_size.y) / (in.half_size.y * 2.0), 0.0, 1.0);
        
        let shape_color = mix(in.color_start, in.color_end, gradient_factor);
        
        // Border (Chegara)
        let border_alpha = 1.0 - smoothstep(in.border_width - smoothing, in.border_width, abs(dist_box));
        
        // Chegarani chizish (Agar border_width > 0)
        let fill_color = mix(shape_color, in.border_color, border_alpha * step(0.1, in.border_width));

        // Shaklni soya ustiga qatlamlash (Alpha blending)
        let src_a = fill_color.a * alpha_box;
        let dst_a = final_color.a * (1.0 - src_a); 
        
        let out_a = src_a + dst_a;
        if (out_a > 0.0) {
            let out_rgb = (fill_color.rgb * src_a + final_color.rgb * dst_a) / out_a;
            final_color = vec4<f32>(out_rgb, out_a);
        }
    }

    if (final_color.a <= 0.0) {
        discard;
    }

    return final_color;
}