use wgpu::util::DeviceExt;

// GPUga yuboriladigan ma'lumot (Uniform Buffer)
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // 4x4 Matritsa: Piksellarni -> GPU koordinatasiga o'giradi
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use glam::Mat4;
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    // Ekran o'lchami o'zgarganda chaqiriladi
    pub fn update_view_proj(&mut self, width: f32, height: f32) {
        use glam::Mat4;
        // Orthographic Projection: 
        // Chap-Yuqori (0,0) -> O'ng-Past (width, height)
        let projection = Mat4::orthographic_rh(
            0.0, width, 
            height, 0.0, 
            -1.0, 1.0
        );
        self.view_proj = projection.to_cols_array_2d();
    }
}

pub struct CameraState {
    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl CameraState {
    pub fn new(device: &wgpu::Device, width: f32, height: f32) -> Self {
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(width, height);

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
        });

        Self {
            uniform,
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: f32, height: f32) {
        self.uniform.update_view_proj(width, height);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}