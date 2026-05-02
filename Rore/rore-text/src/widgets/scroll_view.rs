use glam::Vec2;
use rore_core::reactive::command::{CommandQueue, UICommand};
use rore_core::reactive::signals::{create_effect, create_signal_untracked, Signal};
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, EventResult, RenderOutput, Widget, WidgetEvent};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::Style;
use std::cell::Cell;

pub struct ScrollView {
    pub id: Option<String>,
    pub style: Style,
    pub child: Option<Box<dyn Widget>>,

    pub scroll_x: Signal<f32>,
    pub scroll_y: Signal<f32>,

    // Maksimal chegaralar render qismida hisoblanadi
    pub max_scroll_x: Cell<f32>,
    pub max_scroll_y: Cell<f32>,

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
}

impl Widget for ScrollView {
    fn type_name(&self) -> &'static str {
        "ScrollView"
    }

    fn is_interactive(&self) -> bool {
        // HODISALARNI USHLASH UCHUN: Vidjet kursor hodisalariga javob beradi
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

        let taffy_node = engine.new_node(self.style.clone(), &child_nodes);
        let my_id = arena.allocate_node();
        self.node_id = Some(my_id);

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        if let Some(id_str) = &self.id {
            arena.register_id(id_str, my_id);
            engine.register_id(id_str, taffy_node);
        }

        // Qutining o'zini hodisalarni eshita oladigan qilib belgilaymiz
        engine.mark_interactive(taffy_node);

        // REAKTIV MO'JIZA: Scroll o'zgarganda faqat Render xaritasi yangilanadi, Taffy UXLAB YOTADI!
        let sig_x = self.scroll_x;
        let sig_y = self.scroll_y;
        let id_clone = my_id;

        create_effect(move || {
            let _x = sig_x.get();
            let _y = sig_y.get();
            // DIRTY_COLOR flagi yadroga Taffy'ni bezovta qilmasdan faqat chizishni yangilashni aytadi
            CommandQueue::send(UICommand::MarkDirty(
                id_clone,
                rore_core::state::DIRTY_COLOR,
            ));
        });

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    fn handle_event(&mut self, state: &mut FrameworkState, event: &WidgetEvent) -> EventResult {
        let mut changed = false;

        match event {
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
                let current_y = self.scroll_y.get_untracked();
                let current_x = self.scroll_x.get_untracked();

                let new_y = (current_y - dy).clamp(0.0, self.max_scroll_y.get());
                let new_x = (current_x - dx).clamp(0.0, self.max_scroll_x.get());

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

        if let Ok(children) = engine.taffy.children(taffy_node) {
            let mut total_child_width = 0.0_f32;
            let mut total_child_height = 0.0_f32;

            for &child_node in &children {
                if let Ok(child_layout) = engine.taffy.layout(child_node) {
                    let right = child_layout.location.x + child_layout.size.width;
                    let bottom = child_layout.location.y + child_layout.size.height;

                    if right > total_child_width {
                        total_child_width = right;
                    }
                    if bottom > total_child_height {
                        total_child_height = bottom;
                    }
                }
            }

            let max_x = (total_child_width - layout.width).max(0.0);
            let max_y = (total_child_height - layout.height).max(0.0);

            // Xotiraga yozamiz. Kursor o'zgartirganda limitdan chiqib ketmaydi.
            self.max_scroll_x.set(max_x);
            self.max_scroll_y.set(max_y);

            let child_parent_pos = Vec2::new(layout.x - sx, layout.y - sy);

            for (i, &child_node) in children.iter().enumerate() {
                if let Some(&child_id) = state.arena.node_map.get(&child_node) {
                    if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                        let child_output = widget_ref.render(
                            engine,
                            state,
                            child_node,
                            child_parent_pos,
                            strict_clip, // GPU endi qutidan toshib chiqqanlarni chizmaydi!
                            format!("{}_scroll_{}", path, i),
                        );
                        output.extend(child_output);
                        state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
                    }
                }
            }
        }

        output
    }
}
