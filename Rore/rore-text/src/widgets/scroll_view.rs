use glam::Vec2;
use rore_core::reactive::command::{CommandQueue, UICommand};
use rore_core::reactive::signals::{create_effect, create_signal_untracked, Signal};
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, EventResult, RenderOutput, Widget, WidgetEvent};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_render::Instance;
use rore_types::Style;
use std::cell::Cell;

pub struct ScrollView {
    pub id: Option<String>,
    pub style: Style,
    pub child: Option<Box<dyn Widget>>,

    pub scroll_x: Signal<f32>,
    pub scroll_y: Signal<f32>,

    pub max_scroll_x: Cell<f32>,
    pub max_scroll_y: Cell<f32>,

    pub velocity_x: Cell<f32>,
    pub velocity_y: Cell<f32>,
    pub is_dragging: Cell<bool>,
    pub is_animating: Cell<bool>,
    pub last_time: Cell<f32>,

    node_id: Option<NodeId>,
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            id: None,
            style: Style::default(),
            child: None,
            scroll_x: create_signal_untracked(0.0),
            scroll_y: create_signal_untracked(0.0),
            max_scroll_x: Cell::new(0.0),
            max_scroll_y: Cell::new(0.0),

            velocity_x: Cell::new(0.0),
            velocity_y: Cell::new(0.0),
            is_dragging: Cell::new(false),
            is_animating: Cell::new(false),
            last_time: Cell::new(0.0),

            node_id: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(w));
        self
    }

    pub fn scroll_x(mut self, s: Signal<f32>) -> Self {
        self.scroll_x = s;
        self
    }

    pub fn scroll_y(mut self, s: Signal<f32>) -> Self {
        self.scroll_y = s;
        self
    }
}

impl Widget for ScrollView {
    fn type_name(&self) -> &'static str {
        "ScrollView"
    }

    fn is_interactive(&self) -> bool {
        true
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let mut child_nodes = Vec::new();
        if let Some(child) = self.child.take() {
            let child_id = child.build(arena, engine, ctx);
            if let Some(&t_node) = arena.taffy_map.get(&child_id) {
                child_nodes.push(t_node);
            }
        }

        let mut taffy_scroll_style = self.style.clone();
        taffy_scroll_style.overflow = rore_types::Overflow::Scroll;
        taffy_scroll_style.min_height = rore_types::Val::Px(0.0);
        taffy_scroll_style.min_width = rore_types::Val::Px(0.0);

        // TAFFY'NING MARKAZGA TORTISH BUGINI YO'Q QILAMIZ:
        taffy_scroll_style.flex_direction = rore_types::FlexDirection::Column; // Skroll asosan vertikal ketadi
        taffy_scroll_style.align_items = rore_types::Align::Start; // Yon tomonni tepaga tortish
        taffy_scroll_style.justify_content = rore_types::Align::Start; // Balandlikni tepaga tortish

        let taffy_node = engine.new_node(taffy_scroll_style, &child_nodes);
        let my_id = arena.allocate_node();
        self.node_id = Some(my_id);

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        if let Some(id_str) = &self.id {
            arena.register_id(id_str, my_id);
            engine.register_id(id_str, taffy_node);
        }

        engine.mark_interactive(taffy_node);

        let sig_x = self.scroll_x;
        let sig_y = self.scroll_y;
        let id_clone = my_id;

        create_effect(move || {
            let _x = sig_x.get();
            let _y = sig_y.get();
            CommandQueue::send(UICommand::MarkDirty(
                id_clone,
                rore_core::state::DIRTY_COLOR,
            ));
        });

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    // ... tepadagi importlar va struktura tana qismlari o'zgarishsiz ...

    fn handle_event(&mut self, state: &mut FrameworkState, event: &WidgetEvent) -> EventResult {
        let mut changed = false;

        match event {
            WidgetEvent::MouseDown => {
                self.is_dragging.set(true);
                self.velocity_x.set(0.0);
                self.velocity_y.set(0.0);
                changed = true;
            }
            WidgetEvent::MouseUp | WidgetEvent::HoverLeave => {
                if self.is_dragging.get() {
                    self.is_dragging.set(false);
                    self.last_time.set(state.global_time);
                    if let Some(id) = self.node_id {
                        if !state.sparse_update_queue.contains(&id) {
                            state.sparse_update_queue.push(id);
                        }
                    }
                    state.request_redraw();
                    changed = true;
                }
            }
            WidgetEvent::MouseScroll { delta_x, delta_y } => {
                let current_y = self.scroll_y.get_untracked();
                let current_x = self.scroll_x.get_untracked();

                let new_y = (current_y - delta_y).clamp(0.0, self.max_scroll_y.get());
                let new_x = (current_x - delta_x).clamp(0.0, self.max_scroll_x.get());

                if new_y != current_y {
                    self.scroll_y.set(new_y);
                    changed = true;
                }
                if new_x != current_x {
                    self.scroll_x.set(new_x);
                    changed = true;
                }
            }
            WidgetEvent::MouseDrag { dx, dy } => {
                self.is_dragging.set(true);
                self.last_time.set(state.global_time);

                let current_y = self.scroll_y.get_untracked();
                let current_x = self.scroll_x.get_untracked();

                let new_y = (current_y - dy).clamp(0.0, self.max_scroll_y.get());
                let new_x = (current_x - dx).clamp(0.0, self.max_scroll_x.get());

                self.velocity_x
                    .set((self.velocity_x.get() * 0.5 + dx * 0.5).clamp(-60.0, 60.0));
                self.velocity_y
                    .set((self.velocity_y.get() * 0.5 + dy * 0.5).clamp(-60.0, 60.0));

                if new_y != current_y {
                    self.scroll_y.set(new_y);
                    changed = true;
                }
                if new_x != current_x {
                    self.scroll_x.set(new_x);
                    changed = true;
                }
            }
            _ => return EventResult::Ignored,
        }

        if changed {
            if let Some(id) = self.node_id {
                // INQILOB: Skroll o'zgarishi bilanoq uni Yadroga aytamiz!
                state.scroll_offsets.insert(
                    id,
                    Vec2::new(self.scroll_x.get_untracked(), self.scroll_y.get_untracked()),
                );
                state.needs_aabb_update = true; // CPU zudlik bilan hamma tugmalar koordinatasini yangilaydi

                if !state.sparse_update_queue.contains(&id) {
                    state.sparse_update_queue.push(id);
                }
            }
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);
        let my_id = self.node_id.unwrap();

        let mut max_x = self.max_scroll_x.get();
        let mut max_y = self.max_scroll_y.get();

        let mut vx = self.velocity_x.get();
        let mut vy = self.velocity_y.get();
        let is_drag = self.is_dragging.get();
        let moving = !is_drag && (vx.abs() > 0.5 || vy.abs() > 0.5);

        if moving {
            if !self.is_animating.get() {
                self.is_animating.set(true);
                let lock_name = format!("scroll_{}", my_id.0);
                state.wake_registry.lock().unwrap().acquire(&lock_name);
            }

            let current_time = state.global_time;
            let dt = (current_time - self.last_time.get()).max(0.001).min(0.05);
            self.last_time.set(current_time);

            let mut cx = self.scroll_x.get_untracked();
            let mut cy = self.scroll_y.get_untracked();

            cx -= vx * dt * 60.0;
            cy -= vy * dt * 60.0;

            if cx < 0.0 {
                cx = 0.0;
                vx = 0.0;
            } else if cx > max_x {
                cx = max_x;
                vx = 0.0;
            }
            if cy < 0.0 {
                cy = 0.0;
                vy = 0.0;
            } else if cy > max_y {
                cy = max_y;
                vy = 0.0;
            }

            vx *= 0.92;
            vy *= 0.92;

            self.velocity_x.set(vx);
            self.velocity_y.set(vy);

            if cx != self.scroll_x.get_untracked() || cy != self.scroll_y.get_untracked() {
                self.scroll_x.set(cx);
                self.scroll_y.set(cy);
                // INQILOB: Kinetik skrolling o'zgarishini ham Yadroga beramiz!
                state.scroll_offsets.insert(my_id, Vec2::new(cx, cy));
                state.needs_aabb_update = true;
            }

            if !state.sparse_update_queue.contains(&my_id) {
                state.sparse_update_queue.push(my_id);
            }
        } else {
            if self.is_animating.get() {
                self.is_animating.set(false);
                self.velocity_x.set(0.0);
                self.velocity_y.set(0.0);
                let lock_name = format!("scroll_{}", my_id.0);
                state.wake_registry.lock().unwrap().release(&lock_name);
            }
        }

        let my_bounds = [layout.x, layout.y, layout.width, layout.height];
        let strict_clip = if let Some(clip) = clip_rect {
            let min_x = clip[0].max(my_bounds[0]);
            let min_y = clip[1].max(my_bounds[1]);
            let max_x = (clip[0] + clip[2]).min(my_bounds[0] + my_bounds[2]);
            let max_y = (clip[1] + clip[3]).min(my_bounds[1] + my_bounds[3]);
            Some([
                min_x,
                min_y,
                (max_x - min_x).max(0.0),
                (max_y - min_y).max(0.0),
            ])
        } else {
            Some(my_bounds)
        };

        let sx = self.scroll_x.get_untracked();
        let sy = self.scroll_y.get_untracked();

        let (content_w, content_h) = engine.get_scroll_size(taffy_node);
        max_x = (content_w - layout.width).max(0.0);
        max_y = (content_h - layout.height).max(0.0);

        self.max_scroll_x.set(max_x);
        self.max_scroll_y.set(max_y);

        if let Ok(children) = engine.taffy.children(taffy_node) {
            let child_parent_pos = Vec2::new(layout.x - sx, layout.y - sy);

            for (i, &child_node) in children.iter().enumerate() {
                if let Some(&child_id) = state.arena.node_map.get(&child_node) {
                    if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                        let child_output = widget_ref.render(
                            engine,
                            state,
                            child_node,
                            child_parent_pos,
                            strict_clip,
                            format!("{}_scroll_{}", path, i),
                        );
                        output.extend(child_output);
                        state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
                    }
                }
            }

            // ... Skrollbar chizish qismlari pastga qarab o'zgarishsiz davom etadi ...

            let scrollbar_thickness = 6.0;
            let scrollbar_margin = 2.0;
            let scrollbar_color = [0.5, 0.5, 0.5, 0.4];

            if max_y > 0.0 {
                let content_h = layout.height + max_y;
                let thumb_h = (layout.height / content_h * layout.height).max(20.0);
                let thumb_y = layout.y + (sy / max_y) * (layout.height - thumb_h);

                let thumb_inst = Instance {
                    position: Vec2::new(
                        layout.x + layout.width - scrollbar_thickness - scrollbar_margin,
                        thumb_y,
                    ),
                    size: Vec2::new(scrollbar_thickness, thumb_h),
                    color_start: scrollbar_color,
                    color_end: scrollbar_color,
                    target_color_start: scrollbar_color,
                    target_color_end: scrollbar_color,
                    gradient_angle: 0.0,
                    border_radius: [3.0; 4],
                    border_width: [0.0; 4],
                    border_color: [0.0; 4],
                    target_border_color: [0.0; 4],
                    shadow_color: [0.0; 4],
                    shadow_offset: Vec2::ZERO,
                    shadow_blur: 0.0,
                    shadow_spread: 0.0,
                    clip_rect: strict_clip.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]),
                    anim_start_time: 0.0,
                    anim_duration: 0.0,
                };
                output.sparse_instances.push((my_id.0 + 200000, thumb_inst));
            }

            if max_x > 0.0 {
                let content_w = layout.width + max_x;
                let thumb_w = (layout.width / content_w * layout.width).max(20.0);
                let thumb_x = layout.x + (sx / max_x) * (layout.width - thumb_w);

                let thumb_inst = Instance {
                    position: Vec2::new(
                        thumb_x,
                        layout.y + layout.height - scrollbar_thickness - scrollbar_margin,
                    ),
                    size: Vec2::new(thumb_w, scrollbar_thickness),
                    color_start: scrollbar_color,
                    color_end: scrollbar_color,
                    target_color_start: scrollbar_color,
                    target_color_end: scrollbar_color,
                    gradient_angle: 0.0,
                    border_radius: [3.0; 4],
                    border_width: [0.0; 4],
                    border_color: [0.0; 4],
                    target_border_color: [0.0; 4],
                    shadow_color: [0.0; 4],
                    shadow_offset: Vec2::ZERO,
                    shadow_blur: 0.0,
                    shadow_spread: 0.0,
                    clip_rect: strict_clip.unwrap_or([-10000.0, -10000.0, 20000.0, 20000.0]),
                    anim_start_time: 0.0,
                    anim_duration: 0.0,
                };
                output.sparse_instances.push((my_id.0 + 200001, thumb_inst));
            }
        }

        output
    }
}
