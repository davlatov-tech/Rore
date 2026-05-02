use glam::Vec2;
use std::cell::RefCell;
use std::collections::HashSet;

use rore_core::reactive::command::{CommandQueue, UICommand};
use rore_core::reactive::signals::{create_effect, Signal};
use rore_core::state::{FrameworkState, NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, EventResult, RenderOutput, Widget, WidgetEvent};
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_render::Instance;
use rore_types::{Color, Style};

#[derive(Clone, Copy, Debug)]
pub struct CandleData {
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
}

pub struct CandlestickChart {
    pub id: Option<String>,
    pub style: Style,
    pub data: Signal<Vec<CandleData>>,
    pub pan_x: Signal<f32>,
    pub pan_y: Signal<f32>,
    pub zoom: Signal<f32>,
    node_id: Option<NodeId>,

    pool: Vec<NodeId>,
    last_used_ids: RefCell<HashSet<u32>>,
}

impl CandlestickChart {
    pub fn new(
        data: Signal<Vec<CandleData>>,
        pan_x: Signal<f32>,
        pan_y: Signal<f32>,
        zoom: Signal<f32>,
    ) -> Self {
        Self {
            id: None,
            style: Style::default(),
            data,
            pan_x,
            pan_y,
            zoom,
            node_id: None,
            pool: Vec::new(),
            last_used_ids: RefCell::new(HashSet::new()),
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
}

impl Widget for CandlestickChart {
    fn type_name(&self) -> &'static str {
        "CandlestickChart"
    }

    fn is_interactive(&self) -> bool {
        true
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        _ctx: &BuildContext,
    ) -> NodeId {
        let taffy_node = engine.new_leaf(self.style.clone());
        let my_id = arena.allocate_node();
        self.node_id = Some(my_id);

        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        if let Some(id_str) = &self.id {
            arena.register_id(id_str, my_id);
            engine.register_id(id_str, taffy_node);
        }

        engine.mark_interactive(taffy_node);

        let mut gpu_pool = Vec::with_capacity(1000);
        for _ in 0..1000 {
            gpu_pool.push(arena.allocate_node());
        }
        self.pool = gpu_pool;

        let d_sig = self.data;
        let p_x_sig = self.pan_x;
        let p_y_sig = self.pan_y;
        let z_sig = self.zoom;
        let id_clone = my_id;

        create_effect(move || {
            let _d = d_sig.get();
            let _px = p_x_sig.get();
            let _py = p_y_sig.get();
            let _z = z_sig.get();
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
            WidgetEvent::HoverEnter => {
                state.current_cursor_icon = winit::window::CursorIcon::Crosshair;
                return EventResult::Consumed;
            }
            WidgetEvent::HoverLeave => {
                state.current_cursor_icon = winit::window::CursorIcon::Default;
                return EventResult::Consumed;
            }
            WidgetEvent::MouseDrag { dx, dy } => {
                self.pan_x.update(|pan| *pan += dx);
                self.pan_y.update(|pan| *pan += dy);
                changed = true;
            }
            WidgetEvent::MouseScroll {
                delta_x: _,
                delta_y,
            } => {
                self.zoom.update(|z| {
                    let mut new_z = *z - delta_y * 0.05;
                    new_z = new_z.clamp(0.1, 20.0);
                    *z = new_z;
                });
                changed = true;
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
        _state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        clip_rect: Option<[f32; 4]>,
        _path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);

        let data = self.data.get_untracked();
        let mut pool_idx = 0;
        let mut current_used_ids = std::collections::HashSet::new();

        if !data.is_empty() && layout.width > 0.0 && layout.height > 0.0 {
            let zoom = self.zoom.get_untracked().max(0.1);
            let pan_x = self.pan_x.get_untracked();
            let pan_y = self.pan_y.get_untracked();

            let candle_w = 6.0 * zoom;
            let spacing = 2.0 * zoom;
            let step = candle_w + spacing;
            let right_edge = layout.x + layout.width + pan_x - spacing;

            let mut min_price = f32::MAX;
            let mut max_price = f32::MIN;
            let mut visible_candles = Vec::new();

            // CULLING (X o'qi bo'yicha)
            for (i, candle) in data.iter().enumerate() {
                let distance_from_end = (data.len() - 1 - i) as f32;
                let cx = right_edge - (distance_from_end * step) - candle_w;

                if cx + candle_w >= layout.x && cx <= layout.x + layout.width {
                    if candle.low < min_price {
                        min_price = candle.low;
                    }
                    if candle.high > max_price {
                        max_price = candle.high;
                    }
                    visible_candles.push((cx, candle));
                }
            }

            let my_clip = if let Some(c) = clip_rect {
                let min_x = c[0].max(layout.x);
                let min_y = c[1].max(layout.y);
                let max_x = (c[0] + c[2]).min(layout.x + layout.width);
                let max_y = (c[1] + c[3]).min(layout.y + layout.height);
                [
                    min_x,
                    min_y,
                    (max_x - min_x).max(0.0),
                    (max_y - min_y).max(0.0),
                ]
            } else {
                [layout.x, layout.y, layout.width, layout.height]
            };

            let padding_y = layout.height * 0.1;
            let drawable_height = layout.height - (padding_y * 2.0);

            let grid_spacing = drawable_height * 0.2;
            let mut offset_y = pan_y % grid_spacing;
            if offset_y < 0.0 {
                offset_y += grid_spacing;
            } // Salbiy raqamlarni himoyalash

            let grid_color = Color::hex("#ffffff").with_alpha(0.03); // Tiniq hex rang ishlatiyapti
            let grid_color_arr = [grid_color.r, grid_color.g, grid_color.b, grid_color.a];

            for i in -1..6 {
                if pool_idx >= self.pool.len() {
                    break;
                }
                let grid_id = self.pool[pool_idx].0;
                pool_idx += 1;
                current_used_ids.insert(grid_id);

                let y = layout.y + padding_y + (grid_spacing * i as f32) + offset_y;
                let grid_inst = Instance {
                    position: Vec2::new(layout.x, y),
                    size: Vec2::new(layout.width, 1.0),
                    color_start: grid_color_arr,
                    color_end: grid_color_arr,
                    target_color_start: grid_color_arr,
                    target_color_end: grid_color_arr,
                    gradient_angle: 0.0,
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                    target_border_color: [0.0; 4],
                    shadow_color: [0.0; 4],
                    shadow_offset: Vec2::ZERO,
                    shadow_blur: 0.0,
                    shadow_spread: 0.0,
                    clip_rect: my_clip,
                    anim_start_time: 0.0,
                    anim_duration: 0.0,
                };
                output.sparse_instances.push((grid_id, grid_inst));
            }

            if !visible_candles.is_empty() {
                let price_range = (max_price - min_price).max(0.001);
                let price_to_y = |price: f32| -> f32 {
                    let normalized = (max_price - price) / price_range;
                    layout.y + padding_y + (normalized * drawable_height) + pan_y
                };

                let color_green = Color::hex("#0ECB81");
                let color_red = Color::hex("#F6465D");
                let green_arr = [color_green.r, color_green.g, color_green.b, color_green.a];
                let red_arr = [color_red.r, color_red.g, color_red.b, color_red.a];

                for (cx, candle) in visible_candles {
                    if pool_idx + 2 > self.pool.len() {
                        break;
                    }

                    let y_high = price_to_y(candle.high);
                    let y_low = price_to_y(candle.low);
                    let mut y_open = price_to_y(candle.open);
                    let mut y_close = price_to_y(candle.close);

                    let is_bullish = candle.close >= candle.open;
                    let color = if is_bullish { green_arr } else { red_arr };
                    if !is_bullish {
                        std::mem::swap(&mut y_open, &mut y_close);
                    }

                    let body_height = (y_open - y_close).max(1.0);
                    let wick_x = cx + (candle_w / 2.0) - 0.5;

                    // Wick
                    let wick_id = self.pool[pool_idx].0;
                    pool_idx += 1;
                    current_used_ids.insert(wick_id);
                    let wick_inst = Instance {
                        position: Vec2::new(wick_x, y_high),
                        size: Vec2::new(1.0, (y_low - y_high).max(1.0)),
                        color_start: color,
                        color_end: color,
                        target_color_start: color,
                        target_color_end: color,
                        gradient_angle: 0.0,
                        border_radius: 0.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                        target_border_color: [0.0; 4],
                        shadow_color: [0.0; 4],
                        shadow_offset: Vec2::ZERO,
                        shadow_blur: 0.0,
                        shadow_spread: 0.0,
                        clip_rect: my_clip,
                        anim_start_time: 0.0,
                        anim_duration: 0.0,
                    };
                    output.sparse_instances.push((wick_id, wick_inst));

                    // Body
                    let body_id = self.pool[pool_idx].0;
                    pool_idx += 1;
                    current_used_ids.insert(body_id);
                    let body_inst = Instance {
                        position: Vec2::new(cx, y_close),
                        size: Vec2::new(candle_w, body_height),
                        color_start: color,
                        color_end: color,
                        target_color_start: color,
                        target_color_end: color,
                        gradient_angle: 0.0,
                        border_radius: 1.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                        target_border_color: [0.0; 4],
                        shadow_color: [0.0; 4],
                        shadow_offset: Vec2::ZERO,
                        shadow_blur: 0.0,
                        shadow_spread: 0.0,
                        clip_rect: my_clip,
                        anim_start_time: 0.0,
                        anim_duration: 0.0,
                    };
                    output.sparse_instances.push((body_id, body_inst));
                }
            }
        }

        let ghost_inst = Instance {
            position: Vec2::new(-10000.0, -10000.0),
            size: Vec2::ZERO,
            color_start: [0.0; 4],
            color_end: [0.0; 4],
            target_color_start: [0.0; 4],
            target_color_end: [0.0; 4],
            gradient_angle: 0.0,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: [0.0; 4],
            target_border_color: [0.0; 4],
            shadow_color: [0.0; 4],
            shadow_offset: Vec2::ZERO,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            clip_rect: [-10000.0, -10000.0, 0.0, 0.0],
            anim_start_time: 0.0,
            anim_duration: 0.0,
        };

        for old_id in self.last_used_ids.borrow().iter() {
            if !current_used_ids.contains(old_id) {
                output.sparse_instances.push((*old_id, ghost_inst.clone()));
            }
        }

        *self.last_used_ids.borrow_mut() = current_used_ids;

        output
    }
}
