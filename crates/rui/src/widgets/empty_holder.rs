//! A widget with predefined size.

use std::panic::Location;

use druid_shell::kurbo::Size;
use tracing::debug;

use crate::box_constraints::BoxConstraints;

use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx};
use crate::event::Event;
use crate::lifecycle::LifeCycle;
use crate::object::{Properties, RenderObject, RenderObjectInterface};
use crate::style::size::{Height, MaxHeight, MaxWidth, MinHeight, MinWidth, Width};
use crate::tree::Children;
use crate::ui::Ui;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct EmptyHolder;

impl Properties for EmptyHolder {
    type Object = EmptyHolderObject;
}

#[derive(Debug, Default, PartialEq)]
pub struct EmptyHolderObject;

impl RenderObject<EmptyHolder> for EmptyHolderObject {
    type Action = ();

    fn create(props: EmptyHolder) -> Self {
        EmptyHolderObject
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: EmptyHolder, children: &mut Children) {}
}

impl RenderObjectInterface for EmptyHolderObject {}
