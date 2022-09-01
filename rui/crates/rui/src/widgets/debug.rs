//! A widget with predefined size.

use std::panic::Location;

use druid_shell::kurbo::Size;
use tracing::debug;

use crate::box_constraints::BoxConstraints;

use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx};
use crate::event::Event;
use crate::lifecycle::LifeCycle;
use crate::object::{Properties, RenderObject, RenderObjectInterface};
use crate::tree::Children;
use crate::ui::Ui;
use style::size::{Height, MaxHeight, MaxWidth, MinHeight, MinWidth, Width};

/// A widget with predefined size.
///
/// If given a child, this widget forces its child to have a specific width and/or height
/// (assuming values are permitted by this widget's parent). If either the width or height is not set,
/// this widget will size itself to match the child's size in that dimension.
///
/// If not given a child, SizedBox will try to size itself as close to the specified height
/// and width as possible given the parent's constraints. If height or width is not set,
/// it will be treated as zero.
#[derive(Debug, Default, PartialEq)]
pub struct Debug;

impl Properties for Debug {
    type Object = DebugObject;
}

impl Debug {
    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = crate::key::Key::current();
        ui.render_object(caller, self, content);
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct DebugObject;

impl RenderObject<Debug> for DebugObject {
    type Action = ();

    fn create(props: Debug) -> Self {
        DebugObject
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: Debug) {}
}

impl RenderObjectInterface for DebugObject {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        if !children.is_empty() {
            children[0].event(ctx, event);
        }

    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children) {
        if !children.is_empty() {
            children[0].lifecycle(ctx, event);
        }
    }

    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        bc.debug_check("SizedBox");

        let size = if !children.is_empty() {
            children[0].layout_box(ctx, bc, true)
        } else {
            Size::ZERO
        };
        tracing::debug!("Debug:\n {:#?}", children[0].debug_state());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        if !children.is_empty() {
            children[0].paint(ctx);
        }
    }
}
