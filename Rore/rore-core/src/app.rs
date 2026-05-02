use crate::calculs::*;
use crate::state::WakeRegistry;
use crate::time::TimeManager;
use rore_render::State as RenderState;
use rore_types::text::TextRenderer;
use rore_types::RoreConfig;

use glam::Vec2;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

pub use crate::widgets::base::Widget;

#[derive(Debug)]
pub enum RoreUserEvent {
    WakeUp,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Click(String),
    Input(String, String),
    Tick(f32),
    Init,
    Resize(f32, f32),
    SelectionChanged(String, Option<(usize, usize)>),
}

pub trait App: Send + 'static {
    fn view(&self) -> Box<dyn Widget>;
    fn update(&mut self, event: AppEvent);
}

pub fn run<F>(app: impl App + 'static, config: RoreConfig, text_renderer_factory: F)
where
    F: FnOnce(&wgpu::Device, &wgpu::Queue, &wgpu::SurfaceConfiguration) -> Box<dyn TextRenderer>,
{
    env_logger::init();

    // INQILOB: Winit Custom Event bilan ochiladi
    let event_loop = EventLoopBuilder::<RoreUserEvent>::with_user_event()
        .build()
        .unwrap();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Rore UI App")
        .with_inner_size(PhysicalSize::new(1024, 768))
        .with_resizable(true)
        .with_maximized(true)
        .with_visible(true)
        .build(&event_loop)
        .unwrap();

    let window = Arc::new(window);

    let mut render_state = pollster::block_on(RenderState::new(&window, text_renderer_factory));
    let mut time_manager = TimeManager::new();
    let mut last_cursor_icon = winit::window::CursorIcon::Default;

    let wake_registry = Arc::new(Mutex::new(WakeRegistry::new()));
    {
        let proxy_clone = proxy.clone();
        wake_registry.lock().unwrap().set_waker(move || {
            let _ = proxy_clone.send_event(RoreUserEvent::WakeUp);
        });
    }

    let window_loop = window.clone();
    let (tx_logic, rx_logic): (Sender<LogicMessage>, Receiver<LogicMessage>) = mpsc::channel();
    let (tx_render, rx_render): (Sender<RenderPacket>, Receiver<RenderPacket>) = mpsc::channel();
    let (tx_recycle, rx_recycle): (
        Sender<crate::widgets::base::RenderOutput>,
        Receiver<crate::widgets::base::RenderOutput>,
    ) = mpsc::channel();

    let initial_size = render_state.size;
    let initial_scale = window.scale_factor();
    let config_clone = config.clone();
    let wake_registry_logic = wake_registry.clone();

    render_state.resize(initial_size, initial_scale);
    let _ = tx_logic.send(LogicMessage::Resize(
        initial_size.width as f32,
        initial_size.height as f32,
        initial_scale as f32,
    ));
    let _ = tx_logic.send(LogicMessage::RequestRedraw);

    // Katta mantiqiy tsikl calculs.rs ga ko'chib o'tdi!
    thread::spawn(move || {
        crate::calculs::run_logic_thread(
            app,
            rx_logic,
            tx_render,
            rx_recycle,
            config_clone,
            wake_registry_logic,
            initial_size.width as f32,
            initial_size.height as f32,
            initial_scale as f32,
        );
    });

    let mut latest_packet: Option<RenderPacket> = None;

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::UserEvent(RoreUserEvent::WakeUp) => {
                    window_loop.request_redraw();
                }
                Event::WindowEvent {
                    event: window_event,
                    window_id,
                } if window_id == window_loop.id() => match window_event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0 && physical_size.height > 0 {
                            let scale_factor = window_loop.scale_factor();
                            render_state.resize(physical_size, scale_factor);
                            let _ = tx_logic.send(LogicMessage::Resize(
                                physical_size.width as f32,
                                physical_size.height as f32,
                                scale_factor as f32,
                            ));
                            window_loop.request_redraw();
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let scale_factor = window_loop.scale_factor();
                        let logical_x = (position.x / scale_factor) as f32;
                        let logical_y = (position.y / scale_factor) as f32;
                        let _ = tx_logic.send(LogicMessage::CursorMoved(logical_x, logical_y));
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let _ = tx_logic.send(LogicMessage::MouseInput(state, button));
                    }
                    WindowEvent::KeyboardInput {
                        event: key_event, ..
                    } => {
                        let _ = tx_logic.send(LogicMessage::KeyboardInput(key_event));
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let (x_delta, y_delta) = match delta {
                            MouseScrollDelta::LineDelta(x, y) => (x * 40.0, y * 40.0),
                            MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                        };
                        let _ = tx_logic.send(LogicMessage::MouseWheel(x_delta, y_delta));
                    }
                    WindowEvent::RedrawRequested => {
                        let mut got_new_packet = false;
                        let mut combined_scissors = Vec::new();
                        let mut is_full_forced = false;

                        while let Ok(mut packet) = rx_render.try_recv() {
                            got_new_packet = true;

                            if packet.is_full_redraw_forced || packet.scissor_rects.is_empty() {
                                is_full_forced = true;
                            } else if !is_full_forced {
                                combined_scissors.extend(packet.scissor_rects.iter().copied());
                            }

                            if packet.current_cursor_icon != last_cursor_icon {
                                window_loop.set_cursor_icon(packet.current_cursor_icon);
                                last_cursor_icon = packet.current_cursor_icon;
                            }

                            // 1. O'chirilgan node'larni GPU dan tozalash!
                            render_state.free_gpu_indices(&packet.deleted_nodes);

                            let mut compiler = DisplayListCompiler::new();
                            compiler.final_insts = packet.output.sparse_instances.clone();
                            compiler.final_texts = packet.output.sparse_texts.clone();

                            for (id, cmds) in &packet.output.node_commands {
                                compiler.compile(*id, cmds);
                            }

                            for cmd in &packet.commands {
                                match cmd {
                                    RenderCommand::RegisterShader(_id, _wgsl) => {}
                                    RenderCommand::UpdateNodeCommands(id, cmds) => {
                                        compiler.compile(*id, cmds)
                                    }
                                    RenderCommand::UpdateInstance(id, inst) => {
                                        compiler.final_insts.push((*id, inst.clone()))
                                    }
                                    RenderCommand::UpdateText(_id, text) => {
                                        compiler.final_texts.push(text.clone())
                                    }
                                    RenderCommand::Remove(del_id) => {
                                        compiler.final_texts.push((
                                            *del_id,
                                            "".to_string(),
                                            rore_types::Color::TRANSPARENT,
                                            16.0,
                                            Vec2::ZERO,
                                            None,
                                            0.0,
                                        ));
                                    }
                                }
                            }

                            render_state.update_instances_sparse(
                                &compiler.final_insts,
                                &packet.draw_order,
                                packet.total_nodes,
                            );
                            render_state
                                .text_system
                                .update_sparse(&compiler.final_texts);

                            let custom_draws = std::mem::take(&mut packet.custom_draws);
                            let mut mapped_customs = Vec::new();
                            for c_draw in custom_draws {
                                if let Some(wgsl) = c_draw.wgsl_code {
                                    render_state.custom_shaders.compile(
                                        &render_state.device,
                                        &render_state.config,
                                        &render_state.camera.bind_group_layout,
                                        &c_draw.shader_id,
                                        &wgsl,
                                    );
                                }
                                mapped_customs.push((
                                    c_draw.shader_id,
                                    c_draw.rect,
                                    c_draw.clip,
                                    c_draw.uniforms,
                                ));
                            }
                            if !mapped_customs.is_empty() {
                                render_state.update_custom_draws(mapped_customs);
                            }

                            if let Some(old_packet) = latest_packet.replace(packet) {
                                let _ = tx_recycle.send(old_packet.output);
                            }
                        }

                        render_state.global_time = time_manager.elapsed;

                        if got_new_packet {
                            if let Some(_packet) = &latest_packet {
                                let final_scissors = if is_full_forced {
                                    &[]
                                } else {
                                    &combined_scissors[..]
                                };

                                match render_state.render(
                                    [0.11, 0.11, 0.18, 1.0],
                                    final_scissors,
                                    is_full_forced,
                                ) {
                                    Ok(_) => {}
                                    Err(wgpu::SurfaceError::Lost) => render_state
                                        .resize(render_state.size, window_loop.scale_factor()),
                                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                    Err(e) => eprintln!("{:?}", e),
                                }
                            }
                        } else {
                            // Tizim bo'sh yotganda ham eski holatni saqlab qolish
                            match render_state.render([0.11, 0.11, 0.18, 1.0], &[], true) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => render_state
                                    .resize(render_state.size, window_loop.scale_factor()),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    time_manager.update();
                    let _ =
                        tx_logic.send(LogicMessage::Tick(time_manager.dt, time_manager.elapsed));

                    let is_ticking = crate::reactive::signals::ACTIVE_TICKERS
                        .load(std::sync::atomic::Ordering::SeqCst)
                        > 0;

                    let has_locks = !wake_registry.lock().unwrap().is_empty();

                    let is_animating = is_ticking || has_locks;

                    if is_animating {
                        elwt.set_control_flow(ControlFlow::WaitUntil(
                            std::time::Instant::now() + std::time::Duration::from_millis(16),
                        ));
                        window_loop.request_redraw();
                    } else {
                        elwt.set_control_flow(ControlFlow::Wait);
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
