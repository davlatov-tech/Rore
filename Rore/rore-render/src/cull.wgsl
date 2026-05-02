struct CullConfig {
    total_instances: u32,
    grid_width: u32,
    grid_height: u32,
    is_full_redraw: u32,
};

struct InstanceRaw {
    model_pos: vec2<f32>,
    model_size: vec2<f32>,
    clip_rect: vec4<f32>,
    style_index: u32,
    z_index: f32,
    padding: vec2<u32>,
};

@group(0) @binding(0) var<storage, read> instances_in: array<InstanceRaw>;
@group(0) @binding(1) var<storage, read_write> instances_out: array<InstanceRaw>;
@group(0) @binding(2) var<storage, read_write> indirect_buffer: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> cull_config: CullConfig;
@group(0) @binding(4) var<storage, read> draw_order: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let order_idx = global_id.x;
    if (order_idx >= cull_config.total_instances) {
        return;
    }

    let real_idx = draw_order[order_idx];
    var instance = instances_in[real_idx];

    let cx = instance.clip_rect.x;
    let cy = instance.clip_rect.y;
    let cw = instance.clip_rect.z;
    let ch = instance.clip_rect.w;

    let ix = instance.model_pos.x;
    let iy = instance.model_pos.y;
    let iw = instance.model_size.x;
    let ih = instance.model_size.y;

    let out_of_bounds = (ix + iw < cx) || (ix > cx + cw) || (iy + ih < cy) || (iy > cy + ch);

    if (out_of_bounds) {
        instance.model_size = vec2<f32>(0.0, 0.0);
    }

    instances_out[order_idx] = instance;

    if (order_idx == 0u) {
        atomicStore(&indirect_buffer[1], cull_config.total_instances);
    }
}
