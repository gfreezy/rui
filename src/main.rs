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

use crate::app::App;
use crate::ui::Ui;
use crate::widgets::button::Button;
use crate::widgets::column::{Alignment, Column};
use crate::widgets::label::Label;
use crate::widgets::padding::Padding;

fn win(ui: &mut Ui) {
    Column::new(10., Alignment::Center).build(ui, |ui| {
        Label::new("text").build(ui);
        Padding::new((20., 20.)).build(ui, |ui| {
            Button::new().labeled(ui, "click me", || println!("clicked"));
        });
        Button::new().labeled(ui, "click me 2", || println!("clicked"));
    });
}

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();
    let app = App::new("test");
    app.run(win);
}
