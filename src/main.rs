#[macro_use]
mod macros;
pub mod app;
pub mod app_state;
pub mod box_constraints;
pub mod command;
pub mod constraints;
pub mod context;
mod debug_state;
pub mod event;
pub mod ext_event;
pub mod id;
pub mod key;
pub mod lifecycle;
mod live_style;
pub mod menu;
pub mod object;
pub mod perf;
pub mod sliver_constraints;
mod style;
pub mod text;
pub mod tree;
pub mod ui;
pub mod widgets;
pub mod window;

use app::WindowDesc;
use druid_shell::kurbo::{Insets, Point, Size};

use druid_shell::piet::Color;
use menu::mac::menu_bar;

use live_style::live_style;
use style::alignment::HorizontalAlignment;
use style::{draw, Style};
use widgets::background::Background;
use widgets::flex::{self, Flex};
use widgets::padding::Padding;
use widgets::sized_box::SizedBox;

use crate::app::AppLauncher;
use crate::ui::Ui;
use crate::widgets::button::Button;

use crate::widgets::scroll_view::ScrollView;
use crate::widgets::text::Text;

use crate::widgets::vstack::VStack;

fn win(ui: &mut Ui) {
    // scroll_view(ui, |ui| {
    let flex_style: Style = live_style(ui, ".flex");

    flex(ui, flex_style, |ui| {
        let count = ui.state_node(|| 0isize);
        // let text_val = ui.state_node(|| "haha".to_string());
        // TextBox::new((*text_val).clone())
        //     .text_size(20.)
        //     .on_changed(move |val| text_val.set(format!("{val}")))
        //     .build(ui);

        let style = live_style(ui, ".text");
        let _i = 1;
        for i in 0..(*count as usize) {
            let count2 = ui.state_node(|| 0isize);
            text(
                ui,
                &format!("label {}, count: {}", i, *count2),
                style.clone(),
            );

            button(ui, &format!("button{i}, click to incr你好"), move || {
                println!("click to incr");
                count2.update(|c| *c += 1);
            });
        }

        button(ui, "incr buttons", move || {
            count.update(|c| *c += 1);
            println!("incr buttons");
        });

        button(ui, "decr buttons", move || {
            count.update(|c| {
                if *c > 0 {
                    *c -= 1
                }
            });
            println!("decr buttons");
        });
    });
    // });
}

fn scroll_view(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    ScrollView::new(Point::ZERO, Size::new(600., 400.)).build(ui, content);
}

fn vstack(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    VStack::new(10., HorizontalAlignment::Center).build(ui, content);
}

fn flex(ui: &mut Ui, style: Style, content: impl FnMut(&mut Ui)) {
    Flex::new(style).build(ui, content);
}

fn text(ui: &mut Ui, text: &str, style: Style) {
    Text::new(text).style(style).build(ui);
}

fn button<'a>(ui: &'a mut Ui<'_>, text: &str, click: impl FnMut() + 'static) {
    Button::new().labeled(ui, text, click);
}

fn test(ui: &mut Ui) {
    let style = live_style(ui, ".text");
    Padding::new(Insets::uniform(100.)).build(ui, |ui| button(ui, "incr buttons", move || {}));
}

fn main() {
    let desc = WindowDesc::new("app".to_string(), win).menu(|_| menu_bar());
    let app = AppLauncher::with_window(desc).log_to_console();
    app.launch().unwrap();
}
