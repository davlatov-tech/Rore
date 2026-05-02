use crate::app::{App, AppEvent};
use crate::state::FrameworkState;
use crate::time::TimeManager;
use crate::widgets::base::{BuildContext, EventResult, RenderOutput, WidgetEvent};
use glam::Vec2;
use rore_layout::LayoutEngine;
use rore_types::RoreConfig;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use winit::event::KeyEvent;
use winit::event::{ElementState, MouseButton};

#[derive(Clone)]
pub struct CustomShaderDraw {
    pub shader_id: String,
    pub wgsl_code: Option<String>,
    pub rect: [f32; 4],
    pub clip: [f32; 4],
    pub uniforms: Vec<u8>,
}

pub enum RenderCommand {
    UpdateNodeCommands(u32, Vec<crate::widgets::base::DisplayCommand>),
    UpdateInstance(u32, rore_render::Instance),
    UpdateText(u32, rore_types::text::SparseTextItem),
    RegisterShader(String, String),
    Remove(u32),
}

pub struct RenderPacket {
    pub output: RenderOutput,
    pub commands: Vec<RenderCommand>,
    pub current_cursor_icon: winit::window::CursorIcon,
    pub is_animating: bool,
    pub total_nodes: u32,
    pub scissor_rects: Vec<[u32; 4]>,
    pub is_full_redraw_forced: bool,
    pub deleted_nodes: Vec<u32>,
    pub draw_order: Vec<u32>,
    pub custom_draws: Vec<CustomShaderDraw>,
}

pub enum LogicMessage {
    Resize(f32, f32, f32),
    CursorMoved(f32, f32),
    MouseInput(ElementState, MouseButton),
    KeyboardInput(KeyEvent),
    MouseWheel(f32, f32), // Endi x_delta va y_delta
    Tick(f32, f32),
    RequestRedraw,
    RegisterShader(String, String),
}

pub struct DisplayListCompiler {
    pub clip_stack: Vec<[f32; 4]>,
    pub transform_stack: Vec<Vec2>,
    pub final_insts: Vec<(u32, rore_render::Instance)>,
    pub final_texts: Vec<rore_types::text::SparseTextItem>,
    pub final_custom: Vec<CustomShaderDraw>,
}

impl DisplayListCompiler {
    pub fn new() -> Self {
        Self {
            clip_stack: vec![[-10000.0, -10000.0, 20000.0, 20000.0]],
            transform_stack: vec![Vec2::ZERO],
            final_insts: Vec::new(),
            final_texts: Vec::new(),
            final_custom: Vec::new(),
        }
    }

    pub fn compile(&mut self, id: u32, cmds: &[crate::widgets::base::DisplayCommand]) {
        for cmd in cmds {
            match cmd {
                crate::widgets::base::DisplayCommand::PushClip { rect } => {
                    let current = self.clip_stack.last().unwrap();
                    let min_x = current[0].max(rect[0]);
                    let min_y = current[1].max(rect[1]);
                    let max_x = (current[0] + current[2]).min(rect[0] + rect[2]);
                    let max_y = (current[1] + current[3]).min(rect[1] + rect[3]);
                    self.clip_stack.push([
                        min_x,
                        min_y,
                        (max_x - min_x).max(0.0),
                        (max_y - min_y).max(0.0),
                    ]);
                }
                crate::widgets::base::DisplayCommand::PopClip => {
                    if self.clip_stack.len() > 1 {
                        self.clip_stack.pop();
                    }
                }
                crate::widgets::base::DisplayCommand::PushTransform { offset } => {
                    let current = self.transform_stack.last().unwrap();
                    self.transform_stack.push(*current + *offset);
                }
                crate::widgets::base::DisplayCommand::PopTransform => {
                    if self.transform_stack.len() > 1 {
                        self.transform_stack.pop();
                    }
                }
                crate::widgets::base::DisplayCommand::DrawQuad {
                    rect,
                    color,
                    border_radius,
                    border_width,
                    border_color,
                    anim_start_time,
                    anim_duration,
                } => {
                    let current_clip = *self.clip_stack.last().unwrap();
                    let current_transform = *self.transform_stack.last().unwrap();
                    self.final_insts.push((
                        id,
                        rore_render::Instance {
                            position: Vec2::new(
                                rect[0] + current_transform.x,
                                rect[1] + current_transform.y,
                            ),
                            size: Vec2::new(rect[2], rect[3]),
                            color_start: *color,
                            color_end: *color,
                            target_color_start: *color,
                            target_color_end: *color,
                            gradient_angle: 0.0,
                            border_radius: *border_radius,
                            border_width: *border_width,
                            border_color: *border_color,
                            target_border_color: *border_color,
                            shadow_color: [0.0; 4],
                            shadow_offset: Vec2::ZERO,
                            shadow_blur: 0.0,
                            shadow_spread: 0.0,
                            clip_rect: current_clip,
                            anim_start_time: *anim_start_time,
                            anim_duration: *anim_duration,
                        },
                    ));
                }
                crate::widgets::base::DisplayCommand::DrawText {
                    text,
                    pos,
                    font_size,
                    color,
                    clip,
                    width_limit,
                } => {
                    let current_clip = clip.unwrap_or(*self.clip_stack.last().unwrap());
                    let current_transform = *self.transform_stack.last().unwrap();
                    self.final_texts.push((
                        id,
                        text.clone(),
                        rore_types::Color::new(color[0], color[1], color[2], color[3]),
                        *font_size,
                        *pos + current_transform,
                        Some(current_clip),
                        *width_limit,
                    ));
                }
                crate::widgets::base::DisplayCommand::DrawCustomShader {
                    shader_id,
                    wgsl_code,
                    rect,
                    uniforms,
                } => {
                    let current_clip = *self.clip_stack.last().unwrap();
                    let current_transform = *self.transform_stack.last().unwrap();
                    self.final_custom.push(CustomShaderDraw {
                        shader_id: shader_id.clone(),
                        wgsl_code: wgsl_code.clone(),
                        rect: [
                            rect[0] + current_transform.x,
                            rect[1] + current_transform.y,
                            rect[2],
                            rect[3],
                        ],
                        clip: current_clip,
                        uniforms: uniforms.clone(),
                    });
                }
            }
        }
    }
}

pub fn run_logic_thread<A: App + 'static>(
    mut app: A,
    rx_logic: Receiver<LogicMessage>,
    tx_render: Sender<RenderPacket>,
    rx_recycle: Receiver<RenderOutput>,
    config_clone: RoreConfig,
    wake_registry_logic: Arc<Mutex<crate::state::WakeRegistry>>,
    initial_width: f32,
    initial_height: f32,
    initial_scale: f32,
) {
    let mut fw_state = FrameworkState::new(config_clone.clone(), wake_registry_logic.clone());
    let mut layout_engine = LayoutEngine::new();
    let build_ctx = BuildContext {};
    let mut logic_time_manager = TimeManager::new();

    let mut previous_active_nodes: std::collections::HashSet<u32> =
        std::collections::HashSet::new();
    let mut previous_visual_bounds: HashMap<u32, [u32; 4]> = HashMap::new();

    app.update(AppEvent::Init);
    layout_engine.clear();
    fw_state.arena.clear();

    let (_, root_node_id) = crate::reactive::signals::create_scope(|| {
        let root_widget = app.view();
        root_widget.build(&mut fw_state.arena, &mut layout_engine, &build_ctx)
    });

    let root_taffy_node = *fw_state.arena.taffy_map.get(&root_node_id).unwrap();
    layout_engine.root = Some(root_taffy_node);

    let mut current_width = initial_width;
    let mut current_height = initial_height;
    let mut current_scale = initial_scale;

    layout_engine.compute(
        current_width / current_scale,
        current_height / current_scale,
    );
    fw_state.update_aabbs(&layout_engine, root_taffy_node, true);

    loop {
        let Ok(first_msg) = rx_logic.recv() else {
            break;
        };

        let mut needs_compute = false;
        let mut msgs = vec![first_msg];
        while let Ok(m) = rx_logic.try_recv() {
            msgs.push(m);
        }

        let mut batched_msgs = Vec::with_capacity(msgs.len());
        let mut last_cursor_idx = None;
        let mut last_resize_idx = None;

        for msg in msgs {
            match msg {
                LogicMessage::CursorMoved(_, _) => {
                    if let Some(idx) = last_cursor_idx {
                        batched_msgs[idx] = msg;
                    } else {
                        last_cursor_idx = Some(batched_msgs.len());
                        batched_msgs.push(msg);
                    }
                }
                LogicMessage::Resize(_, _, _) => {
                    if let Some(idx) = last_resize_idx {
                        batched_msgs[idx] = msg;
                    } else {
                        last_resize_idx = Some(batched_msgs.len());
                        batched_msgs.push(msg);
                    }
                }
                _ => batched_msgs.push(msg),
            }
        }

        let mut commands = Vec::new();

        for msg in batched_msgs {
            match msg {
                LogicMessage::RegisterShader(id, wgsl) => {
                    commands.push(RenderCommand::RegisterShader(id, wgsl));
                }
                LogicMessage::Resize(w, h, scale) => {
                    current_width = w;
                    current_height = h;
                    current_scale = scale;
                    needs_compute = true;
                    fw_state.full_redraw = true;
                    app.update(AppEvent::Resize(w / scale, h / scale));
                }
                LogicMessage::CursorMoved(x, y) => {
                    if fw_state.config.mouse_support {
                        fw_state.update_cursor(x, y);

                        // =========================================================================
                        // INQILOB: Capturing marshruti. Agar kursor bosilgan ushlab turilsa, yordam beradi.
                        // =========================================================================
                        if let Some(active) = fw_state.active_node {
                            let mut dx = 0.0;
                            let mut dy = 0.0;
                            if let Some(last) = fw_state.last_cursor_pos {
                                dx = x - last.x;
                                dy = y - last.y;
                            }

                            if dx != 0.0 || dy != 0.0 {
                                if let Some(&node_id) = fw_state.arena.node_map.get(&active) {
                                    if let Some(mut widget) =
                                        fw_state.arena.widgets[node_id.0 as usize].take()
                                    {
                                        widget.handle_event(
                                            &mut fw_state,
                                            &WidgetEvent::MouseDrag { dx, dy },
                                        );
                                        fw_state.arena.widgets[node_id.0 as usize] = Some(widget);
                                    }
                                }
                            }
                        }

                        let new_hover = fw_state.hit_test(x, y);

                        if new_hover != fw_state.hovered_node {
                            if let Some(old_node) = fw_state.hovered_node {
                                let bubble_chain = fw_state.get_event_bubble_chain(old_node);
                                for node in bubble_chain {
                                    if let Some(&node_id) = fw_state.arena.node_map.get(&node) {
                                        if let Some(mut widget) =
                                            fw_state.arena.widgets[node_id.0 as usize].take()
                                        {
                                            let res = widget.handle_event(
                                                &mut fw_state,
                                                &WidgetEvent::HoverLeave,
                                            );
                                            fw_state.arena.widgets[node_id.0 as usize] =
                                                Some(widget);
                                            if res == EventResult::Consumed {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(new_node) = new_hover {
                                let bubble_chain = fw_state.get_event_bubble_chain(new_node);
                                for node in bubble_chain {
                                    if let Some(&node_id) = fw_state.arena.node_map.get(&node) {
                                        if let Some(mut widget) =
                                            fw_state.arena.widgets[node_id.0 as usize].take()
                                        {
                                            let res = widget.handle_event(
                                                &mut fw_state,
                                                &WidgetEvent::HoverEnter,
                                            );
                                            fw_state.arena.widgets[node_id.0 as usize] =
                                                Some(widget);
                                            if res == EventResult::Consumed {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            fw_state.hovered_node = new_hover;
                        }

                        // =========================================================================
                        // Yangi mantiq: MouseMove ham yetkaziladi
                        // =========================================================================
                        if let Some(hover) = fw_state.hovered_node {
                            let bubble_chain = fw_state.get_event_bubble_chain(hover);
                            for node in bubble_chain {
                                if let Some(&node_id) = fw_state.arena.node_map.get(&node) {
                                    if let Some(mut widget) =
                                        fw_state.arena.widgets[node_id.0 as usize].take()
                                    {
                                        let res = widget.handle_event(
                                            &mut fw_state,
                                            &WidgetEvent::MouseMove { x, y },
                                        );
                                        fw_state.arena.widgets[node_id.0 as usize] = Some(widget);
                                        if res == EventResult::Consumed {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                LogicMessage::MouseInput(state, button) => {
                    if fw_state.config.mouse_support || fw_state.config.touch_support {
                        match state {
                            ElementState::Pressed => {
                                if button == MouseButton::Left {
                                    if let Some(hit_node) = fw_state.hovered_node {
                                        let bubble_chain =
                                            fw_state.get_event_bubble_chain(hit_node);
                                        let mut consumed_node = None;
                                        for node in bubble_chain {
                                            let mut consumed = false;
                                            if let Some(&node_id) =
                                                fw_state.arena.node_map.get(&node)
                                            {
                                                if let Some(mut widget) = fw_state.arena.widgets
                                                    [node_id.0 as usize]
                                                    .take()
                                                {
                                                    let result = widget.handle_event(
                                                        &mut fw_state,
                                                        &WidgetEvent::MouseDown,
                                                    );
                                                    fw_state.arena.widgets[node_id.0 as usize] =
                                                        Some(widget);
                                                    if matches!(result, EventResult::Consumed) {
                                                        consumed = true;
                                                    }
                                                }
                                            }
                                            if consumed {
                                                consumed_node = Some(node);
                                                break;
                                            }
                                        }
                                        // =========================================================================
                                        // INQILOB: Active Node ni kursor holati bilan belgilaymiz!
                                        // =========================================================================
                                        fw_state.active_node =
                                            consumed_node.or(fw_state.hovered_node);
                                        fw_state.focused_node =
                                            consumed_node.or(fw_state.hovered_node);
                                    } else {
                                        fw_state.active_node = None;
                                        fw_state.focused_node = None;
                                    }
                                }
                            }
                            ElementState::Released => {
                                if button == MouseButton::Left {
                                    if fw_state.active_node.is_some() {
                                        if let Some(hit_node) = fw_state.hovered_node {
                                            let bubble_chain =
                                                fw_state.get_event_bubble_chain(hit_node);
                                            for node in bubble_chain {
                                                let mut consumed = false;
                                                if let Some(&node_id) =
                                                    fw_state.arena.node_map.get(&node)
                                                {
                                                    if let Some(mut widget) = fw_state.arena.widgets
                                                        [node_id.0 as usize]
                                                        .take()
                                                    {
                                                        let result = widget.handle_event(
                                                            &mut fw_state,
                                                            &WidgetEvent::Click,
                                                        );
                                                        fw_state.arena.widgets
                                                            [node_id.0 as usize] = Some(widget);
                                                        if matches!(result, EventResult::Consumed) {
                                                            consumed = true;
                                                        }
                                                    }
                                                }
                                                if consumed {
                                                    if let Some(&node_id) =
                                                        fw_state.arena.node_map.get(&node)
                                                    {
                                                        if let Some(id_str) = fw_state
                                                            .arena
                                                            .node_to_id_str
                                                            .get(&node_id)
                                                        {
                                                            app.update(AppEvent::Click(
                                                                id_str.clone(),
                                                            ));
                                                        }
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    fw_state.active_node = None;
                                }
                            }
                        }
                    }
                }
                LogicMessage::KeyboardInput(key_event) => {
                    if key_event.state == ElementState::Pressed {
                        if let Some(focused_node) = fw_state.focused_node {
                            let bubble_chain = fw_state.get_event_bubble_chain(focused_node);
                            for node in bubble_chain {
                                let mut consumed = false;
                                if let Some(&node_id) = fw_state.arena.node_map.get(&node) {
                                    if let Some(mut widget) =
                                        fw_state.arena.widgets[node_id.0 as usize].take()
                                    {
                                        if let Some(text) = &key_event.text {
                                            if !text.as_str().chars().any(|c: char| c.is_control())
                                            {
                                                let res = widget.handle_event(
                                                    &mut fw_state,
                                                    &WidgetEvent::TextInput(text.to_string()),
                                                );
                                                if res == EventResult::Consumed {
                                                    consumed = true;
                                                }
                                            }
                                        }
                                        let res = widget.handle_event(
                                            &mut fw_state,
                                            &WidgetEvent::KeyPress(key_event.logical_key.clone()),
                                        );
                                        if res == EventResult::Consumed {
                                            consumed = true;
                                        }
                                        fw_state.arena.widgets[node_id.0 as usize] = Some(widget);
                                    }
                                }
                                if consumed {
                                    break;
                                }
                            }
                        }
                    }
                }
                // =========================================================================
                // INQILOB: G'ildirakcha aylanganda Bubbling marshruti (Zanjirli scroll)
                // =========================================================================
                LogicMessage::MouseWheel(delta_x, delta_y) => {
                    if let Some(hit_node) = fw_state.hovered_node {
                        let bubble_chain = fw_state.get_event_bubble_chain(hit_node);
                        for node in bubble_chain {
                            let mut consumed = false;
                            if let Some(&node_id) = fw_state.arena.node_map.get(&node) {
                                if let Some(mut widget) =
                                    fw_state.arena.widgets[node_id.0 as usize].take()
                                {
                                    let result = widget.handle_event(
                                        &mut fw_state,
                                        &WidgetEvent::MouseScroll { delta_x, delta_y },
                                    );
                                    fw_state.arena.widgets[node_id.0 as usize] = Some(widget);
                                    if matches!(result, EventResult::Consumed) {
                                        consumed = true;
                                    }
                                }
                            }
                            if consumed {
                                break;
                            }
                        }
                    }
                }
                LogicMessage::Tick(dt, gpu_time) => {
                    logic_time_manager.add_accum(dt);
                    fw_state.global_time = gpu_time;

                    crate::reactive::signals::tick_all(dt);
                    let is_animating = crate::reactive::context::tick_tweens(dt);
                    let is_loading = crate::reactive::resource::ACTIVE_RESOURCES
                        .load(std::sync::atomic::Ordering::SeqCst)
                        > 0;

                    if is_animating || is_loading {
                        fw_state.request_redraw();
                    }

                    if let Some(focused) = fw_state.focused_node {
                        if let Some(&node_id) = fw_state.arena.node_map.get(&focused) {
                            if let Some(w) = fw_state.arena.widgets[node_id.0 as usize].as_ref() {
                                if w.type_name() == "TextInput" {
                                    if !fw_state.sparse_update_queue.contains(&node_id) {
                                        fw_state.sparse_update_queue.push(node_id);
                                    }
                                    fw_state.request_redraw();
                                }
                            }
                        }
                    }
                    while logic_time_manager.consume_fixed_step() {
                        app.update(AppEvent::Tick(logic_time_manager.fixed_dt));
                    }
                }
                LogicMessage::RequestRedraw => {
                    fw_state.request_redraw();
                }
            }
        }

        crate::reactive::signals::process_pending_effects();
        fw_state.process_commands(&mut layout_engine);

        let rebuilds = std::mem::take(&mut fw_state.pending_rebuilds);
        let mut tree_changed = false;
        for (node_id, action) in rebuilds {
            if let Some(mut widget) = fw_state.arena.widgets[node_id.0 as usize].take() {
                widget.rebuild(&mut fw_state, &mut layout_engine, action);
                fw_state.arena.widgets[node_id.0 as usize] = Some(widget);
                tree_changed = true;
            }
        }
        if tree_changed {
            fw_state.process_drop_queue(&layout_engine);
            needs_compute = true;
        }

        let do_full_redraw = fw_state.full_redraw;
        let do_partial_redraw =
            !fw_state.sparse_update_queue.is_empty() || fw_state.dirty_rect.is_some();

        if do_full_redraw || do_partial_redraw {
            for &node_id in &fw_state.pending_dirty_nodes {
                let idx = node_id.0 as usize;
                let flags = fw_state.arena.dirty_flags[idx];
                if (flags & crate::state::DIRTY_LAYOUT) != 0
                    || (flags & crate::state::DIRTY_ALL) != 0
                {
                    if let Some(taffy_node) = fw_state.arena.taffy_map.get(&node_id) {
                        let _ = layout_engine.taffy.dirty(*taffy_node);
                        needs_compute = true;
                    }
                }
            }
            fw_state.pending_dirty_nodes.clear();

            if needs_compute {
                layout_engine.compute(
                    current_width / current_scale,
                    current_height / current_scale,
                );
                fw_state.update_aabbs(&layout_engine, root_taffy_node, fw_state.needs_aabb_update);
                fw_state.needs_aabb_update = false;
            } else if fw_state.needs_aabb_update {
                fw_state.update_aabbs(&layout_engine, root_taffy_node, true);
                fw_state.needs_aabb_update = false;
            }

            fw_state.current_cursor_icon = winit::window::CursorIcon::Default;
            let mut render_output = rx_recycle
                .try_recv()
                .unwrap_or_else(|_| RenderOutput::new());

            render_output.sparse_instances.clear();
            render_output.sparse_texts.clear();
            render_output.texture_loads.clear();
            render_output.node_commands.clear();
            for list in render_output.images.values_mut() {
                list.clear();
            }

            let mut nodes_to_update: Vec<_> = fw_state.sparse_update_queue.drain(..).collect();
            let mut dirty_rects: Vec<[u32; 4]> = Vec::new();

            if do_full_redraw {
                fw_state.full_redraw = false;
                previous_visual_bounds.clear();
                for (id, node) in &fw_state.arena.taffy_map {
                    let bounds = fw_state.node_bounds.get(node).copied().unwrap_or([0.0; 4]);
                    let mut overflow = [0.0, 0.0, 0.0, 0.0];
                    if let Some(widget) = fw_state.arena.widgets[id.0 as usize].as_ref() {
                        overflow = widget.visual_overflow();
                    }

                    // INQILOB: Haqiqiy geometrik qirqish matematikasi
                    let sf = current_scale;
                    let v_x = bounds[0] - overflow[3];
                    let v_y = bounds[1] - overflow[0];
                    let v_w = bounds[2] + overflow[3] + overflow[1];
                    let v_h = bounds[3] + overflow[0] + overflow[2];

                    let raw_x = v_x * sf;
                    let raw_y = v_y * sf;
                    let raw_w = v_w * sf;
                    let raw_h = v_h * sf;

                    let x1 = raw_x.floor() as i32;
                    let y1 = raw_y.floor() as i32;
                    let x2 = (raw_x + raw_w).ceil() as i32 + 2;
                    let y2 = (raw_y + raw_h).ceil() as i32 + 2;

                    let safe_x = x1.max(0) as u32;
                    let safe_y = y1.max(0) as u32;
                    let safe_w = (x2 - x1.max(0)).max(0) as u32;
                    let safe_h = (y2 - y1.max(0)).max(0) as u32;

                    previous_visual_bounds.insert(id.0, [safe_x, safe_y, safe_w, safe_h]);
                }

                fw_state.is_overlay_pass = false;
                if let Some(root_widget_ref) =
                    fw_state.arena.widgets[root_node_id.0 as usize].take()
                {
                    let new_output = root_widget_ref.render(
                        &layout_engine,
                        &mut fw_state,
                        root_taffy_node,
                        Vec2::ZERO,
                        None,
                        "root".to_string(),
                    );
                    render_output.extend(new_output);
                    fw_state.arena.widgets[root_node_id.0 as usize] = Some(root_widget_ref);
                }

                fw_state.is_overlay_pass = true;
                let overlays = fw_state.arena.overlays.clone();
                for (i, &overlay_node) in overlays.iter().enumerate() {
                    let mut start_pos = Vec2::ZERO;
                    if let Some(target_id) = fw_state.arena.anchors.get(&overlay_node) {
                        if let Some(target_node) = fw_state.arena.dynamic_nodes.get(target_id) {
                            if let Some(t_node) = fw_state.arena.taffy_map.get(target_node) {
                                if let Some(bounds) = fw_state.node_bounds.get(t_node).copied() {
                                    start_pos = Vec2::new(bounds[0], bounds[1] + bounds[3]);
                                }
                            }
                        }
                    }
                    if let Some(&overlay_id) = fw_state.arena.node_map.get(&overlay_node) {
                        if let Some(overlay_widget_ref) =
                            fw_state.arena.widgets[overlay_id.0 as usize].take()
                        {
                            let new_output = overlay_widget_ref.render(
                                &layout_engine,
                                &mut fw_state,
                                overlay_node,
                                start_pos,
                                None,
                                format!("overlay_{}", i),
                            );
                            render_output.extend(new_output);
                            fw_state.arena.widgets[overlay_id.0 as usize] =
                                Some(overlay_widget_ref);
                        }
                    }
                }
                fw_state.is_overlay_pass = false;

                fw_state.current_draw_order = render_output
                    .sparse_instances
                    .iter()
                    .map(|(id, _)| *id)
                    .chain(render_output.node_commands.iter().map(|(id, _)| *id))
                    .collect();

                // =========================================================================
                // INQILOB: O'ta tezkor (O(1)) qidiruv uchun Xesh-Jadvalni to'ldiramiz
                // =========================================================================
                fw_state.draw_order_set = fw_state.current_draw_order.iter().copied().collect();
            } else if do_partial_redraw {
                let mut shifted_nodes = Vec::new();
                for (id, taffy_node) in &fw_state.arena.taffy_map {
                    let old_rect_opt = previous_visual_bounds.get(&id.0).copied();
                    let bounds = fw_state
                        .node_bounds
                        .get(taffy_node)
                        .copied()
                        .unwrap_or([0.0; 4]);
                    let mut overflow = [0.0, 0.0, 0.0, 0.0];
                    if let Some(widget) = fw_state.arena.widgets[id.0 as usize].as_ref() {
                        overflow = widget.visual_overflow();
                    }

                    // INQILOB: Matematik qirqish - 2
                    let sf = current_scale;
                    let v_x = bounds[0] - overflow[3];
                    let v_y = bounds[1] - overflow[0];
                    let v_w = bounds[2] + overflow[3] + overflow[1];
                    let v_h = bounds[3] + overflow[0] + overflow[2];

                    let raw_x = v_x * sf;
                    let raw_y = v_y * sf;
                    let raw_w = v_w * sf;
                    let raw_h = v_h * sf;

                    let x1 = raw_x.floor() as i32;
                    let y1 = raw_y.floor() as i32;
                    let x2 = (raw_x + raw_w).ceil() as i32 + 2;
                    let y2 = (raw_y + raw_h).ceil() as i32 + 2;

                    let safe_x = x1.max(0) as u32;
                    let safe_y = y1.max(0) as u32;
                    let safe_w = (x2 - x1.max(0)).max(0) as u32;
                    let safe_h = (y2 - y1.max(0)).max(0) as u32;

                    let new_rect = [safe_x, safe_y, safe_w, safe_h];

                    let has_shifted = match old_rect_opt {
                        Some(old_rect) => old_rect != new_rect,
                        None => true,
                    };
                    if has_shifted && !nodes_to_update.contains(id) {
                        shifted_nodes.push(*id);
                    }
                }

                nodes_to_update.extend(shifted_nodes);

                for node_id in &nodes_to_update {
                    let old_rect = previous_visual_bounds.get(&node_id.0).copied();
                    let mut new_rect = None;

                    if let Some(taffy_node) = fw_state.arena.taffy_map.get(node_id).copied() {
                        let bounds = fw_state
                            .node_bounds
                            .get(&taffy_node)
                            .copied()
                            .unwrap_or([0.0; 4]);
                        let mut overflow = [0.0, 0.0, 0.0, 0.0];
                        if let Some(widget) = fw_state.arena.widgets[node_id.0 as usize].as_ref() {
                            overflow = widget.visual_overflow();
                        }

                        // INQILOB: Matematik qirqish - 3
                        let sf = current_scale;
                        let v_x = bounds[0] - overflow[3];
                        let v_y = bounds[1] - overflow[0];
                        let v_w = bounds[2] + overflow[3] + overflow[1];
                        let v_h = bounds[3] + overflow[0] + overflow[2];

                        let raw_x = v_x * sf;
                        let raw_y = v_y * sf;
                        let raw_w = v_w * sf;
                        let raw_h = v_h * sf;

                        let x1 = raw_x.floor() as i32;
                        let y1 = raw_y.floor() as i32;
                        let x2 = (raw_x + raw_w).ceil() as i32 + 2;
                        let y2 = (raw_y + raw_h).ceil() as i32 + 2;

                        let safe_x = x1.max(0) as u32;
                        let safe_y = y1.max(0) as u32;
                        let safe_w = (x2 - x1.max(0)).max(0) as u32;
                        let safe_h = (y2 - y1.max(0)).max(0) as u32;

                        let rect = [safe_x, safe_y, safe_w, safe_h];
                        new_rect = Some(rect);
                        previous_visual_bounds.insert(node_id.0, rect);
                    } else {
                        previous_visual_bounds.remove(&node_id.0);
                    }

                    if let Some(old) = old_rect {
                        dirty_rects.push(old);
                    }
                    if let Some(new) = new_rect {
                        dirty_rects.push(new);
                    }
                }

                fw_state.dirty_rect = None;

                let target_nodes: std::collections::HashSet<crate::state::NodeId> =
                    nodes_to_update.iter().copied().collect();
                let mut topmost = Vec::new();
                for &node_id in &nodes_to_update {
                    let mut has_dirty_ancestor = false;
                    let mut curr = fw_state
                        .arena
                        .taffy_map
                        .get(&node_id)
                        .and_then(|t| fw_state.parent_map.get(t))
                        .copied();
                    while let Some(p) = curr {
                        if let Some(p_id) = fw_state.arena.node_map.get(&p) {
                            if target_nodes.contains(p_id) {
                                has_dirty_ancestor = true;
                                break;
                            }
                        }
                        curr = fw_state.parent_map.get(&p).copied();
                    }
                    if !has_dirty_ancestor {
                        topmost.push(node_id);
                    }
                }

                for top_id in topmost {
                    if let Some(t_node) = fw_state.arena.taffy_map.get(&top_id).copied() {
                        let p_pos = fw_state.get_parent_pos(&layout_engine, t_node);
                        let clip_rect = fw_state.get_clip_rect(t_node);
                        if let Some(widget_ref) = fw_state.arena.widgets[top_id.0 as usize].take() {
                            let subtree_output = widget_ref.render(
                                &layout_engine,
                                &mut fw_state,
                                t_node,
                                p_pos,
                                clip_rect,
                                "partial".to_string(),
                            );

                            // =========================================================================
                            // INQILOB: O(N) chiziqli qidiruvdan qutulib, O(1) xesh orqali tezkor qo'shish!
                            // =========================================================================
                            for (id, inst) in subtree_output.sparse_instances {
                                commands.push(RenderCommand::UpdateInstance(id, inst));
                                // insert() tekshiradi va yo'q bo'lsa qo'shadi. Juda tez!
                                if fw_state.draw_order_set.insert(id) {
                                    fw_state.current_draw_order.push(id);
                                }
                            }

                            for text in subtree_output.sparse_texts {
                                commands.push(RenderCommand::UpdateText(text.0, text));
                            }

                            for (id, cmds) in subtree_output.node_commands {
                                commands.push(RenderCommand::UpdateNodeCommands(id, cmds));
                                if fw_state.draw_order_set.insert(id) {
                                    fw_state.current_draw_order.push(id);
                                }
                            }

                            fw_state.arena.widgets[top_id.0 as usize] = Some(widget_ref);
                        }
                    }
                }
            }

            let mut current_active_nodes = std::collections::HashSet::new();
            for (i, &is_active) in fw_state.arena.active.iter().enumerate() {
                if is_active {
                    current_active_nodes.insert(i as u32);
                }
            }

            let deleted_nodes: Vec<u32> = previous_active_nodes
                .difference(&current_active_nodes)
                .copied()
                .collect();
            for &del_id in &deleted_nodes {
                commands.push(RenderCommand::Remove(del_id));
            }

            // INQILOB: Z-Index/Draw Order ro'yxatini tozalash!
            fw_state
                .current_draw_order
                .retain(|id| !deleted_nodes.contains(id));

            // =========================================================================
            // INQILOB: O'chirilganlarni Xesh-Jadvaldan ham o'chiramiz (Garbage Collection)
            // =========================================================================
            for del_id in &deleted_nodes {
                fw_state.draw_order_set.remove(del_id);
            }

            if !do_full_redraw {
                for &del_id in &deleted_nodes {
                    if let Some(old_rect) = previous_visual_bounds.remove(&del_id) {
                        dirty_rects.push(old_rect);
                    }
                }
            }

            previous_active_nodes = current_active_nodes;
            fw_state.clear_dirty_flags();

            let mut final_full_redraw = do_full_redraw;

            if !final_full_redraw && !dirty_rects.is_empty() {
                let merge_distance = 50;
                let mut merged = true;
                while merged {
                    merged = false;
                    let mut i = 0;
                    while i < dirty_rects.len() {
                        let mut j = i + 1;
                        while j < dirty_rects.len() {
                            let r1 = dirty_rects[i];
                            let r2 = dirty_rects[j];
                            let r1_exp = [
                                r1[0].saturating_sub(merge_distance),
                                r1[1].saturating_sub(merge_distance),
                                r1[2] + merge_distance * 2,
                                r1[3] + merge_distance * 2,
                            ];
                            let intersect = !(r2[0] > r1_exp[0] + r1_exp[2]
                                || r2[0] + r2[2] < r1_exp[0]
                                || r2[1] > r1_exp[1] + r1_exp[3]
                                || r2[1] + r2[3] < r1_exp[1]);
                            if intersect {
                                let min_x = r1[0].min(r2[0]);
                                let min_y = r1[1].min(r2[1]);
                                let max_x = (r1[0] + r1[2]).max(r2[0] + r2[2]);
                                let max_y = (r1[1] + r1[3]).max(r2[1] + r2[3]);
                                dirty_rects[i] = [min_x, min_y, max_x - min_x, max_y - min_y];
                                dirty_rects.remove(j);
                                merged = true;
                            } else {
                                j += 1;
                            }
                        }
                        i += 1;
                    }
                }

                while dirty_rects.len() > 10 {
                    let mut min_dist = u32::MAX;
                    let mut merge_pair = (0, 1);
                    for i in 0..dirty_rects.len() {
                        for j in (i + 1)..dirty_rects.len() {
                            let r1 = dirty_rects[i];
                            let r2 = dirty_rects[j];
                            let c1_x = r1[0] + r1[2] / 2;
                            let c1_y = r1[1] + r1[3] / 2;
                            let c2_x = r2[0] + r2[2] / 2;
                            let c2_y = r2[1] + r2[3] / 2;
                            let dx = c1_x.abs_diff(c2_x);
                            let dy = c1_y.abs_diff(c2_y);
                            let dist = dx * dx + dy * dy;
                            if dist < min_dist {
                                min_dist = dist;
                                merge_pair = (i, j);
                            }
                        }
                    }
                    let r1 = dirty_rects[merge_pair.0];
                    let r2 = dirty_rects[merge_pair.1];
                    let min_x = r1[0].min(r2[0]);
                    let min_y = r1[1].min(r2[1]);
                    let max_x = (r1[0] + r1[2]).max(r2[0] + r2[2]);
                    let max_y = (r1[1] + r1[3]).max(r2[1] + r2[3]);
                    dirty_rects[merge_pair.0] = [min_x, min_y, max_x - min_x, max_y - min_y];
                    dirty_rects.remove(merge_pair.1);
                }

                let mut total_area = 0;
                for r in &dirty_rects {
                    total_area += r[2] * r[3];
                }
                let screen_area = current_width.max(1.0) as u32 * current_height.max(1.0) as u32;
                if total_area > (screen_area as f32 * 0.70) as u32 {
                    final_full_redraw = true;
                    dirty_rects.clear();
                }
            }

            let total_nodes = fw_state.arena.widgets.len() as u32;
            let mut compiler = DisplayListCompiler::new();
            for (id, cmds) in &render_output.node_commands {
                compiler.compile(*id, cmds);
            }

            let _ = tx_render.send(RenderPacket {
                output: render_output,
                commands,
                current_cursor_icon: fw_state.current_cursor_icon,
                is_animating: false,
                total_nodes,
                scissor_rects: dirty_rects,
                is_full_redraw_forced: final_full_redraw,
                deleted_nodes: deleted_nodes.clone(),
                draw_order: fw_state.current_draw_order.clone(),
                custom_draws: compiler.final_custom,
            });

            // Winit ga tosh otish! Terminal spam bo'lmaydi
            wake_registry_logic.lock().unwrap().wake();
        }
    }
}
