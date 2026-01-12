// Kamera (Global)
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Texture va Sampler
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>, // 0..1
};

struct InstanceInput {
    @location(1) model_pos: vec2<f32>,
    @location(2) model_size: vec2<f32>,
    @location(3) color_start: vec4<f32>,
    @location(4) color_end: vec4<f32>,
    @location(5) border_color: vec4<f32>,
    @location(6) shadow_color: vec4<f32>,
    @location(7) shadow_data: vec4<f32>, 
    @location(8) properties: vec4<f32>,  // x: radius
    @location(9) clip_rect: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    
    // YANGI: SDF hisoblash uchun kerakli ma'lumotlar
    @location(1) local_pos: vec2<f32>,   // Markazga nisbatan koordinata
    @location(2) half_size: vec2<f32>,   // Yarim o'lcham
    @location(3) radius: f32,            // Radius
    @location(4) world_pos: vec2<f32>,   // Clipping uchun
    @location(5) clip_rect: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    let size = instance.model_size;
    
    // Dunyo koordinatasi
    let world_pos = instance.model_pos + (model.position * size);
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    
    out.tex_coords = model.position; 
    
    // YANGI: Markazga nisbatan koordinata (-width/2 .. +width/2)
    // model.position 0..1 bo'lgani uchun, undan 0.5 ayirib markazlashtiramiz
    out.local_pos = (model.position - 0.5) * size;
    out.half_size = size * 0.5;
    out.radius = instance.properties.x;
    
    out.world_pos = world_pos;
    out.clip_rect = instance.clip_rect;
    
    return out;
}

// Rounded Box formulasi (Asosiy shaderdan olindi)
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. TASHQI CLIPPING (Scroll uchun)
    let cx = in.clip_rect.x;
    let cy = in.clip_rect.y;
    let cw = in.clip_rect.z;
    let ch = in.clip_rect.w;

    if (in.world_pos.x < cx || in.world_pos.x > cx + cw ||
        in.world_pos.y < cy || in.world_pos.y > cy + ch) {
        discard;
    }

    // 2. ROUNDED CORNERS (SDF)
    // Radius o'lchamdan katta bo'lib ketmasligi kerak
    let r = min(in.radius, min(in.half_size.x, in.half_size.y));
    let dist = sd_rounded_box(in.local_pos, in.half_size, r);

    // Anti-aliasing (Burchaklarni silliqlash)
    // Agar masofa (dist) > 0 bo'lsa, demak piksel shakl tashqarisida.
    // smoothstep yordamida qirrasini yumshatamiz.
    let alpha_shape = 1.0 - smoothstep(0.0, 1.0, dist);

    if (alpha_shape <= 0.0) {
        discard;
    }

    // 3. TEXTURE SAMPLE
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Yakuniy rang: Rasm rangi * Shakl shakli (Alpha Mask)
    return vec4<f32>(tex_color.rgb, tex_color.a * alpha_shape);
}