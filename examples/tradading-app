use std::cell::RefCell;

use rore_core::app::{run, App, AppEvent, Widget};
use rore_core::reactive::signals::{create_selector, create_ticker, provide_context, Signal};

// Yadroviy "Lego" Vidjetlar
use rore_text::text::TextSystem;
use rore_text::widgets::theme::Theme;
use rore_text::widgets::{Button, Text, TextInput, UiBox};
use rore_types::*;

// Kutubxona va haqiqiy ishlovchi vidjetlarni import qilamiz
use rore_library::chart::CandlestickChart;

use rore_library::{CandleData, OrderRow};

thread_local! {
    static SCREEN_SIZE: RefCell<(f32, f32)> = RefCell::new((1024.0, 768.0));
}

struct RoreTradingApp {}

impl App for RoreTradingApp {
    fn view(&self) -> Box<dyn Widget> {
        let theme_signal = Signal::new(Theme::dark());
        provide_context(theme_signal);

        let is_buy = Signal::new(true);

        let initial_price_btc = 68109.98;
        let mut initial_candles_btc = Vec::new();
        let mut price_btc = initial_price_btc;
        let mut rng = 12345u32;

        for _ in 0..150 {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let volatility = ((rng % 100) as f32 - 50.0) * 15.0;
            let open = price_btc;
            let close = price_btc + volatility;
            let high = open.max(close) + ((rng % 50) as f32 * 5.0);
            let low = open.min(close) - (((rng >> 8) % 50) as f32 * 5.0);
            initial_candles_btc.push(CandleData {
                open,
                high,
                low,
                close,
            });
            price_btc = close;
        }

        let candles_btc = Signal::new(initial_candles_btc);
        let current_price_btc = Signal::new(price_btc);

        let pan_x_btc = Signal::new(0.0);
        let pan_y_btc = Signal::new(0.0);
        let zoom_btc = Signal::new(4.0);

        let initial_price_eth = 3540.25;
        let mut initial_candles_eth = Vec::new();
        let mut price_eth = initial_price_eth;
        let mut rng_eth = 98765u32; // ETH uchun butunlay boshqa Seed (boshqa yo'nalish)

        for _ in 0..150 {
            rng_eth = rng_eth.wrapping_mul(1103515245).wrapping_add(12345);
            let volatility = ((rng_eth % 100) as f32 - 50.0) * 2.0; // ETH da absolyut sonlar kichikroq
            let open = price_eth;
            let close = price_eth + volatility;
            let high = open.max(close) + ((rng_eth % 50) as f32 * 1.0);
            let low = open.min(close) - (((rng_eth >> 8) % 50) as f32 * 1.0);
            initial_candles_eth.push(CandleData {
                open,
                high,
                low,
                close,
            });
            price_eth = close;
        }

        let candles_eth = Signal::new(initial_candles_eth);
        let current_price_eth = Signal::new(price_eth);

        let pan_x_eth = Signal::new(0.0);
        let pan_y_eth = Signal::new(0.0);
        let zoom_eth = Signal::new(4.0);

        let mut asks_data = Vec::new();
        let mut bids_data = Vec::new();
        for i in 0..20 {
            let ask_p = price_btc + 10.0 + (i as f32 * 5.0);
            let bid_p = price_btc - 10.0 - (i as f32 * 5.0);
            asks_data.push(OrderRow {
                price: ask_p,
                amount: 2.5,
                total: ask_p * 2.5,
            });
            bids_data.push(OrderRow {
                price: bid_p,
                amount: 3.1,
                total: bid_p * 3.1,
            });
        }
        asks_data.reverse();

        let asks = Signal::new(asks_data);
        let bids = Signal::new(bids_data);

        let mut frame_count = 0;
        create_ticker(move |_dt| {
            frame_count += 1;

            // BTC Harakati
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let tick_btc = ((rng % 100) as f32 - 50.0) * 1.5; // Shiddatliroq tebranish

            candles_btc.update(|data| {
                if let Some(last) = data.last_mut() {
                    last.close += tick_btc;
                    if last.close > last.high {
                        last.high = last.close;
                    }
                    if last.close < last.low {
                        last.low = last.close;
                    }
                    current_price_btc.set(last.close);
                }
                if frame_count % 120 == 0 {
                    let last_close = data.last().unwrap().close;
                    data.push(CandleData {
                        open: last_close,
                        high: last_close,
                        low: last_close,
                        close: last_close,
                    });
                }
            });

            // ETH Harakati
            rng_eth = rng_eth.wrapping_mul(1103515245).wrapping_add(12345);
            let tick_eth = ((rng_eth % 100) as f32 - 50.0) * 0.4;

            candles_eth.update(|data| {
                if let Some(last) = data.last_mut() {
                    last.close += tick_eth;
                    if last.close > last.high {
                        last.high = last.close;
                    }
                    if last.close < last.low {
                        last.low = last.close;
                    }
                    current_price_eth.set(last.close);
                }
                if frame_count % 120 == 0 {
                    let last_close = data.last().unwrap().close;
                    data.push(CandleData {
                        open: last_close,
                        high: last_close,
                        low: last_close,
                        close: last_close,
                    });
                }
            });

            // OrderBook harakati
            if frame_count % 5 == 0 {
                asks.update(|a| {
                    for (i, row) in a.iter_mut().enumerate() {
                        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                        let base_amt = 2.0 + (i as f32 * 0.5);
                        let noise = ((rng % 200) as f32 / 100.0) - 1.0;
                        row.amount = (base_amt + noise).max(0.01);
                        row.total = row.price * row.amount;
                    }
                });
                bids.update(|b| {
                    for (i, row) in b.iter_mut().enumerate() {
                        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                        let base_amt = 2.0 + (i as f32 * 0.5);
                        let noise = ((rng % 200) as f32 / 100.0) - 1.0;
                        row.amount = (base_amt + noise).max(0.01);
                        row.total = row.price * row.amount;
                    }
                });
            }
        });

        Box::new(
            UiBox::new()
                .bg_color(Color::hex("#0B0E11"))
                .style(Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    gap: Size {
                        width: 0.0,
                        height: 2.0,
                    },
                    ..Default::default()
                })
                .child(build_topbar(theme_signal, current_price_btc))
                .child(
                    UiBox::new()
                        .style(Style {
                            flex_grow: 1.0,
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            gap: Size {
                                width: 2.0,
                                height: 0.0,
                            }, // Panellar orasidagi qora chiziq
                            ..Default::default()
                        })
                        .child(build_left_toolbar())
                        .child(
                            UiBox::new()
                                .style(Style {
                                    flex_grow: 1.0,
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    gap: Size {
                                        width: 0.0,
                                        height: 2.0,
                                    },
                                    ..Default::default()
                                })
                                // ASOSIY GRAFIK BLOKI (BTC / USDT)
                                .child(
                                    UiBox::new()
                                        .bg_color(Color::hex("#181A20"))
                                        .style(Style {
                                            flex_grow: 6.0,
                                            width: Val::Percent(100.0),
                                            flex_direction: FlexDirection::Column,
                                            ..Default::default()
                                        })
                                        .child(build_chart_toolbar())
                                        .child(
                                            CandlestickChart::new(
                                                candles_btc,
                                                pan_x_btc,
                                                pan_y_btc,
                                                zoom_btc,
                                            )
                                            .style(
                                                Style {
                                                    flex_grow: 1.0,
                                                    width: Val::Percent(100.0),
                                                    ..Default::default()
                                                },
                                            ),
                                        ),
                                )
                                .child(
                                    UiBox::new()
                                        .bg_color(Color::hex("#181A20"))
                                        .style(Style {
                                            flex_grow: 4.0,
                                            width: Val::Percent(100.0),
                                            flex_direction: FlexDirection::Column,
                                            ..Default::default()
                                        })
                                        .child(
                                            // Kichik sarlavha
                                            UiBox::new()
                                                .style(Style {
                                                    flex_direction: FlexDirection::Row,
                                                    width: Val::Percent(100.0),
                                                    padding: Thickness {
                                                        left: Val::Px(16.0),
                                                        top: Val::Px(8.0),
                                                        bottom: Val::Px(8.0),
                                                        right: Val::Px(16.0),
                                                    },
                                                    ..Default::default()
                                                })
                                                .child(
                                                    Text::new("ETH / USDT")
                                                        .color(Color::hex("#848E9C"))
                                                        .size(12.0),
                                                )
                                                .child(
                                                    Text::new(create_selector(
                                                        current_price_eth,
                                                        |p| format!("  {:.2}", p),
                                                    ))
                                                    .color(Color::hex("#0ECB81"))
                                                    .size(12.0),
                                                ),
                                        )
                                        .child(
                                            CandlestickChart::new(
                                                candles_eth,
                                                pan_x_eth,
                                                pan_y_eth,
                                                zoom_eth,
                                            )
                                            .style(
                                                Style {
                                                    flex_grow: 1.0,
                                                    width: Val::Percent(100.0),
                                                    ..Default::default()
                                                },
                                            ),
                                        ),
                                ),
                        )
                        .child(
                            // Order Book
                            build_order_book(asks, bids),
                        )
                        .child(
                            // Place Order (Signalni uzatamiz)
                            build_place_order(is_buy),
                        ),
                ),
        )
    }

    fn update(&mut self, event: AppEvent) {
        if let AppEvent::Resize(w, h) = event {
            SCREEN_SIZE.with(|s| *s.borrow_mut() = (w, h));
        }
    }
}

fn build_topbar(theme_signal: Signal<Theme>, current_price: Signal<f32>) -> impl Widget {
    UiBox::new()
        .bg_color(Color::hex("#181A20"))
        .style(Style {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            flex_direction: FlexDirection::Row,
            align_items: Align::Center,
            padding: Thickness {
                left: Val::Px(24.0),
                right: Val::Px(24.0),
                ..Default::default()
            },
            gap: Size {
                width: 30.0,
                height: 0.0,
            },
            ..Default::default()
        })
        .child(
            Text::new("BINANCE V2")
                .size(20.0)
                .color(Color::hex("#FCD535")),
        )
        .child(
            Text::new("BTC / USDT")
                .size(20.0)
                .color(Color::hex("#EAECEF")),
        )
        .child(
            UiBox::new()
                .style(Style {
                    flex_direction: FlexDirection::Column,
                    align_items: Align::Start,
                    ..Default::default()
                })
                .child(
                    Text::new(create_selector(current_price, |p| format!("{:.2}", p)))
                        .size(18.0)
                        .color(Color::hex("#0ECB81")),
                )
                .child(
                    Text::new("$68,109.98")
                        .size(12.0)
                        .color(Color::hex("#848E9C")),
                ),
        )
        .child(
            UiBox::new()
                .style(Style {
                    flex_direction: FlexDirection::Column,
                    align_items: Align::Start,
                    ..Default::default()
                })
                .child(
                    Text::new("24h Change")
                        .size(12.0)
                        .color(Color::hex("#848E9C")),
                )
                .child(Text::new("-1.25%").size(14.0).color(Color::hex("#F6465D"))),
        )
        .child(UiBox::new().style(Style {
            flex_grow: 1.0,
            ..Default::default()
        }))
        .child(
            Button::new("btn_theme")
                .child(
                    Text::new("Dark/Light")
                        .color(Color::hex("#848E9C"))
                        .size(14.0),
                )
                .corner_radius(4.0)
                .style(Style {
                    padding: Thickness::all(Val::Px(8.0)),
                    ..Default::default()
                })
                .colors(
                    Color::TRANSPARENT,
                    Color::hex("#2B3139"),
                    Color::hex("#1e293b"),
                )
                .on_click(move || {
                    let is_dark = theme_signal.get_untracked() == Theme::dark();
                    theme_signal.set(if is_dark {
                        Theme::light()
                    } else {
                        Theme::dark()
                    });
                }),
        )
}

fn build_left_toolbar() -> impl Widget {
    UiBox::new().bg_color(Color::hex("#181A20")).style(Style {
        width: Val::Px(48.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: Align::Center,
        padding: Thickness {
            top: Val::Px(16.0),
            ..Default::default()
        },
        gap: Size {
            width: 0.0,
            height: 24.0,
        },
        ..Default::default()
    })
}

fn build_chart_toolbar() -> impl Widget {
    UiBox::new()
        .style(Style {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            flex_direction: FlexDirection::Row,
            align_items: Align::Center,
            padding: Thickness {
                left: Val::Px(16.0),
                right: Val::Px(16.0),
                ..Default::default()
            },
            gap: Size {
                width: 5.0,
                height: 0.0,
            },
            ..Default::default()
        })
        .child(Text::new("Time").color(Color::hex("#848E9C")).size(12.0))
        .child(UiBox::new().style(Style {
            width: Val::Px(10.0),
            ..Default::default()
        }))
        .child(time_btn("15m"))
        .child(time_btn("1H"))
        .child(time_btn("4H"))
        .child(time_btn("1D"))
}

fn time_btn(label: &str) -> Button {
    Button::new(&format!("btn_time_{}", label))
        .child(Text::new(label).color(Color::hex("#848E9C")).size(12.0))
        .corner_radius(4.0)
        .style(Style {
            padding: Thickness {
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                top: Val::Px(4.0),
                bottom: Val::Px(4.0),
            },
            ..Default::default()
        })
        .colors(
            Color::TRANSPARENT,
            Color::hex("#2B3139"),
            Color::hex("#2B3139"),
        )
}

fn build_order_book(asks: Signal<Vec<OrderRow>>, bids: Signal<Vec<OrderRow>>) -> impl Widget {
    let mut container = UiBox::new().bg_color(Color::hex("#181A20")).style(Style {
        width: Val::Px(320.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        ..Default::default()
    });

    container = container.child(
        UiBox::new()
            .style(Style {
                flex_direction: FlexDirection::Row,
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
                        flex_shrink: 0.0,
                        ..Default::default()
                    })
                    .child(
                        Text::new("Price(USDT)")
                            .color(Color::hex("#848E9C"))
                            .size(12.0),
                    ),
            )
            .child(
                UiBox::new()
                    .style(Style {
                        width: Val::Percent(33.3),
                        flex_shrink: 0.0,
                        align_items: Align::End,
                        ..Default::default()
                    })
                    .child(
                        Text::new("Amount(BTC)")
                            .color(Color::hex("#848E9C"))
                            .size(12.0),
                    ),
            )
            .child(
                UiBox::new()
                    .style(Style {
                        width: Val::Percent(33.3),
                        flex_shrink: 0.0,
                        align_items: Align::End,
                        ..Default::default()
                    })
                    .child(Text::new("Total").color(Color::hex("#848E9C")).size(12.0)),
            ),
    );

    for i in 0..14 {
        let p = create_selector(asks, move |a| a.get(i).map(|r| r.price).unwrap_or(0.0));
        let a = create_selector(asks, move |a| a.get(i).map(|r| r.amount).unwrap_or(0.0));
        let t = create_selector(asks, move |a| a.get(i).map(|r| r.total).unwrap_or(0.0));
        container = container.child(build_reactive_row(p, a, t, Color::hex("#F6465D")));
    }

    container = container.child(UiBox::new().style(Style {
        height: Val::Px(15.0),
        ..Default::default()
    }));

    for i in 0..14 {
        let p = create_selector(bids, move |b| b.get(i).map(|r| r.price).unwrap_or(0.0));
        let a = create_selector(bids, move |b| b.get(i).map(|r| r.amount).unwrap_or(0.0));
        let t = create_selector(bids, move |b| b.get(i).map(|r| r.total).unwrap_or(0.0));
        container = container.child(build_reactive_row(p, a, t, Color::hex("#0ECB81")));
    }
    container
}

fn build_reactive_row(
    price: Signal<f32>,
    amount: Signal<f32>,
    total: Signal<f32>,
    color: Color,
) -> impl Widget {
    UiBox::new()
        .style(Style {
            flex_direction: FlexDirection::Row,
            padding: Thickness {
                left: Val::Px(16.0),
                right: Val::Px(16.0),
                top: Val::Px(3.0),
                bottom: Val::Px(3.0),
            },
            ..Default::default()
        })
        .child(
            UiBox::new()
                .style(Style {
                    width: Val::Percent(33.3),
                    flex_shrink: 0.0,
                    ..Default::default()
                })
                .child(
                    Text::new(create_selector(price, |p| format!("{:.2}", p)))
                        .color(color)
                        .size(12.0),
                ),
        )
        .child(
            UiBox::new()
                .style(Style {
                    width: Val::Percent(33.3),
                    flex_shrink: 0.0,
                    align_items: Align::End,
                    ..Default::default()
                })
                .child(
                    Text::new(create_selector(amount, |a| format!("{:.5}", a)))
                        .color(Color::hex("#EAECEF"))
                        .size(12.0),
                ),
        )
        .child(
            UiBox::new()
                .style(Style {
                    width: Val::Percent(33.3),
                    flex_shrink: 0.0,
                    align_items: Align::End,
                    ..Default::default()
                })
                .child(
                    Text::new(create_selector(total, |t| format!("{:.2}", t)))
                        .color(Color::hex("#848E9C"))
                        .size(12.0),
                ),
        )
}

fn build_place_order(is_buy: Signal<bool>) -> impl Widget {
    UiBox::new()
        .bg_color(Color::hex("#181A20"))
        .style(Style {
            // Yon panel hajmi o'zgarmasligi ta'minlandi (Original 300 piksel)
            width: Val::Px(300.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: Thickness::all(Val::Px(16.0)),
            gap: Size {
                width: 0.0,
                height: 16.0,
            },
            ..Default::default()
        })
        .child(
            UiBox::new()
                .style(Style {
                    flex_direction: FlexDirection::Row,
                    gap: Size {
                        width: 16.0,
                        height: 0.0,
                    },
                    ..Default::default()
                })
                .child(Text::new("Spot").size(14.0).color(Color::hex("#FCD535")))
                .child(
                    Text::new("Cross 3x")
                        .size(14.0)
                        .color(Color::hex("#848E9C")),
                ),
        )
        .child(
            UiBox::new()
                .style(Style {
                    flex_direction: FlexDirection::Row,
                    gap: Size {
                        width: 8.0,
                        height: 0.0,
                    },
                    ..Default::default()
                })
                .child(
                    Button::new("btn_buy_tab")
                        .style(Style {
                            flex_grow: 1.0,
                            padding: Thickness::all(Val::Px(10.0)),
                            align_items: Align::Center,
                            ..Default::default()
                        })
                        .colors(
                            create_selector(is_buy, |&b| {
                                if b {
                                    Color::hex("#0ECB81")
                                } else {
                                    Color::hex("#2B3139")
                                }
                            }),
                            create_selector(is_buy, |&b| {
                                if b {
                                    Color::hex("#0b9961")
                                } else {
                                    Color::hex("#3b434f")
                                }
                            }),
                            create_selector(is_buy, |&b| {
                                if b {
                                    Color::hex("#087a4d")
                                } else {
                                    Color::hex("#1e2229")
                                }
                            }),
                        )
                        .corner_radius(4.0)
                        .on_click(move || is_buy.set(true))
                        .child(
                            Text::new("BUY")
                                .size(14.0)
                                .color(create_selector(is_buy, |&b| {
                                    if b {
                                        Color::WHITE
                                    } else {
                                        Color::hex("#848E9C")
                                    }
                                })),
                        ),
                )
                .child(
                    Button::new("btn_sell_tab")
                        .style(Style {
                            flex_grow: 1.0,
                            padding: Thickness::all(Val::Px(10.0)),
                            align_items: Align::Center,
                            ..Default::default()
                        })
                        .colors(
                            create_selector(is_buy, |&b| {
                                if !b {
                                    Color::hex("#F6465D")
                                } else {
                                    Color::hex("#2B3139")
                                }
                            }),
                            create_selector(is_buy, |&b| {
                                if !b {
                                    Color::hex("#c9364a")
                                } else {
                                    Color::hex("#3b434f")
                                }
                            }),
                            create_selector(is_buy, |&b| {
                                if !b {
                                    Color::hex("#a32838")
                                } else {
                                    Color::hex("#1e2229")
                                }
                            }),
                        )
                        .corner_radius(4.0)
                        .on_click(move || is_buy.set(false))
                        .child(
                            Text::new("SELL")
                                .size(14.0)
                                .color(create_selector(is_buy, |&b| {
                                    if !b {
                                        Color::WHITE
                                    } else {
                                        Color::hex("#848E9C")
                                    }
                                })),
                        ),
                ),
        )
        .child(
            UiBox::new()
                .style(Style {
                    flex_direction: FlexDirection::Column,
                    gap: Size {
                        width: 0.0,
                        height: 6.0,
                    },
                    ..Default::default()
                })
                .child(Text::new("Price").size(12.0).color(Color::hex("#848E9C")))
                .child(
                    TextInput::new("input_price")
                        .placeholder("Price USDT")
                        .bg_color(Color::hex("#2B3139"))
                        .text_color(Color::WHITE)
                        .corner_radius(4.0)
                        .style(Style {
                            padding: Thickness::all(Val::Px(12.0)),
                            width: Val::Percent(100.0),
                            height: Val::Px(40.0),
                            ..Default::default()
                        }),
                ),
        )
        .child(
            UiBox::new()
                .style(Style {
                    flex_direction: FlexDirection::Column,
                    gap: Size {
                        width: 0.0,
                        height: 6.0,
                    },
                    ..Default::default()
                })
                .child(Text::new("Amount").size(12.0).color(Color::hex("#848E9C")))
                .child(
                    TextInput::new("input_amount")
                        .placeholder("Amount BTC")
                        .bg_color(Color::hex("#2B3139"))
                        .text_color(Color::WHITE)
                        .corner_radius(4.0)
                        .style(Style {
                            padding: Thickness::all(Val::Px(10.0)),
                            // Bu ham maksimal kenglikni qoplaydi
                            width: Val::Percent(100.0),
                            height: Val::Px(40.0),
                            ..Default::default()
                        }),
                ),
        )
        .child(
            Button::new("btn_buy_execute")
                .style(Style {
                    margin: Thickness {
                        top: Val::Px(16.0),
                        ..Default::default()
                    },
                    padding: Thickness::all(Val::Px(14.0)),
                    width: Val::Percent(100.0),
                    align_items: Align::Center,
                    ..Default::default()
                })
                .corner_radius(4.0)
                .colors(
                    create_selector(is_buy, |&b| {
                        if b {
                            Color::hex("#0ECB81")
                        } else {
                            Color::hex("#F6465D")
                        }
                    }),
                    create_selector(is_buy, |&b| {
                        if b {
                            Color::hex("#0b9961")
                        } else {
                            Color::hex("#c9364a")
                        }
                    }),
                    create_selector(is_buy, |&b| {
                        if b {
                            Color::hex("#087a4d")
                        } else {
                            Color::hex("#a32838")
                        }
                    }),
                )
                .on_click(move || {
                    let side = if is_buy.get_untracked() {
                        "SOTIB OLISH"
                    } else {
                        "SOTISH"
                    };
                    println!("{} BUYRUG'I BOSILDI!", side);
                })
                .child(
                    Text::new(create_selector(is_buy, |&b| {
                        if b {
                            "Buy BTC".to_string()
                        } else {
                            "Sell BTC".to_string()
                        }
                    }))
                    .size(16.0)
                    .color(Color::WHITE),
                ),
        )
}

fn main() {
    let renderer_factory =
        move |device: &_, queue: &_, config: &_| -> Box<dyn rore_types::text::TextRenderer> {
            Box::new(TextSystem::new(device, queue, config))
        };

    run(RoreTradingApp {}, RoreConfig::desktop(), renderer_factory);
}
