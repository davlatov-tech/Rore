use std::cell::RefCell;

use rore_core::app::{run, App, AppEvent, Widget};
// XATO TUZATILDI: Ishlatilmagan create_selector olib tashlandi
use rore_core::reactive::signals::{provide_context, Signal};

use rore_text::text::TextSystem;
use rore_text::widgets::theme::Theme;

use rore_text::widgets::{Button, HBox, Spacer, Text, TextInput, UiBox, VBox};

use rore_core::widgets::portal::Portal;
use rore_core::widgets::show::Show;
use rore_text::widgets::scroll_view::ScrollView;

use rore_types::{Align, Color, LayoutModifiers, RoreConfig, Style, Thickness, Val};

thread_local! {
    static SCREEN_SIZE: RefCell<(f32, f32)> = RefCell::new((1024.0, 768.0));
}

struct RoreDashboardApp {}

impl App for RoreDashboardApp {
    fn view(&self) -> Box<dyn Widget> {
        let theme_signal = Signal::new(Theme::dark());
        provide_context(theme_signal);

        let main_scroll = Signal::new(0.0);
        let is_dropdown_open = Signal::new(false);

        let mut settings_list = VBox::new().gap(12.0).width(Val::Percent(100.0));
        for i in 1..=40 {
            settings_list = settings_list.child(
                HBox::new()
                    .bg_color(Color::hex("#181A20"))
                    .corner_radius(8.0)
                    .padding(16.0)
                    .gap(20.0)
                    .modify_style(|s| s.align_items = Align::Center)
                    .child(
                        VBox::new()
                            .width(200.0)
                            .gap(4.0)
                            .child(
                                Text::new(format!("Sozlama bloki #{}", i))
                                    .color(Color::WHITE)
                                    .size(16.0),
                            )
                            .child(
                                Text::new("Ushbu maydonni o'zgartiring")
                                    .color(Color::hex("#848E9C"))
                                    .size(12.0),
                            ),
                    )
                    .child(
                        TextInput::new(&format!("input_setting_{}", i))
                            .placeholder("Qiymat kiriting...")
                            .bg_color(Color::hex("#2B3139"))
                            .text_color(Color::WHITE)
                            .corner_radius(6.0)
                            .width(Val::Percent(100.0))
                            .height(40.0)
                            .padding(10.0)
                            .expand(),
                    )
                    .child(
                        Button::new(&format!("btn_save_{}", i))
                            .colors(
                                Color::hex("#0ECB81"),
                                Color::hex("#0b9961"),
                                Color::hex("#087a4d"),
                            )
                            .corner_radius(6.0)
                            .padding(10.0)
                            .width(100.0)
                            .center()
                            .child(Text::new("Saqlash").color(Color::WHITE).size(14.0)),
                    ),
            );
        }

        Box::new(
            VBox::new()
                .bg_color(Color::hex("#0B0E11"))
                .width(Val::Percent(100.0))
                .height(Val::Percent(100.0))
                .child(
                    HBox::new()
                        .height(65.0)
                        .bg_color(Color::hex("#181A20"))
                        .modify_style(|s| {
                            s.padding.left = Val::Px(30.0);
                            s.padding.right = Val::Px(30.0);
                            s.align_items = Align::Center;
                        })
                        .child(Text::new("RORE").size(24.0).color(Color::hex("#FCD535")))
                        .child(Text::new(" UI DASHBOARD").size(20.0).color(Color::WHITE))
                        .child(Spacer::new())
                        .child(
                            Button::new("profile_btn")
                                .colors(Color::hex("#2B3139"), Color::hex("#3b434f"), Color::hex("#1e2229"))
                                .corner_radius(8.0)
                                .style(Style {
                                    padding: Thickness { left: Val::Px(16.0), right: Val::Px(16.0), top: Val::Px(8.0), bottom: Val::Px(8.0) },
                                    ..Default::default()
                                })
                                .on_click(move || is_dropdown_open.set(!is_dropdown_open.get_untracked()))
                                .child(Text::new("Mening Profilim ▼").color(Color::WHITE))
                        )
                        .child(
                            Show::new(
                                is_dropdown_open,
                                move || Box::new(
                                    Portal::new("profile_btn")
                                        .on_close(move || is_dropdown_open.set(false))
                                        .child(
                                            VBox::new()
                                                .bg_color(Color::hex("#1E293B"))
                                                .corner_radius(8.0)
                                                .padding(8.0)
                                                .width(200.0)
                                                .gap(5.0)

                                                .style(Style {
                                                    margin: Thickness { top: Val::Px(10.0), left: Val::Px(-60.0), ..Default::default() },
                                                    ..Default::default()
                                                })
                                                .child(menu_item("Sozlamalar"))
                                                .child(menu_item("Hamyon"))
                                                .child(menu_item("Xavfsizlik"))
                                                .child(UiBox::new().height(1.0).bg_color(Color::hex("#334155")))
                                                .child(menu_item("Chiqish").color(Color::hex("#F6465D")))
                                        )
                                ),
                                || Box::new(UiBox::new())
                            )
                        )
                )
                .child(
                    ScrollView::new()
                        .scroll_y(main_scroll)
                        // XATO TUZATILDI: expand() o'rniga to'g'ridan-to'g'ri style beramiz
                        .style(Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..Default::default()
                        })
                        .child(
                            VBox::new()
                                .padding(40.0)
                                .gap(40.0)
                                .child(Text::new("1. Zamonaviy Input Maydonlari").size(20.0).color(Color::WHITE))
                                .child(
                                    HBox::new().gap(30.0)
                                        .child(
                                            VBox::new().gap(10.0).width(350.0)
                                                .child(Text::new("Single-line (Parol / Qidiruv)").color(Color::hex("#848E9C")).size(14.0))
                                                .child(
                                                    TextInput::new("inp_search")
                                                        .placeholder("Matn kiriting (Enter pastga tushirmaydi)...")
                                                        .bg_color(Color::hex("#181A20"))
                                                        .corner_radius(8.0)
                                                        .width(Val::Percent(100.0))
                                                        .height(45.0)
                                                        .padding(12.0)
                                                )
                                        )
                                        .child(
                                            VBox::new().gap(10.0).width(400.0)
                                                .child(Text::new("Auto-Grow & Multi-line (Xabar yozish)").color(Color::hex("#848E9C")).size(14.0))
                                                .child(
                                                    TextInput::new("inp_msg")
                                                        .multiline(true)
                                                        .placeholder("Juda uzun xabar yozing... quti avtomat o'sadi!")
                                                        .bg_color(Color::hex("#181A20"))
                                                        .corner_radius(8.0)
                                                        .width(Val::Percent(100.0))
                                                        .height(Val::Auto)
                                                        .padding(12.0)
                                                )
                                        )
                                )
                                .child(Text::new("2. Kinetik Skroll (Scrollbar va 40 ta formalar)").size(20.0).color(Color::WHITE))
                                .child(settings_list)
                        )
                )
        )
    }

    fn update(&mut self, event: AppEvent) {
        if let AppEvent::Resize(w, h) = event {
            SCREEN_SIZE.with(|s| *s.borrow_mut() = (w, h));
        }
    }
}

fn menu_item(label: &str) -> Text {
    Text::new(label)
        .color(Color::WHITE)
        .size(14.0)
        .style(Style {
            padding: Thickness::all(Val::Px(10.0)),
            ..Default::default()
        })
}

fn main() {
    let renderer_factory =
        move |device: &_, queue: &_, config: &_| -> Box<dyn rore_types::text::TextRenderer> {
            Box::new(TextSystem::new(device, queue, config))
        };

    run(RoreDashboardApp {}, RoreConfig::desktop(), renderer_factory);
}
