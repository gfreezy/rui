#![allow(unused)]
#[macro_use]
mod macros;
pub mod app;
pub mod app_state;
pub mod box_constraints;
pub mod commands;
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
use std::sync::{Arc, Mutex};

use app::WindowDesc;
use commands::{sys, SingleUse, Target};
use crossbeam_channel::{Receiver, Sender};
use debug_state::DebugState;
use druid_shell::kurbo::{Point, Size};

use id::ElementId;
use key::{Key, LocalKey, EMPTY_LOCAL_KEY};
use menu::mac::application::quit;
use menu::mac::menu_bar;
use menu::{Menu, MenuItem};

use live_style::live_style;
use object::{Properties, RenderObject};
use sliver_constraints::{AxisDirection, CacheExtent};
use style::Style;

use style::alignment::Alignment;
use widgets::sized_box::SizedBox;
use widgets::sliver_list::SliverChildDelegate;
use widgets::sliver_to_box;
use widgets::viewport::ViewportOffset;

use crate::app::AppLauncher;
use crate::ui::Ui;
use crate::widgets::button::Button;

use crate::widgets::text::Text;

fn inspect(ui: &mut Ui, snapshot: Arc<Mutex<Snapshot>>) {
    let selected = ui.state_node(|| ElementId::ZERO);

    row(ui, |ui| {
        viewport(ui, AxisDirection::Down, AxisDirection::Right, |ui| {
            let mut data = vec![];
            snapshot.lock().unwrap().debug_state.visit(
                &mut |debug_state, level| {
                    data.push((level, debug_state.clone()));
                },
                0,
            );

            let delegate = VecSliverListDelegate {
                data,
                key_fn: |(level, s)| s.id.to_string(),
                content: move |ui, (level, debug_state)| {
                    let ident = level * 4;
                    let current_id = debug_state.id;
                    button(
                        ui,
                        &format!(
                            "{:ident$}{}(id: {}, len: {})",
                            "",
                            debug_state.display_name,
                            debug_state.id,
                            debug_state.children.len()
                        ),
                        move || {
                            selected.set(current_id);
                        },
                    );
                },
            };

            sliver_list(ui, delegate);
            // snapshot.debug_state.visit(
            //     &mut |debug_state, level| {
            //         sliver_to_box(ui, "center".to_string(), |ui| {
            //             let ident = level * 4;
            //             let current_id = debug_state.id;
            //             button(
            //                 ui,
            //                 &format!(
            //                     "{:ident$}{}(id: {}, len: {})",
            //                     "",
            //                     debug_state.display_name,
            //                     debug_state.id,
            //                     debug_state.children.len()
            //                 ),
            //                 move || {
            //                     selected.set(current_id);
            //                 },
            //             );
            //         });
            //     },
            //     0,
            // );
        });
        viewport(ui, AxisDirection::Down, AxisDirection::Right, |ui| {
            if let Some(debug_state) = snapshot
                .lock()
                .unwrap()
                .debug_state
                .debug_state_for_id(*selected)
            {
                sliver_to_box(ui, "center13".to_string(), |ui| {
                    text(ui, &debug_state.to_string(), Default::default());
                });
            }
        });
    });
}

fn win(ui: &mut Ui, snapshot: Arc<Mutex<Snapshot>>) {
    column(ui, |ui| {
        expand(ui, |ui| {
            let style = live_style(ui, ".text");
            text(ui, "haha", style);
        });

        expand(ui, |ui| {
            // debug(ui, |ui| {
            viewport(ui, AxisDirection::Down, AxisDirection::Right, |ui| {
                for i in 0..10 {
                    sliver_to_box(ui, i.to_string(), |ui| {
                        let style = live_style(ui, ".text");
                        text(ui, &format!("hello{}", i), style);
                    });
                }
                sliver_list(
                    ui,
                    Delegate {
                        center: EMPTY_LOCAL_KEY.to_string(),
                    },
                )
            })
            // });
        });
    });

    snapshot.lock().unwrap().debug_state = ui.tree[0].debug_state();
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
    Button::new()
        .text_align(druid_shell::piet::TextAlignment::Start)
        .labeled(ui, text, click);
}

fn viewport(
    ui: &mut Ui,
    axis_direction: AxisDirection,
    cross_axis_direction: AxisDirection,
    content: impl FnMut(&mut Ui),
) {
    widgets::viewport::Viewport::new(
        axis_direction,
        cross_axis_direction,
        0.0,
        None,
        CacheExtent::Viewport(1.),
    )
    .build(ui, content)
}

fn sliver_to_box(ui: &mut Ui, local_key: String, content: impl FnMut(&mut Ui)) {
    widgets::sliver_to_box::SliverToBox.build(ui, local_key, content);
}

struct VecSliverListDelegate<T, C: FnMut(&mut Ui, &T) + 'static> {
    data: Vec<T>,
    key_fn: fn(&T) -> String,
    content: C,
}

impl<T: PartialEq + 'static, C: FnMut(&mut Ui, &T) + 'static> SliverChildDelegate
    for VecSliverListDelegate<T, C>
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn key(&self, index: usize) -> LocalKey {
        (self.key_fn)(&self.data[index])
    }

    fn build(&mut self, ui: &mut Ui, index: usize) {
        (self.content)(ui, &self.data[index])
    }

    fn estimated_count(&self) -> Option<usize> {
        Some(self.data.len())
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
        let old = old_delegate.as_any().downcast_ref::<Self>().unwrap();
        &old.data != &self.data
    }

    fn find_index_by_key(&self, key: &LocalKey) -> Option<usize> {
        self.data
            .iter()
            .position(|item| &(self.key_fn)(item) == key)
    }
}

struct Delegate {
    center: String,
}

impl SliverChildDelegate for Delegate {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn key(&self, index: usize) -> String {
        index.to_string()
    }

    fn build(&mut self, ui: &mut Ui, index: usize) {
        // tracing::debug!("build in delegate: {index}");
        let style = live_style(ui, ".inspect-text");
        button(ui, &format!("number {index}"), || {});
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

    fn find_index_by_key(&self, key: &LocalKey) -> Option<usize> {
        key.parse().ok()
    }

    fn did_finish_layout(&self, first_index: usize, last_index: usize) {}
}

fn sliver_list(ui: &mut Ui, delegate: impl SliverChildDelegate + 'static) {
    widgets::sliver_list::SliverList::new(Box::new(delegate)).build(ui)
}

#[derive(Debug)]
struct Snapshot {
    debug_state: DebugState,
}

fn main() {
    let snapshot = Arc::new(Mutex::new(Snapshot {
        debug_state: DebugState::default(),
    }));
    let snapshot1 = snapshot.clone();
    let desc = WindowDesc::new("app".to_string(), move |ui| win(ui, snapshot1.clone()))
        .menu(|_| menu_bar());
    let inspector_desc =
        WindowDesc::new("app".to_string(), move |ui| inspect(ui, snapshot.clone()))
            .menu(|_| menu_bar());
    let app = AppLauncher::with_windows(vec![desc, inspector_desc]).log_to_console();
    app.launch().unwrap();
}
