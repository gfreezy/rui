#![allow(unused)]
mod inspect_state;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use inspect_state::ExpandedState;
use rui::app::WindowDesc;

use rui::debug_state::DebugState;

use rui::id::ElementId;
use rui::key::{LocalKey, EMPTY_LOCAL_KEY};

use rui::menu::mac::menu_bar;

use rui::live_style::live_style;

use rui::sliver_constraints::CacheExtent;
use rui::tree::State;
use rui::{live_s, widgets};
use style::layout::AxisDirection;
use style::Style;

use rui::widgets::sliver_list::SliverChildDelegate;

use style::alignment::Alignment;

use rui::app::AppLauncher;
use rui::prelude::*;
use rui::ui::Ui;
use rui::widgets::button::Button;

use rui::widgets::text::Text;

fn inspect(ui: &mut Ui, snapshot: Arc<Mutex<Snapshot>>) {
    let selected = ui.state_node(|| ElementId::ZERO);

    row(ui, live_s!(ui, ""), |ui| {
        viewport(ui, live_s!(ui, ""), |ui| {
            let mut expanded = ui.state_node(|| ExpandedState::default());
            let root_state = snapshot.lock().unwrap();
            let mut data = inspect_state::InspectDebugState::new(&root_state.debug_state);
            let rows = data.flatten(
                |debug_state, _| expanded.expanded(debug_state.id),
                |debug_state, level| {
                    let ident = level * 4;
                    let symbol = match (
                        debug_state.has_children(),
                        expanded.expanded(debug_state.id),
                    ) {
                        (true, true) => '-',
                        (true, false) => '+',
                        (false, _) => ' ',
                    };
                    (
                        debug_state.id,
                        format!(
                            "{:ident$}{} {}(id: {}, len: {})",
                            " ",
                            symbol,
                            debug_state.display_name,
                            debug_state.id,
                            debug_state.children.len()
                        ),
                    )
                },
            );

            let delegate = VecSliverListDelegate {
                data: rows,
                key_fn: |(id, row)| id.to_string().into(),
                content: move |ui, &(id, ref row)| {
                    button(
                        ui,
                        &row,
                        move || {
                            selected.set(id);
                            expanded.toggle(id);
                        },
                        live_s!(ui, ""),
                    );
                },
            };

            sliver_list(ui, delegate);
        });

        viewport(ui, live_s!(ui, ""), |ui| {
            if let Some(debug_state) = snapshot
                .lock()
                .unwrap()
                .debug_state
                .debug_state_for_id(*selected)
            {
                sliver_to_box(ui, "center13".into(), |ui| {
                    text(
                        ui,
                        &debug_state.to_string(),
                        live_s!(
                            ui,
                            r#".s {
                            font-size: 18;
                            color: rgb(10, 20, 30);
                        }"#
                        ),
                    );
                });
            }
        });
    });
}

fn win(ui: &mut Ui, snapshot: Arc<Mutex<Snapshot>>) {
    column(
        ui,
        live_s!(
            ui,
            r#".style {
                axis: horizontal;
                main-axis-alignment: center;
                cross-axis-alignment: center;
            }
        "#
        ),
        |ui| {
            expand(ui, |ui| {
                text(
                    ui,
                    "haha",
                    live_s!(
                        ui,
                        r#".text {
                            font-size: 30;
                 color: rgb(0, 10, 10);
                }"#
                    ),
                );
            });

            expand(ui, |ui| {
                viewport(ui, live_s!(ui, ""), |ui| {
                    for i in 0..10usize {
                        sliver_to_box(ui, i.to_string().into(), |ui| {
                            let style = live_s!(
                                ui,
                                r#"
                            .text {
                            font-size: 30;
                            color: rgb(43, 10, 10);
                        }"#
                            );
                            text(ui, &format!("hello{}", i), style);
                        });
                    }
                    sliver_list(
                        ui,
                        Delegate {
                            center: EMPTY_LOCAL_KEY.into(),
                        },
                    )
                })
            });
        },
    );

    snapshot.lock().unwrap().debug_state = ui.tree[0].debug_state();
}

struct VecSliverListDelegate<T: 'static, C: FnMut(&mut Ui, &T) + 'static> {
    data: Vec<T>,
    key_fn: fn(&T) -> LocalKey,
    content: C,
}

impl<T: PartialEq + Debug + 'static, C: FnMut(&mut Ui, &T) + 'static> SliverChildDelegate
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
        _sc: &rui::constraints::SliverConstraints,
        _first_index: usize,
        _last_index: usize,
        _leading_scroll_offset: f64,
        _trailing_scroll_offset: f64,
    ) -> Option<f64> {
        None
    }

    fn should_rebuild(&self, old_delegate: &dyn SliverChildDelegate) -> bool {
        let old = old_delegate.as_any().downcast_ref::<Self>().unwrap();
        let should_rebuild = &old.data != &self.data;
        if should_rebuild {
            tracing::trace!("should_rebuild: {:?} {:?}", &self.data, &old.data);
        }
        should_rebuild
    }

    fn find_index_by_key(&self, key: &LocalKey) -> Option<usize> {
        self.data
            .iter()
            .position(|item| &(self.key_fn)(item) == key)
    }
}

struct Delegate {
    center: LocalKey,
}

impl SliverChildDelegate for Delegate {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn key(&self, index: usize) -> LocalKey {
        index.to_string().into()
    }

    fn build(&mut self, ui: &mut Ui, index: usize) {
        button(ui, &format!("number {index}"), || {}, live_s!(ui, ""));
    }

    fn estimated_count(&self) -> Option<usize> {
        // tracing::debug!("estimated count");
        Some(100)
    }

    fn estimate_max_scroll_offset(
        &self,
        _sc: &rui::constraints::SliverConstraints,
        _first_index: usize,
        _last_index: usize,
        _leading_scroll_offset: f64,
        _trailing_scroll_offset: f64,
    ) -> Option<f64> {
        None
    }

    fn should_rebuild(&self, _old_delegate: &dyn SliverChildDelegate) -> bool {
        false
    }

    fn find_index_by_key(&self, key: &LocalKey) -> Option<usize> {
        key.parse().ok()
    }

    fn did_finish_layout(&self, _first_index: usize, _last_index: usize) {}
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
