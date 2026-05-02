use crate::{
    camera::CameraState,
    instance::{Instance, InstanceRaw, StyleRaw},
    texture::Texture,
};
use rore_types::text::TextRenderer;
use std::collections::HashMap;
use std::time::Instant;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct TimeUniform {
    pub current_time: f32,
    pub grid_width: f32,
    pub grid_height: f32,
    pub is_full_redraw: f32,
    pub clear_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CullConfigUniform {
    pub total_instances: u32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub is_full_redraw: u32,
}

pub struct State<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) eraser_pipeline: wgpu::RenderPipeline,
    pub(crate) image_pipeline: wgpu::RenderPipeline,
    pub(crate) cull_pipeline: wgpu::ComputePipeline,
    pub(crate) composite_pipeline: wgpu::RenderPipeline,

    pub(crate) frame_index: usize,
    pub(crate) current_capacity: u32,
    pub(crate) low_memory_timer: Option<Instant>,

    pub(crate) cull_bind_groups: [wgpu::BindGroup; 3],
    pub(crate) instances_in_buffers: [wgpu::Buffer; 3],
    pub(crate) indirect_buffers: [wgpu::Buffer; 3],
    pub(crate) cull_config_buffers: [wgpu::Buffer; 3],
    pub(crate) instance_out_buffers: [wgpu::Buffer; 3],
    pub(crate) draw_order_buffers: [wgpu::Buffer; 3],

    pub(crate) tile_mask_buffers: [wgpu::Buffer; 3],

    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub style_bind_group_layout: wgpu::BindGroupLayout,
    pub cull_bind_group_layout: wgpu::BindGroupLayout,

    pub style_bind_groups: [wgpu::BindGroup; 3],
    pub style_buffers: [wgpu::Buffer; 3],

    pub(crate) node_to_gpu_idx: HashMap<u32, u32>,
    pub(crate) gpu_free_list: Vec<u32>,

    pub(crate) styles_cache: Vec<StyleRaw>,
    pub(crate) instances_cache: Vec<InstanceRaw>,

    pub(crate) time_buffer: wgpu::Buffer,
    pub(crate) time_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) time_bind_groups: [wgpu::BindGroup; 3],

    pub(crate) depth_texture_view: wgpu::TextureView,

    pub textures: HashMap<String, Texture>,
    pub image_batches: HashMap<String, Vec<Instance>>,
    pub image_instance_buffer: wgpu::Buffer,
    pub image_instance_buffer_size: wgpu::BufferAddress,

    pub(crate) vertex_buffer: wgpu::Buffer,
    pub camera: CameraState,
    pub(crate) num_instances: u32,

    pub(crate) current_draw_count: u32,
    pub custom_shaders: crate::custom_shader::CustomShaderManager,
    pub current_custom_draws: Vec<(String, [f32; 4], [f32; 4], Vec<u8>)>,
    pub custom_bind_group: Option<wgpu::BindGroup>,
    pub custom_offsets: Vec<(u32, u32)>,
    pub text_system: Box<dyn TextRenderer>,

    pub(crate) offscreen_texture: wgpu::Texture,
    pub(crate) offscreen_view: wgpu::TextureView,
    pub(crate) offscreen_sampler: wgpu::Sampler,
    pub(crate) offscreen_bind_group: wgpu::BindGroup,
    pub(crate) offscreen_instance_buffer: wgpu::Buffer,
    pub global_time: f32,
    pub animation_end_time: f32,
}

impl<'a> State<'a> {
    pub fn free_gpu_indices(&mut self, deleted_nodes: &[u32]) {
        for &node_id in deleted_nodes {
            if let Some(gpu_idx) = self.node_to_gpu_idx.remove(&node_id) {
                self.gpu_free_list.push(gpu_idx);
            }
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, _scale_factor: f64) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera
                .resize(&self.queue, new_size.width as f32, new_size.height as f32);
            self.depth_texture_view =
                crate::dynamic::create_depth_texture(&self.device, &self.config);
            self.text_system.resize(new_size.width, new_size.height);

            self.offscreen_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Offscreen Master Texture"),
                size: wgpu::Extent3d {
                    width: new_size.width.max(1),
                    height: new_size.height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.offscreen_view = self
                .offscreen_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.offscreen_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.offscreen_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.offscreen_sampler),
                    },
                ],
                label: Some("Offscreen Bind Group"),
            });

            self.queue.write_buffer(
                &self.offscreen_instance_buffer,
                0,
                bytemuck::cast_slice(&[InstanceRaw {
                    model_pos: [0.0, 0.0],
                    model_size: [new_size.width as f32, new_size.height as f32],
                    clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
                    style_index: 0,
                    z_index: -999.0, // Eraser identifikatori
                    padding: [0, 0],
                }]),
            );
        }
    }

    pub fn update_instances_sparse(
        &mut self,
        sparse_instances: &[(u32, Instance)],
        draw_order: &[u32],
        _total_nodes: u32,
    ) {
        for (node_id, inst) in sparse_instances {
            let gpu_idx = *self.node_to_gpu_idx.entry(*node_id).or_insert_with(|| {
                if let Some(reused) = self.gpu_free_list.pop() {
                    reused
                } else {
                    let next = self.num_instances;
                    self.num_instances += 1;
                    next
                }
            });

            let raw_inst = InstanceRaw {
                model_pos: [inst.position.x, inst.position.y],
                model_size: [inst.size.x, inst.size.y],
                clip_rect: inst.clip_rect,
                style_index: gpu_idx as u32,
                z_index: 0.0,
                padding: [0, 0],
            };

            let raw_style = StyleRaw {
                color_start: inst.color_start,
                color_end: inst.color_end,
                target_color_start: inst.target_color_start,
                target_color_end: inst.target_color_end,
                border_color: inst.border_color,
                target_border_color: inst.target_border_color,
                shadow_color: inst.shadow_color,
                shadow_data: [
                    inst.shadow_offset.x,
                    inst.shadow_offset.y,
                    inst.shadow_blur,
                    inst.shadow_spread,
                ],
                properties: [
                    inst.border_radius,
                    inst.border_width,
                    inst.gradient_angle,
                    0.0,
                ],
                anim_data: [inst.anim_start_time, inst.anim_duration, 0.0, 0.0],
            };

            let offset_inst =
                (gpu_idx as usize * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress;
            let offset_style =
                (gpu_idx as usize * std::mem::size_of::<StyleRaw>()) as wgpu::BufferAddress;

            for i in 0..3 {
                self.queue.write_buffer(
                    &self.instances_in_buffers[i],
                    offset_inst,
                    bytemuck::bytes_of(&raw_inst),
                );
                self.queue.write_buffer(
                    &self.style_buffers[i],
                    offset_style,
                    bytemuck::bytes_of(&raw_style),
                );
            }

            if inst.anim_duration > 0.0 {
                let end = inst.anim_start_time + inst.anim_duration;
                if end > self.animation_end_time {
                    self.animation_end_time = end;
                }
            }
        }

        let mut order_data = Vec::with_capacity(draw_order.len());
        for &id in draw_order {
            if let Some(&gpu_idx) = self.node_to_gpu_idx.get(&id) {
                order_data.push(gpu_idx);
            }
        }

        self.current_draw_count = order_data.len() as u32;

        if !order_data.is_empty() {
            let order_bytes = bytemuck::cast_slice(&order_data);
            for i in 0..3 {
                self.queue
                    .write_buffer(&self.draw_order_buffers[i], 0, order_bytes);
            }
        }
    }

    pub fn update_custom_draws(&mut self, draws: Vec<(String, [f32; 4], [f32; 4], Vec<u8>)>) {
        self.current_custom_draws = draws;
        let (bg, offsets) = self
            .custom_shaders
            .prepare_uniforms(&self.device, &self.current_custom_draws);
        self.custom_bind_group = bg;
        self.custom_offsets = offsets;
    }

    pub fn render(
        &mut self,
        clear_color: [f64; 4],
        scissor_rects: &[[u32; 4]],
        is_full_redraw: bool,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let current_time = self.global_time;
        self.queue.write_buffer(
            &self.time_buffer,
            0,
            bytemuck::cast_slice(&[TimeUniform {
                current_time,
                grid_width: 0.0,
                grid_height: 0.0,
                is_full_redraw: if is_full_redraw { 1.0 } else { 0.0 },
                clear_color: [
                    clear_color[0] as f32,
                    clear_color[1] as f32,
                    clear_color[2] as f32,
                    clear_color[3] as f32,
                ],
            }]),
        );

        let cull_config = [CullConfigUniform {
            total_instances: self.current_draw_count,
            grid_width: 0,
            grid_height: 0,
            is_full_redraw: if is_full_redraw { 1 } else { 0 },
        }];

        self.queue.write_buffer(
            &self.cull_config_buffers[self.frame_index],
            0,
            bytemuck::cast_slice(&cull_config),
        );

        self.text_system.prepare(&self.device, &self.queue);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Cull Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.cull_pipeline);
            cpass.set_bind_group(0, &self.cull_bind_groups[self.frame_index], &[]);
            let workgroups = ((self.current_draw_count as f32) / 64.0).ceil() as u32;
            if workgroups > 0 {
                cpass.dispatch_workgroups(workgroups, 1, 1);
            }
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Offscreen Master Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.offscreen_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if is_full_redraw {
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear_color[0],
                                g: clear_color[1],
                                b: clear_color[2],
                                a: clear_color[3],
                            })
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !is_full_redraw && !scissor_rects.is_empty() {
                for &rect in scissor_rects {
                    let sx = rect[0];
                    let sy = rect[1];

                    if sx >= self.config.width || sy >= self.config.height {
                        continue;
                    }

                    let max_w = self.config.width - sx;
                    let max_h = self.config.height - sy;

                    let sw = rect[2].min(max_w);
                    let sh = rect[3].min(max_h);

                    if sw == 0 || sh == 0 {
                        continue;
                    }

                    rpass.set_scissor_rect(sx, sy, sw, sh);

                    rpass.set_pipeline(&self.eraser_pipeline);
                    rpass.set_bind_group(0, &self.camera.bind_group, &[]);
                    rpass.set_bind_group(1, &self.time_bind_groups[self.frame_index], &[]);
                    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    rpass.set_vertex_buffer(1, self.offscreen_instance_buffer.slice(..));
                    rpass.draw(0..4, 0..1);

                    rpass.set_pipeline(&self.render_pipeline);
                    rpass.set_bind_group(0, &self.camera.bind_group, &[]);
                    rpass.set_bind_group(1, &self.style_bind_groups[self.frame_index], &[]);
                    rpass.set_bind_group(2, &self.time_bind_groups[self.frame_index], &[]);
                    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    rpass.set_vertex_buffer(
                        1,
                        self.instance_out_buffers[self.frame_index].slice(..),
                    );

                    if self.current_draw_count > 0 {
                        rpass.draw(0..4, 0..self.current_draw_count);
                    }

                    self.text_system.render(&mut rpass);

                    if let Some(bg) = &self.custom_bind_group {
                        for (i, draw) in self.current_custom_draws.iter().enumerate() {
                            if let Some(pipeline) = self.custom_shaders.pipelines.get(&draw.0) {
                                rpass.set_pipeline(pipeline);
                                rpass.set_bind_group(0, &self.camera.bind_group, &[]);

                                let offsets = self.custom_offsets[i];
                                rpass.set_bind_group(1, bg, &[offsets.0, offsets.1]);

                                rpass.draw(0..4, 0..1);
                            }
                        }
                    }
                }
            } else {
                rpass.set_pipeline(&self.render_pipeline);
                rpass.set_bind_group(0, &self.camera.bind_group, &[]);
                rpass.set_bind_group(1, &self.style_bind_groups[self.frame_index], &[]);
                rpass.set_bind_group(2, &self.time_bind_groups[self.frame_index], &[]);
                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                rpass.set_vertex_buffer(1, self.instance_out_buffers[self.frame_index].slice(..));

                if self.current_draw_count > 0 {
                    rpass.draw(0..4, 0..self.current_draw_count);
                }

                self.text_system.render(&mut rpass);

                if let Some(bg) = &self.custom_bind_group {
                    for (i, draw) in self.current_custom_draws.iter().enumerate() {
                        if let Some(pipeline) = self.custom_shaders.pipelines.get(&draw.0) {
                            rpass.set_pipeline(pipeline);
                            rpass.set_bind_group(0, &self.camera.bind_group, &[]);

                            let offsets = self.custom_offsets[i];
                            rpass.set_bind_group(1, bg, &[offsets.0, offsets.1]);

                            rpass.draw(0..4, 0..1);
                        }
                    }
                }
            }
        }

        {
            let mut c_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Composite Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            c_pass.set_pipeline(&self.composite_pipeline);
            c_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            c_pass.set_bind_group(1, &self.offscreen_bind_group, &[]);
            c_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            c_pass.set_vertex_buffer(1, self.offscreen_instance_buffer.slice(..));

            c_pass.draw(0..4, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.frame_index = (self.frame_index + 1) % 3;

        Ok(())
    }
}
