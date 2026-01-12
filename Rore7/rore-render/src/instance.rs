// TUZATILDI: Import olib tashlandi
// use rore_types::{Color, Rect}; <-- BU KERAK EMAS

#[derive(Clone, Copy, Debug)]
pub struct Instance {
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub gradient_angle: f32,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
    pub shadow_color: [f32; 4],
    pub shadow_offset: glam::Vec2,
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub clip_rect: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model_pos: [f32; 2],
    model_size: [f32; 2],
    color_start: [f32; 4],
    color_end: [f32; 4],
    border_color: [f32; 4],
    shadow_color: [f32; 4],
    shadow_data: [f32; 4],
    properties: [f32; 4], 
    clip_rect: [f32; 4],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model_pos: self.position.to_array(),
            model_size: self.size.to_array(),
            color_start: self.color_start,
            color_end: self.color_end,
            border_color: self.border_color,
            shadow_color: self.shadow_color,
            shadow_data: [
                self.shadow_offset.x, 
                self.shadow_offset.y, 
                self.shadow_blur, 
                self.shadow_spread
            ],
            properties: [
                self.border_radius, 
                self.border_width, 
                self.gradient_angle.to_radians(),
                0.0 
            ],
            clip_rect: self.clip_rect,
        }
    }
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 8, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 16, shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 32, shader_location: 4, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 48, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 64, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 80, shader_location: 7, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 96, shader_location: 8, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 112, shader_location: 9, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
}