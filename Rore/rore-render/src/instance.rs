use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug)]
pub struct Instance {
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    // Animatsiya maqsadlari
    pub target_color_start: [f32; 4],
    pub target_color_end: [f32; 4],
    pub gradient_angle: f32,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
    pub target_border_color: [f32; 4],
    pub shadow_color: [f32; 4],
    pub shadow_offset: glam::Vec2,
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub clip_rect: [f32; 4],
    // Animatsiya vaqti
    pub anim_start_time: f32,
    pub anim_duration: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, PartialEq)]
pub struct StyleRaw {
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub target_color_start: [f32; 4],
    pub target_color_end: [f32; 4],
    pub border_color: [f32; 4],
    pub target_border_color: [f32; 4],
    pub shadow_color: [f32; 4],
    pub shadow_data: [f32; 4], // x, y, blur, spread
    pub properties: [f32; 4],  // radius, border_width, angle, padding
    pub anim_data: [f32; 4],   // start_time, duration, unused, unused
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct InstanceRaw {
    pub model_pos: [f32; 2],
    pub model_size: [f32; 2],
    pub clip_rect: [f32; 4],
    pub style_index: u32,
    pub z_index: f32,
    pub padding: [u32; 2],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 36,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
