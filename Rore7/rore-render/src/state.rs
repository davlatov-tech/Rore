use wgpu::util::DeviceExt;
use winit::window::Window;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::{
    vertex::Vertex, 
    camera::CameraState, 
    instance::{Instance, InstanceRaw}, 
    text::{TextSystem, FontManager}, 
    texture::Texture
};

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    render_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout, 
    
    pub textures: HashMap<String, Texture>,
    pub image_batches: HashMap<String, Vec<Instance>>,
    
    pub image_instance_buffer: wgpu::Buffer,
    pub image_instance_buffer_size: wgpu::BufferAddress,

    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    camera: CameraState,
    num_instances: u32,
    pub text_system: TextSystem,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window, font_manager: Arc<Mutex<FontManager>>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Rore Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera = CameraState::new(&device, size.width as f32, size.height as f32);
        
        // --- SHAPE PIPELINE ---
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera.bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
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
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // --- IMAGE PIPELINE ---
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let image_shader = device.create_shader_module(wgpu::include_wgsl!("shader_image.wgsl"));
        let image_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Image Pipeline Layout"),
            bind_group_layouts: &[&camera.bind_group_layout, &texture_bind_group_layout], 
            push_constant_ranges: &[],
        });

        let image_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Pipeline"),
            layout: Some(&image_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &image_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()], 
            },
            fragment: Some(wgpu::FragmentState {
                module: &image_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // --- BUFFERS ---
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(Vertex::QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceRaw>() * 10000) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let image_instance_buffer_size = (std::mem::size_of::<InstanceRaw>() * 20000) as wgpu::BufferAddress;
        let image_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Image Instance Buffer"),
            size: image_instance_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text_system = TextSystem::new(&device, &queue, &config, font_manager);

        Self {
            surface, device, queue, config, size,
            render_pipeline, image_pipeline, texture_bind_group_layout,
            
            textures: HashMap::new(),
            image_batches: HashMap::new(),
            
            image_instance_buffer,
            image_instance_buffer_size,

            vertex_buffer, instance_buffer, camera, num_instances: 0, text_system,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.resize(&self.queue, new_size.width as f32, new_size.height as f32);
            self.text_system.resize(new_size.width, new_size.height);
        }
    }

    pub fn update_instances(&mut self, instances: &[Instance]) {
        self.num_instances = instances.len() as u32;
        let raw_data: Vec<InstanceRaw> = instances.iter().map(Instance::to_raw).collect();
        if !raw_data.is_empty() {
             self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&raw_data));
        }
    }

    pub fn load_texture(&mut self, id: &str, bytes: &[u8]) {
        if !self.textures.contains_key(id) {
            println!("Loading Texture: {}", id);
            if let Ok(texture) = Texture::from_bytes(
                &self.device, 
                &self.queue, 
                bytes, 
                Some(id), 
                &self.texture_bind_group_layout
            ) {
                self.textures.insert(id.to_string(), texture);
            } else {
                eprintln!("Failed to load texture: {}", id);
            }
        }
    }

    pub fn queue_image(&mut self, texture_id: &str, instance: Instance) {
        self.image_batches
            .entry(texture_id.to_string())
            .or_insert_with(Vec::new)
            .push(instance);
    }

    // --- FIX: Argument yetishmasligi tuzatildi ---
    pub fn update_text(&mut self, text: &str) {
        use rore_types::Color;
        // 7-argument (width_limit) qo'shildi: f32::INFINITY
        self.text_system.queue_text(text, Color::BLACK, 24.0, 50.0, 50.0, None, f32::INFINITY);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.text_system.prepare(&self.device, &self.queue);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.11, g: 0.11, b: 0.18, a: 1.0 }), 
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let sc_w = self.size.width.max(1);
            let sc_h = self.size.height.max(1);
            render_pass.set_scissor_rect(0, 0, sc_w, sc_h);

            if self.num_instances > 0 {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.draw(0..4, 0..self.num_instances);
            }

            if !self.image_batches.is_empty() {
                render_pass.set_pipeline(&self.image_pipeline);
                render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                
                let mut current_offset: wgpu::BufferAddress = 0;

                for (id, instances) in &self.image_batches {
                    if let Some(texture) = self.textures.get(id) {
                        let raw_data: Vec<InstanceRaw> = instances.iter().map(Instance::to_raw).collect();
                        let batch_bytes = bytemuck::cast_slice(&raw_data);
                        let batch_size = batch_bytes.len() as wgpu::BufferAddress;

                        if current_offset + batch_size > self.image_instance_buffer_size {
                            eprintln!("Image Buffer Overflow! Max size exceeded.");
                            break; 
                        }

                        self.queue.write_buffer(&self.image_instance_buffer, current_offset, batch_bytes);
                        
                        render_pass.set_bind_group(1, &texture.bind_group, &[]);
                        render_pass.set_vertex_buffer(1, self.image_instance_buffer.slice(current_offset..(current_offset + batch_size)));
                        render_pass.draw(0..4, 0..instances.len() as u32);

                        current_offset += batch_size;
                    }
                }
            }

            self.text_system.render(&mut render_pass);
        }

        self.image_batches.clear();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}