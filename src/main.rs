#[macro_use]
mod macros;
pub mod app;
pub mod box_constraints;
pub mod context;
pub mod event;
pub mod id;
pub mod key;
pub mod lifecycle;
pub mod object;
pub mod text;
pub mod tree;
pub mod ui;
pub mod widgets;

use druid_shell::kurbo::{Point, Size};

use crate::app::App;
use crate::ui::Ui;
use crate::widgets::button::Button;
use crate::widgets::hstack::{HStack, VerticalAlignment};
use crate::widgets::scroll_view::ScrollView;
use crate::widgets::text::Text;
use crate::widgets::textbox::TextBox;
use crate::widgets::vstack::{HorizontalAlignment, VStack};

fn win(ui: &mut Ui) {
    scroll_view(ui, |ui| {
        vstack(ui, |ui| {
            let count = ui.state_node(|| 0isize);

            let mut t = "haha".to_string();
            TextBox::new(&mut t).build(ui);

            for _ in 0..(*count.get() as usize) {
                let count = ui.state_node(|| 0isize);
                text(ui, &format!("count: {}", *count.get()));

                button(
                    ui,
                    "click to incr",
                    clone!([count] move || {
                            count.update(|c| *c += 1);
                        }
                    ),
                );
            }

            button(
                ui,
                "incr buttons",
                clone!([count] move || {
                    count.update(|c| *c += 1);
                }),
            );
            button(
                ui,
                "decr buttons",
                clone!([count] move || {
                    count.update(|c| {
                        if *c > 0 {
                            *c -= 1
                        }
                    });
                }),
            );
        });
    });

    // println!("{:?}", &ui.tree.renders[0].children.tracked_states);
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

fn text(ui: &mut Ui, text: &str) {
    Text::new(text).text_size(20.).build(ui);
}

fn button(ui: &mut Ui, text: &str, click: impl FnMut() + 'static) {
    Button::new().labeled(ui, text, click);
}

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();
    let app = App::new("test");
    app.run(win);
}
