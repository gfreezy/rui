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
mod physics;
pub mod sliver_constraints;
mod style;
pub mod text;
pub mod tree;
pub mod ui;
pub mod widgets;
pub mod window;

use std::any::Any;
use std::panic::Location;

use app::WindowDesc;
use druid_shell::kurbo::{Point, Size};

use key::{Key, LocalKey};
use menu::mac::menu_bar;

use live_style::live_style;
use object::{Properties, RenderObject};
use sliver_constraints::{AxisDirection, CacheExtent};
use style::Style;

use style::alignment::Alignment;
use widgets::sized_box::SizedBox;
use widgets::sliver_list::SliverChildDelegate;
use widgets::viewport::ViewportOffset;

use crate::app::AppLauncher;
use crate::ui::Ui;
use crate::widgets::button::Button;

use crate::widgets::scroll_view::ScrollView;
use crate::widgets::text::Text;

fn win(ui: &mut Ui) {
    // scroll_view(ui, |ui| {
    //     column(ui, |ui| {
    //         for i in 0..40 {
    //             let style = live_style(ui, ".text");
    //             text(ui, &format!("haha {i}"), style);
    //         }
    //     });
    // });
    // flex(ui, ".flex", |ui| {
    //     let count = ui.state_node(|| 0isize);

    //     flexible(ui, ".flexible1", |ui| {
    //         flex(ui, ".inner-flex", |ui| {
    //             let style = live_style(ui, ".text");
    //             let _i = 1;
    //             for i in 0..(*count as usize) {
    //                 let count2 = ui.state_node(|| 0isize);

    //                 text(
    //                     ui,
    //                     &format!("label {}, count: {}", i, *count2),
    //                     style.clone(),
    //                 );

    //                 button(ui, &format!("button{i}, click to incr你好"), move || {
    //                     println!("click to incr");
    //                     count2.update(|c| *c += 1);
    //                 });
    //             }
    //         });
    //     });

    //     flexible(ui, ".flexible2", |ui| {
    //         // align(ui, |ui| {
    //         //     Text::new("hello").build(ui);
    //         // });

    //         button(ui, "incr buttons", move || {
    //             count.update(|c| *c += 1);
    //             println!("incr buttons");
    //         });
    //     });
    //     flexible(ui, ".flexible3", |ui| {
    //         button(ui, "decr buttons", move || {
    //             count.update(|c| {
    //                 if *c > 0 {
    //                     *c -= 1
    //                 }
    //             });
    //             println!("decr buttons");
    //         });
    //     });
    // });
    // });
    flex(ui, ".flex", |ui| {
        expand(ui, |ui| {
            let style = live_style(ui, ".text");
            text(ui, "haha", style);
        });

        expand(ui, |ui| {
            // debug(ui, |ui| {
            viewport(
                ui,
                AxisDirection::Down,
                AxisDirection::Right,
                "0".to_string(),
                |ui| {
                    for i in 0..10 {
                        widgets::sliver_to_box::SliverToBox.build(ui, i.to_string(), |ui| {
                            let style = live_style(ui, ".text");
                            text(ui, &format!("hello{}", i), style);
                        });
                    }
                },
            )
            // });
        });
    });
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

fn debug(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    widgets::debug::Debug.build(ui, content);
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

fn viewport(
    ui: &mut Ui,
    axis_direction: AxisDirection,
    cross_axis_direction: AxisDirection,
    center: LocalKey,
    content: impl FnMut(&mut Ui),
) {
    widgets::viewport::Viewport::new(
        axis_direction,
        cross_axis_direction,
        0.0,
        Some(center),
        CacheExtent::Viewport(1.),
    )
    .build(ui, content)
}

struct Delegate {
    center: Key,
}

impl SliverChildDelegate for Delegate {
    fn key(&self, index: usize) -> Key {
        if index == 0 {
            self.center.clone()
        } else {
            Location::caller().into()
        }
    }

    fn build(&self, ui: &mut Ui, index: usize) {
        // tracing::debug!("build in delegate");
        let style = live_style(ui, ".text");
        text(ui, &format!("number {index}"), style);
    }

    fn estimated_count(&self) -> Option<usize> {
        // tracing::debug!("estimated count");
        Some(100)
    }

    fn estimate_max_scroll_offset(
        &self,
        sc: &constraints::SliverConstraints,
        first_index: usize,
        last_index: usize,
        leading_scroll_offset: f64,
        trailing_scroll_offset: f64,
    ) -> Option<f64> {
        None
    }

    fn should_rebuild(&self, old_delegate: &dyn SliverChildDelegate) -> bool {
        false
    }
}

fn sliver_list(ui: &mut Ui, center: Key) {
    widgets::sliver_list::SliverList::new(Box::new(Delegate { center })).build(ui)
}

fn main() {
    let desc = WindowDesc::new("app".to_string(), win).menu(|_| menu_bar());
    let app = AppLauncher::with_window(desc).log_to_console();
    app.launch().unwrap();
}
