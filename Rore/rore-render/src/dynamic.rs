use crate::camera::CameraState;
use crate::instance::{Instance, InstanceRaw, StyleRaw};
use crate::state::{CullConfigUniform, State, TimeUniform};
use crate::vertex::Vertex;
use rore_types::text::TextRenderer;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use winit::window::Window;

pub(crate) fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: config.width.max(1),
        height: config.height.max(1),
        depth_or_array_layers: 1,
    };
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

impl<'a> State<'a> {
    pub async fn new<F>(window: &'a Window, text_renderer_factory: F) -> Self
    where
        F: FnOnce(
            &wgpu::Device,
            &wgpu::Queue,
            &wgpu::SurfaceConfiguration,
        ) -> Box<dyn TextRenderer>,
    {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Rore Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera = CameraState::new(&device, size.width as f32, size.height as f32);
        let depth_texture_view = create_depth_texture(&device, &config);
        let initial_capacity = 10_000;

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Uniform Buffer"),
            contents: bytemuck::cast_slice(&[TimeUniform {
                current_time: 0.0,
                grid_width: 0.0,
                grid_height: 0.0,
                is_full_redraw: 1.0,
                clear_color: [0.0, 0.0, 0.0, 1.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let time_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("time_bind_group_layout"),
            });

        let tile_mask_buffers: [wgpu::Buffer; 3] = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Tile Mask Buffer {}", i)),
                size: 16384,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let time_bind_groups: [wgpu::BindGroup; 3] = std::array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &time_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: time_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: tile_mask_buffers[i].as_entire_binding(),
                    },
                ],
                label: Some(&format!("time_bind_group {}", i)),
            })
        });

        let style_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Style SSBO Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let style_buffers: [wgpu::Buffer; 3] = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Style Buffer {}", i)),
                size: (std::mem::size_of::<StyleRaw>() as u32 * initial_capacity)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let style_bind_groups: [wgpu::BindGroup; 3] = std::array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Style Bind Group {}", i)),
                layout: &style_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: style_buffers[i].as_entire_binding(),
                }],
            })
        });

        let cull_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cull Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let instances_in_buffers: [wgpu::Buffer; 3] = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Instances In Buffer {}", i)),
                size: (std::mem::size_of::<InstanceRaw>() as u32 * initial_capacity)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let instance_out_buffers: [wgpu::Buffer; 3] = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Instances Out Buffer {}", i)),
                size: (std::mem::size_of::<InstanceRaw>() as u32 * initial_capacity)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let indirect_buffers: [wgpu::Buffer; 3] = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Indirect Buffer {}", i)),
                size: std::mem::size_of::<[u32; 4]>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let cull_config_buffers: [wgpu::Buffer; 3] = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Cull Config Buffer {}", i)),
                size: std::mem::size_of::<CullConfigUniform>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let draw_order_buffers: [wgpu::Buffer; 3] = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Draw Order Buffer {}", i)),
                size: (std::mem::size_of::<u32>() as u32 * initial_capacity) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let cull_bind_groups: [wgpu::BindGroup; 3] = std::array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Cull Bind Group {}", i)),
                layout: &cull_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: instances_in_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: instance_out_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: indirect_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: cull_config_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: draw_order_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: tile_mask_buffers[i].as_entire_binding(),
                    },
                ],
            })
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        // 1. ASOSIY UI LAYOUT
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.bind_group_layout, // Group 0
                    &style_bind_group_layout,  // Group 1
                    &time_bind_group_layout,   // Group 2
                ],
                push_constant_ranges: &[],
            });

        // 2. STENCILLAR (TEMIR QOLIPLAR)
        let eraser_stencil = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace, // O'chirg'ich 1 bosadi
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        };

        let ui_stencil = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xff,
                write_mask: 0x00,
            },
            bias: wgpu::DepthBiasState::default(),
        };

        // 3. ASOSIY UI PIPELINE
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
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // 4. O'CHIRG'ICH (ERASER) SHADERI
        let eraser_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Eraser Shader"),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                        r#"
                        struct CameraUniform { view_proj: mat4x4<f32>, };
                        @group(0) @binding(0) var<uniform> camera: CameraUniform;

                        struct TimeUniform {
                            current_time: f32,
                            grid_width: f32,
                            grid_height: f32,
                            is_full_redraw: f32,
                            clear_color: vec4<f32>,
                        };
                        @group(1) @binding(0) var<uniform> time: TimeUniform;
                        @group(1) @binding(1) var<storage, read> tile_mask: array<u32>;

                        struct VertexInput { @location(0) pos: vec2<f32>, };
                        struct InstanceInput {
                            @location(1) model_pos: vec2<f32>,
                            @location(2) model_size: vec2<f32>,
                        };
                        struct VertexOutput { @builtin(position) clip_pos: vec4<f32>, };

                        @vertex fn vs_main(model: VertexInput, inst: InstanceInput) -> VertexOutput {
                            var out: VertexOutput;
                            let w_pos = inst.model_pos + (model.pos * inst.model_size);
                            out.clip_pos = camera.view_proj * vec4<f32>(w_pos, 0.0, 1.0);
                            return out;
                        }

                        @fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                            let px = u32(in.clip_pos.x);
                            let py = u32(in.clip_pos.y);
                            let tx = px / 64u;
                            let ty = py / 64u;
                            let gw = u32(time.grid_width);
                            if (tx < gw && ty < u32(time.grid_height)) {
                                let tile_idx = ty * gw + tx;
                                let arr_idx = tile_idx / 32u;
                                let bit_idx = tile_idx % 32u;
                                let mask = tile_mask[arr_idx];
                                if (((mask >> bit_idx) & 1u) == 0u) {
                                    discard;
                                }
                            }
                            return time.clear_color;
                        }
                        "#
                    )),
                });

        // 5. O'CHIRG'ICH LAYOUTI
        let eraser_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Eraser Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.bind_group_layout, // Group 0
                    &time_bind_group_layout,   // Group 1
                ],
                push_constant_ranges: &[],
            });

        let eraser_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Eraser Pipeline"),
            layout: Some(&eraser_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &eraser_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &eraser_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let cull_shader = device.create_shader_module(wgpu::include_wgsl!("cull.wgsl"));
        let cull_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cull Pipeline Layout"),
            bind_group_layouts: &[&cull_bind_group_layout],
            push_constant_ranges: &[],
        });

        let cull_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cull Pipeline"),
            layout: Some(&cull_pipeline_layout),
            module: &cull_shader,
            entry_point: "main",
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let image_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Image Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.bind_group_layout,
                    &texture_bind_group_layout,
                    &style_bind_group_layout,
                    &time_bind_group_layout,
                ],
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
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let offscreen_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Master Texture"),
            size: wgpu::Extent3d {
                width: size.width.max(1),
                height: size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let offscreen_view = offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let offscreen_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let offscreen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&offscreen_sampler),
                },
            ],
            label: Some("Offscreen Bind Group"),
        });

        let offscreen_instance_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Offscreen Instance Quad"),
                contents: bytemuck::cast_slice(&[InstanceRaw {
                    model_pos: [0.0, 0.0],
                    model_size: [size.width as f32, size.height as f32],
                    clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
                    style_index: 0,
                    z_index: -999.0,
                    padding: [0, 0],
                }]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Composite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                r#"
struct CameraUniform { view_proj: mat4x4<f32>, };
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;
struct VertexInput { @location(0) pos: vec2<f32>, };
struct InstanceInput { @location(1) model_pos: vec2<f32>, @location(2) model_size: vec2<f32>, @location(3) clip_rect: vec4<f32>, @location(4) style_index: u32, @location(5) z_index: f32, };
struct VertexOutput { @builtin(position) clip_pos: vec4<f32>, @location(0) uv: vec2<f32>, };
@vertex fn vs_main(model: VertexInput, inst: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let w_pos = inst.model_pos + (model.pos * inst.model_size);
    out.clip_pos = camera.view_proj * vec4<f32>(w_pos, 0.0, 1.0);
    out.uv = model.pos; return out;
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { return textureSample(t_diffuse, s_diffuse, in.uv); }
"#,
            )),
        });

        let composite_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Composite Pipeline Layout"),
                bind_group_layouts: &[&camera.bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Composite Pipeline"),
            layout: Some(&composite_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &composite_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &composite_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(Vertex::QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let image_instance_buffer_size =
            (std::mem::size_of::<InstanceRaw>() * 20000) as wgpu::BufferAddress;
        let image_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Image Buffer"),
            size: image_instance_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text_system = text_renderer_factory(&device, &queue, &config);
        let custom_shaders = crate::custom_shader::CustomShaderManager::new(&device);
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            eraser_pipeline, // INQILOB
            image_pipeline,
            cull_pipeline,
            composite_pipeline,
            frame_index: 0,
            current_capacity: initial_capacity,
            low_memory_timer: None,
            cull_bind_groups,
            instances_in_buffers,
            indirect_buffers,
            cull_config_buffers,
            instance_out_buffers,
            draw_order_buffers,
            tile_mask_buffers,
            texture_bind_group_layout,
            style_bind_group_layout,
            cull_bind_group_layout,
            style_bind_groups,
            style_buffers,
            node_to_gpu_idx: HashMap::new(),
            gpu_free_list: Vec::new(),
            styles_cache: Vec::with_capacity(initial_capacity as usize),
            instances_cache: Vec::with_capacity(initial_capacity as usize),
            time_buffer,
            time_bind_group_layout,
            time_bind_groups,
            global_time: 0.0,
            animation_end_time: 0.0,
            depth_texture_view,
            textures: HashMap::new(),
            image_batches: HashMap::new(),
            image_instance_buffer,
            image_instance_buffer_size,
            vertex_buffer,
            camera,
            num_instances: 0,
            current_draw_count: 0,
            text_system,
            offscreen_texture,
            offscreen_view,
            offscreen_sampler,
            offscreen_bind_group,
            offscreen_instance_buffer,
            custom_shaders,
            current_custom_draws: Vec::new(),
            custom_bind_group: None,
            custom_offsets: Vec::new(),
        }
    }

    pub(crate) fn resize_buffers(&mut self, new_capacity: u32) {
        if new_capacity == self.current_capacity || new_capacity < 10_000 {
            return;
        }
        self.current_capacity = new_capacity;

        if self.styles_cache.capacity() < new_capacity as usize {
            self.styles_cache
                .reserve(new_capacity as usize - self.styles_cache.len());
            self.instances_cache
                .reserve(new_capacity as usize - self.instances_cache.len());
        }

        for i in 0..3 {
            self.style_buffers[i] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Style Buffer {}", i)),
                size: (std::mem::size_of::<StyleRaw>() as u32 * new_capacity)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.style_bind_groups[i] = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Style Bind Group {}", i)),
                layout: &self.style_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.style_buffers[i].as_entire_binding(),
                }],
            });

            self.instances_in_buffers[i] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Instances In Buffer {}", i)),
                size: (std::mem::size_of::<InstanceRaw>() as u32 * new_capacity)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.instance_out_buffers[i] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Instances Out Buffer {}", i)),
                size: (std::mem::size_of::<InstanceRaw>() as u32 * new_capacity)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.draw_order_buffers[i] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Draw Order Buffer {}", i)),
                size: (std::mem::size_of::<u32>() as u32 * new_capacity) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.cull_bind_groups[i] = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Cull Bind Group {}", i)),
                layout: &self.cull_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.instances_in_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.instance_out_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.indirect_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.cull_config_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.draw_order_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.tile_mask_buffers[i].as_entire_binding(),
                    },
                ],
            });
        }
    }

    pub fn load_texture(&mut self, _id: &str, _bytes: &[u8]) {}
    pub fn queue_image(&mut self, id: &str, instance: Instance) {
        self.image_batches
            .entry(id.to_string())
            .or_insert_with(Vec::new)
            .push(instance);
    }
}
