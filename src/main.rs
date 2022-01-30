#[macro_use]
mod macros;
pub mod app;
pub mod app_state;
pub mod box_constraints;
pub mod command;
pub mod constraints;
pub mod context;
pub mod event;
pub mod ext_event;
pub mod id;
pub mod key;
pub mod lifecycle;
pub mod menu;
pub mod object;
pub mod perf;
pub mod sliver_constraints;
pub mod text;
pub mod tree;
pub mod ui;
pub mod widgets;
pub mod window;

use app::WindowDesc;
use druid_shell::kurbo::{Point, Size};
use menu::mac::menu_bar;
use widgets::text::TextStyle;

use crate::app::AppLauncher;
use crate::ui::Ui;
use crate::widgets::button::Button;
use crate::widgets::hstack::{HStack, VerticalAlignment};
use crate::widgets::scroll_view::ScrollView;
use crate::widgets::text::Text;

use crate::widgets::vstack::{HorizontalAlignment, VStack};

fn win(ui: &mut Ui) {
    // scroll_view(ui, |ui| {
    vstack(ui, |ui| {
        let count = ui.state_node(|| 0isize);
        // let text_val = ui.state_node(|| "haha".to_string());
        // TextBox::new((*text_val).clone())
        //     .text_size(20.)
        //     .on_changed(move |val| text_val.set(format!("{val}")))
        //     .build(ui);

        let style = ui.state_node(|| TextStyle::default());
        let _i = 1;
        for i in 0..(*count as usize) {
            let count2 = ui.state_node(|| 0isize);
            text(
                ui,
                &format!("label {}, count: {}", i, *count2),
                (*style).clone(),
            );

            button(ui, &format!("button{i}, click to incr"), move || {
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

fn hstack(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    HStack::new(10., VerticalAlignment::Center).build(ui, content);
}

fn text(ui: &mut Ui, text: &str, style: TextStyle) {
    Text::new(text).style(style).build(ui);
}

fn button<'a>(ui: &'a mut Ui<'_>, text: &str, click: impl FnMut() + 'static) {
    Button::new().labeled(ui, text, click);
}

fn main() {
    let desc = WindowDesc::new("app".to_string(), win).menu(|_| menu_bar());
    let app = AppLauncher::with_window(desc).log_to_console();
    app.launch().unwrap();
}
