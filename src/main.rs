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
use crate::widgets::label::Label;
use druid_shell::piet::Piet;
use druid_shell::{
    Application, HotKey, Menu, MouseEvent, Region, SysMods, WinHandler, WindowBuilder, WindowHandle,
};
use std::any::Any;

fn win(ui: &mut Ui) {
    Label::new("test").build(ui);
}

fn main() {
    let app = App::new("test");
    app.run(win);
}
