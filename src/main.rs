use crate::app::App;
use crate::ui::Ui;
use crate::widgets::label::Label;
use crate::widgets::padding::Padding;
use crate::widgets::sized_box::SizedBox;

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

fn win(ui: &mut Ui) {
    Padding::new((20., 20.)).build(ui, |ui| {
        Label::new("test").build(ui);
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
