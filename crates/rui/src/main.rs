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

use menu::mac::menu_bar;

use live_style::live_style;
use style::Style;

use style::alignment::Alignment;
use widgets::padding::Padding;

use crate::app::AppLauncher;
use crate::ui::Ui;
use crate::widgets::button::Button;

use crate::widgets::scroll_view::ScrollView;
use crate::widgets::text::Text;

fn win(ui: &mut Ui) {
    // scroll_view(ui, |ui| {

    flex(ui, ".flex", |ui| {
        let count = ui.state_node(|| 0isize);
        // let text_val = ui.state_node(|| "haha".to_string());
        // TextBox::new((*text_val).clone())
        //     .text_size(20.)
        //     .on_changed(move |val| text_val.set(format!("{val}")))
        //     .build(ui);

        flexible(ui, ".flexible1", |ui| {
            flex(ui, ".inner-flex", |ui| {
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
            });
        });
        align(ui, |ui| {
            Text::new("hello").build(ui);
        });

        flexible(ui, ".flexible2", |ui| {
            button(ui, "incr buttons", move || {
                count.update(|c| *c += 1);
                println!("incr buttons");
            });
        });
        flexible(ui, ".flexible3", |ui| {
            button(ui, "decr buttons", move || {
                count.update(|c| {
                    if *c > 0 {
                        *c -= 1
                    }
                });
                println!("decr buttons");
            });
        });
    });
    // });
}

fn scroll_view(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    ScrollView::new(Point::ZERO, Size::new(600., 400.)).build(ui, content);
}

fn flex(ui: &mut Ui, style_name: &str, content: impl FnMut(&mut Ui)) {
    let style = live_style(ui, style_name);
    widgets::flex::Flex::new(
        style.axis,
        style.main_axis_size,
        style.main_axis_alignment,
        style.cross_axis_alignment,
        style.text_direction,
        style.vertical_direction,
    )
    .build(ui, content);
}

fn column(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    widgets::flex::Flex::new(
        style::axis::Axis::Vertical,
        style::layout::MainAxisSize::Min,
        style::layout::MainAxisAlignment::Start,
        style::layout::CrossAxisAlignment::Center,
        style::layout::TextDirection::Ltr,
        style::layout::VerticalDirection::Down,
    )
    .build(ui, content);
}

fn row(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    widgets::flex::Flex::new(
        style::axis::Axis::Horizontal,
        style::layout::MainAxisSize::Min,
        style::layout::MainAxisAlignment::Start,
        style::layout::CrossAxisAlignment::Center,
        style::layout::TextDirection::Ltr,
        style::layout::VerticalDirection::Down,
    )
    .build(ui, content);
}

fn flexible(ui: &mut Ui, style_name: &str, content: impl FnMut(&mut Ui)) {
    let style = live_style(ui, style_name);
    let flex = style.flex.value();
    let flex_fit = style.flex_fit;
    widgets::flex::Flexible::new(flex, flex_fit).build(ui, content);
}

fn expand(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    widgets::flex::Flexible::new(1.0, style::layout::FlexFit::Tight).build(ui, content);
}

fn align(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    widgets::align::Align::new(
        Alignment::bottom_center(),
        None,
        None,
        style::layout::TextDirection::Ltr,
    )
    .build(ui, content);
}

fn text(ui: &mut Ui, text: &str, style: Style) {
    Text::new(text).style(style).build(ui);
}

fn button<'a>(ui: &'a mut Ui<'_>, text: &str, click: impl FnMut() + 'static) {
    Button::new().labeled(ui, text, click);
}

fn test(ui: &mut Ui) {
    let _style = live_style(ui, ".text");
    Padding::new(Insets::uniform(100.)).build(ui, |ui| button(ui, "incr buttons", move || {}));
}

fn main() {
    let desc = WindowDesc::new("app".to_string(), win).menu(|_| menu_bar());
    let app = AppLauncher::with_window(desc).log_to_console();
    app.launch().unwrap();
}
