use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct CustomShaderManager {
    pub pipelines: HashMap<String, wgpu::RenderPipeline>,
    pub uniform_layout: wgpu::BindGroupLayout,
    pub builtin_buffer: Option<wgpu::Buffer>,
    pub custom_buffer: Option<wgpu::Buffer>,
    pub alignment: u32,
}

impl CustomShaderManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let alignment = device.limits().min_uniform_buffer_offset_alignment as u32;

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Custom Shader Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, // Kordinatalar (Rect, Clip)
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, // Foydalanuvchi yuborgan o'zgaruvchilar
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        Self {
            pipelines: HashMap::new(),
            uniform_layout,
            builtin_buffer: None,
            custom_buffer: None,
            alignment,
        }
    }

    pub fn compile(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_layout: &wgpu::BindGroupLayout,
        id: &str,
        wgsl: &str,
    ) {
        if self.pipelines.contains_key(id) {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Custom Shader {}", id)),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("Custom Pipeline Layout {}", id)),
            bind_group_layouts: &[camera_layout, &self.uniform_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Custom Pipeline {}", id)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // Vertex buffer yo'q!
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        self.pipelines.insert(id.to_string(), pipeline);
    }

    pub fn prepare_uniforms(
        &mut self,
        device: &wgpu::Device,
        draws: &[(String, [f32; 4], [f32; 4], Vec<u8>)],
    ) -> (Option<wgpu::BindGroup>, Vec<(u32, u32)>) {
        if draws.is_empty() {
            return (None, vec![]);
        }

        let mut builtins = Vec::new();
        let mut customs = Vec::new();
        let mut offsets = Vec::new();

        for (_, rect, clip, uniforms) in draws {
            let off0 = builtins.len() as u32;
            let off1 = customs.len() as u32;
            offsets.push((off0, off1));

            // Builtins: (32 bytes = 8 floats)
            builtins.extend_from_slice(bytemuck::cast_slice(rect));
            builtins.extend_from_slice(bytemuck::cast_slice(clip));
            let rem0 = builtins.len() as u32 % self.alignment;
            if rem0 != 0 {
                builtins.resize(builtins.len() + (self.alignment - rem0) as usize, 0);
            }

            // Customs
            let start_len = customs.len() as u32;
            if uniforms.is_empty() {
                customs.extend_from_slice(&[0u8; 16]); // Dummy ma'lumot
            } else {
                customs.extend_from_slice(uniforms);
            }
            let block_size = customs.len() as u32 - start_len;
            let mut padding = self.alignment - (block_size % self.alignment);
            if padding == 0 {
                padding = self.alignment;
            }
            customs.resize(customs.len() + padding as usize, 0);
        }

        let builtin_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Custom Builtins Mega-Buffer"),
            contents: &builtins,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let custom_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Custom Uniforms Mega-Buffer"),
            contents: &customs,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Custom Shader Mega Bind Group"),
            layout: &self.uniform_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &builtin_buf,
                        offset: 0,
                        size: wgpu::BufferSize::new(32), // 8 floats
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &custom_buf,
                        offset: 0,
                        size: wgpu::BufferSize::new(self.alignment as u64), // Dynamic block size
                    }),
                },
            ],
        });

        self.builtin_buffer = Some(builtin_buf);
        self.custom_buffer = Some(custom_buf);

        (Some(bg), offsets)
    }
}
