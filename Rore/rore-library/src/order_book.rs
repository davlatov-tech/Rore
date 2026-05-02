use glam::Vec2;
use rore_core::reactive::signals::Signal;
use rore_core::state::{NodeId, UiArena};
use rore_core::widgets::base::{BuildContext, RenderOutput, Widget};
use rore_core::widgets::list::ForList;
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_text::widgets::{ScrollView, Text, UiBox};
use rore_types::{Align, Color, FlexDirection, Style, Thickness, Val};
#[derive(Clone, PartialEq, Debug)]
pub struct OrderRow {
    pub price: f32,
    pub amount: f32,
    pub total: f32,
}

pub struct OrderBook {
    pub asks: Signal<Vec<OrderRow>>,
    pub bids: Signal<Vec<OrderRow>>,
}

impl OrderBook {
    pub fn new(asks: Signal<Vec<OrderRow>>, bids: Signal<Vec<OrderRow>>) -> Self {
        Self { asks, bids }
    }
}

impl Widget for OrderBook {
    fn type_name(&self) -> &'static str {
        "OrderBook"
    }
    fn is_interactive(&self) -> bool {
        false
    }

    fn build(
        self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let text_muted = Color::hex("#848E9C"); // Binance Muted Text

        // HEADER
        let header = UiBox::new()
            .style(Style {
                flex_direction: FlexDirection::Row,
                justify_content: Align::SpaceBetween, // Ikki chetga yoyish
                padding: Thickness {
                    left: Val::Px(16.0),
                    right: Val::Px(16.0),
                    top: Val::Px(12.0),
                    bottom: Val::Px(8.0),
                },
                ..Default::default()
            })
            .child(
                UiBox::new()
                    .style(Style {
                        width: Val::Percent(33.3),
                        align_items: Align::Start,
                        ..Default::default()
                    })
                    .child(Text::new("Price(USDT)").color(text_muted).size(12.0)),
            )
            .child(
                UiBox::new()
                    .style(Style {
                        width: Val::Percent(33.3),
                        align_items: Align::End,
                        ..Default::default()
                    })
                    .child(Text::new("Amount(BTC)").color(text_muted).size(12.0)),
            )
            .child(
                UiBox::new()
                    .style(Style {
                        width: Val::Percent(33.3),
                        align_items: Align::End,
                        ..Default::default()
                    })
                    .child(Text::new("Total").color(text_muted).size(12.0)),
            );

        // SCROLL AREA
        let scroll_area = ScrollView::new()
            .style(Style {
                flex_grow: 1.0,
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..Default::default()
            })
            .child(ForList::new(self.asks, |row| {
                build_order_row(row, Color::hex("#F6465D"))
            })) // Binance Red
            .child(UiBox::new().style(Style {
                height: Val::Px(10.0),
                ..Default::default()
            }))
            .child(ForList::new(self.bids, |row| {
                build_order_row(row, Color::hex("#0ECB81"))
            })); // Binance Green

        let root = UiBox::new()
            .style(Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..Default::default()
            })
            .child(header)
            .child(scroll_area);

        Box::new(root).build(arena, engine, ctx)
    }

    fn render(
        &self,
        _engine: &LayoutEngine,
        _state: &mut rore_core::state::FrameworkState,
        _node: TaffyNode,
        _pos: Vec2,
        _clip: Option<[f32; 4]>,
        _path: String,
    ) -> RenderOutput {
        RenderOutput::new()
    }
}

fn build_order_row(row: OrderRow, price_color: Color) -> Box<dyn Widget> {
    Box::new(
        UiBox::new()
            .style(Style {
                flex_direction: FlexDirection::Row,
                justify_content: Align::SpaceBetween,
                padding: Thickness {
                    left: Val::Px(16.0),
                    right: Val::Px(16.0),
                    top: Val::Px(2.0),
                    bottom: Val::Px(2.0),
                }, // Zich qatorlar
                ..Default::default()
            })
            // PRICE (Chapga)
            .child(
                UiBox::new()
                    .style(Style {
                        width: Val::Percent(33.3),
                        align_items: Align::Start,
                        ..Default::default()
                    })
                    .child(
                        Text::new(format!("{:.2}", row.price))
                            .color(price_color)
                            .size(12.0),
                    ),
            )
            // AMOUNT (O'ngga)
            .child(
                UiBox::new()
                    .style(Style {
                        width: Val::Percent(33.3),
                        align_items: Align::End,
                        ..Default::default()
                    })
                    .child(
                        Text::new(format!("{:.5}", row.amount))
                            .color(Color::hex("#EAECEF"))
                            .size(12.0),
                    ),
            )
            // TOTAL (O'ngga)
            .child(
                UiBox::new()
                    .style(Style {
                        width: Val::Percent(33.3),
                        align_items: Align::End,
                        ..Default::default()
                    })
                    .child(
                        Text::new(format!("{:.2}", row.total))
                            .color(Color::hex("#848E9C"))
                            .size(12.0),
                    ),
            ),
    )
}
