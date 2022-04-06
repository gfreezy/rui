//! A widget with predefined size.

use std::panic::Location;

use druid_shell::kurbo::Size;
use tracing::debug;

use crate::box_constraints::BoxConstraints;

use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx};
use crate::event::Event;
use crate::key::LocalKey;
use crate::lifecycle::LifeCycle;
use crate::object::{Properties, RenderObject, RenderObjectInterface};
use crate::style::size::{Height, MaxHeight, MaxWidth, MinHeight, MinWidth, Width};
use crate::tree::Children;
use crate::ui::Ui;

use super::sliver_list_parent_data::SliverListParentData;

#[derive(Debug, Clone)]
pub struct SliverListItem {
    pub(crate) local_key: LocalKey,
    pub(crate) index: usize,
    pub(crate) child_index: usize,
    pub(crate) parent_data: SliverListParentData,
}

impl Properties for SliverListItem {
    type Object = SliverListItemObject;
}

#[derive(Debug, Default, PartialEq)]
pub struct SliverListItemObject;

impl RenderObject<SliverListItem> for SliverListItemObject {
    type Action = ();

    fn create(props: SliverListItem) -> Self {
        SliverListItemObject
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: SliverListItem, children: &mut Children) {}
}

impl RenderObjectInterface for SliverListItemObject {
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
        bc.debug_check("SliverListItemObject");

        let size = if !children.is_empty() {
            children[0].layout_box(ctx, bc)
        } else {
            Size::ZERO
        };
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        if !children.is_empty() {
            children[0].paint(ctx);
        }
    }
}
