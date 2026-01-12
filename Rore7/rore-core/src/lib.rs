pub mod app;
pub mod widgets;
pub mod scroll;
pub mod state;
pub mod time;

pub use crate::app::{App, AppEvent};
pub use crate::widgets::base::Widget;
pub use crate::widgets::{View, Text, Button, TextInput}; 

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    dpi::PhysicalSize,
    keyboard::{Key, NamedKey},
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use glam::Vec2;
use std::fs; 

use rore_render::{State, text::FontManager};
use rore_layout::LayoutEngine;

use crate::state::FrameworkState;
use crate::widgets::base::{BuildContext, RenderOutput, TextureSource};
use crate::time::TimeManager;
use rore_types::RoreConfig;

pub async fn run(mut app: impl App + 'static, config: RoreConfig) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    
    let window = WindowBuilder::new()
        .with_title("Rore UI App")
        .with_inner_size(PhysicalSize::new(1024, 768))
        .with_resizable(true)
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    let window = Arc::new(window);
    let font_manager = Arc::new(Mutex::new(FontManager::new()));
    
    let mut render_state = State::new(&window, font_manager.clone()).await;
    let mut layout_engine = LayoutEngine::new();
    
    let mut fw_state = FrameworkState::new(config);
    let mut time_manager = TimeManager::new();
    
    let build_ctx = BuildContext {
        font_manager: font_manager.clone(),
    };

    let window_loop = window.clone();
    app.update(AppEvent::Init);

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event: window_event, window_id } if window_id == window_loop.id() => {
                match window_event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    
                    WindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0 && physical_size.height > 0 {
                            render_state.resize(physical_size);
                            window_loop.request_redraw();
                        }
                    },
                    
                    WindowEvent::ScaleFactorChanged { inner_size_writer: _, .. } => {
                         let new_inner_size = window_loop.inner_size();
                         if new_inner_size.width > 0 && new_inner_size.height > 0 {
                            render_state.resize(new_inner_size);
                            window_loop.request_redraw();
                         }
                    },
                    
                    WindowEvent::CursorMoved { position, .. } => {
                        if fw_state.config.mouse_support {
                            fw_state.update_cursor(position.x as f32, position.y as f32);
                            window_loop.request_redraw(); 
                        }
                    }
                    
                WindowEvent::MouseInput { state, button, .. } => {
                        // Sichqoncha yoki Touch qo'llab-quvvatlansa
                        if fw_state.config.mouse_support || fw_state.config.touch_support {
                            match state {
                                ElementState::Pressed => {
                                    if button == MouseButton::Left {
                                        // 1. Aktiv va Fokus elementni belgilash
                                        fw_state.active_node = fw_state.hovered_node;
                                        fw_state.focused_node = fw_state.hovered_node;
                                        
                                        // 2. Input fokusini boshqarish
                                        // Agar foydalanuvchi bo'sh joyga bossa, input fokusi yo'qolishi kerak
                                        if fw_state.hovered_node.is_none() {
                                            fw_state.focused_input_id = None;
                                            fw_state.input_selection = None; // Selection ham o'chadi
                                        }
                                        
                                        window_loop.request_redraw();
                                    }
                                }
                                ElementState::Released => {
                                    if button == MouseButton::Left {
                                        // --- YANGI: DRAG TUGADI ---
                                        // Sichqoncha qo'yib yuborildi, demak selection tugadi
                                        fw_state.drag_start_idx = None;

                                        // Click hodisasini yuborish
                                        if fw_state.active_node.is_some() {
                                            // Faqat sichqoncha bosilgan element ustida qo'yib yuborilsa (Click)
                                            if let Some(hit_node) = fw_state.hovered_node {
                                                if let Some(id) = layout_engine.node_to_id.get(&hit_node) {
                                                    app.update(AppEvent::Click(id.clone()));
                                                }
                                            }
                                        }
                                        
                                        // Aktiv holatni o'chirish
                                        fw_state.active_node = None;
                                        window_loop.request_redraw();
                                    }
                                }
                            }
                        }
                    }
                    
                    // --- YANGI: Klaviatura va Kursor Logikasi ---
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        if key_event.state == ElementState::Pressed {
                            // Agar biror input fokusda bo'lsa
                            if let Some(focused_id) = &fw_state.focused_input_id {
                                
                                // 1. Matn yozish
                                if let Some(text) = &key_event.text {
                                    if !text.chars().any(|c| c.is_control()) {
                                        app.update(AppEvent::Input(focused_id.clone(), text.to_string()));
                                        // Harf yozganda kursorni oldinga suramiz
                                        fw_state.input_cursor_idx += 1;
                                        window_loop.request_redraw();
                                    }
                                }

                                // 2. Maxsus tugmalar
                                match key_event.logical_key {
                                    Key::Named(NamedKey::Backspace) => {
                                        // Backspace faqat kursor boshida bo'lmasa ishlaydi
                                        if fw_state.input_cursor_idx > 0 {
                                            app.update(AppEvent::Input(focused_id.clone(), "\u{08}".to_string()));
                                            fw_state.input_cursor_idx = fw_state.input_cursor_idx.saturating_sub(1);
                                            window_loop.request_redraw();
                                        }
                                    },
                                    Key::Named(NamedKey::Enter) => {
                                        app.update(AppEvent::Input(focused_id.clone(), "\n".to_string()));
                                        window_loop.request_redraw();
                                    },
                                    // Kursor harakati (Navigation)
                                    Key::Named(NamedKey::ArrowLeft) => {
                                        fw_state.input_cursor_idx = fw_state.input_cursor_idx.saturating_sub(1);
                                        // Animatsiyani "reset" qilish (kursor yonib tursin)
                                        // Kelajakda: fw_state.reset_cursor_blink();
                                        window_loop.request_redraw();
                                    },
                                    Key::Named(NamedKey::ArrowRight) => {
                                        fw_state.input_cursor_idx += 1;
                                        window_loop.request_redraw();
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                    
                    WindowEvent::MouseWheel { delta, .. } => {
                        let y_delta = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y * 40.0,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                        };
                        fw_state.handle_scroll(y_delta, "main_scroll");
                        window_loop.request_redraw();
                    }

                    WindowEvent::RedrawRequested => {
                        time_manager.update();
                        app.update(AppEvent::Tick(time_manager.dt));

                        layout_engine.clear();
                        let root_widget = app.view();
                        let root_node = root_widget.build(&mut layout_engine, &build_ctx);
                        layout_engine.root = Some(root_node);
                        
                        let size = render_state.size;
                        layout_engine.compute(size.width as f32, size.height as f32);

                        if !fw_state.scroll_offsets.contains_key("main_scroll") {
                            fw_state.scroll_offsets.insert("main_scroll".to_string(), 0.0);
                        }

                        let mut corrected_offsets = HashMap::new();
                        for (id, raw_offset) in &fw_state.scroll_offsets {
                            if let Some(node) = layout_engine.get_node(id) {
                                if let Ok(viewport_layout) = layout_engine.taffy.layout(node) {
                                    let viewport_h = viewport_layout.size.height;
                                    if let Ok(children) = layout_engine.taffy.children(node) {
                                        if let Some(child) = children.first() {
                                            if let Ok(content_layout) = layout_engine.taffy.layout(*child) {
                                                let content_h = content_layout.size.height;
                                                let max_scroll = (content_h - viewport_h).max(0.0);
                                                let clamped = raw_offset.clamp(0.0, max_scroll);
                                                corrected_offsets.insert(id.clone(), clamped);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        fw_state.scroll_offsets.extend(corrected_offsets);

                        for (id_str, _) in &fw_state.scroll_offsets {
                            if let Some(scroll_node) = layout_engine.get_node(id_str) {
                                if let Ok(children) = layout_engine.taffy.children(scroll_node) {
                                    if let Some(child) = children.first() {
                                        layout_engine.reset_node_y_to_zero(*child);
                                    }
                                }
                            }
                        }

                        let mut node_scroll_offsets = HashMap::new();
                        for (id_str, offset) in &fw_state.scroll_offsets {
                            if let Some(node) = layout_engine.get_node(id_str) {
                                node_scroll_offsets.insert(node, *offset);
                            }
                        }

                        // Hit Test
                        fw_state.hovered_node = layout_engine.hit_test(
                            root_node,
                            fw_state.cursor_pos.x,
                            fw_state.cursor_pos.y,
                            &node_scroll_offsets
                        );

                        let render_output: RenderOutput = root_widget.render(
                            &layout_engine,
                            &mut fw_state, 
                            root_node,
                            Vec2::ZERO,
                            &build_ctx.font_manager,
                            None,
                            "root".to_string()
                        );

                        render_state.update_instances(&render_output.instances);
                        
                        for (text, color, size, pos, clip, width) in render_output.texts {
                            render_state.text_system.queue_text(
                                &text, color, size, pos.x, pos.y, clip, width
                            );
                        }

                        for (id, source) in render_output.texture_loads {
                            if !render_state.textures.contains_key(&id) {
                                match source {
                                    TextureSource::Path(path) => {
                                        match fs::read(&path) {
                                            Ok(bytes) => { render_state.load_texture(&id, &bytes); }
                                            Err(e) => eprintln!("Error loading texture '{}': {:?}", path, e),
                                        }
                                    },
                                    TextureSource::Bytes(bytes) => {
                                        render_state.load_texture(&id, &bytes);
                                    }
                                }
                            }
                        }

                        for (id, instances) in render_output.images {
                            for instance in instances {
                                render_state.queue_image(&id, instance);
                            }
                        }

                        match render_state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                            Err(wgpu::SurfaceError::Timeout) => {}, 
                            Err(e) => eprintln!("{:?}", e),
                        }

                        if fw_state.is_animating() {
                            window_loop.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();
}