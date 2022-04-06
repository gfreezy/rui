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
pub struct SizedBox {
    width: Width,
    height: Height,
    min_width: MinWidth,
    max_width: MaxWidth,
    min_height: MinHeight,
    max_height: MaxHeight,
    clip: bool,
}

impl Properties for SizedBox {
    type Object = SizedBoxObject;
}

impl SizedBox {
    /// Construct container with child, and both width and height not set.
    pub fn new(
        width: Width,
        height: Height,
        min_width: MinWidth,
        max_width: MaxWidth,
        min_height: MinHeight,
        max_height: MaxHeight,
    ) -> Self {
        SizedBox {
            width,
            height,
            min_width,
            max_width,
            min_height,
            max_height,
            clip: true,
        }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = crate::key::Key::current();
        ui.render_object(caller, self, content);
    }

    /// Clip area.
    pub fn clip(mut self) -> Self {
        self.clip = true;
        self
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SizedBoxObject {
    props: SizedBox,
}

fn child_constraints(
    c: &BoxConstraints,
    fixed_width: f64,
    fixed_height: f64,
    min_width: f64,
    min_height: f64,
    max_width: f64,
    max_height: f64,
) -> BoxConstraints {
    let mut min_size = c.min();
    let mut max_size = c.max();

    if !min_width.is_nan() {
        min_size.width = min_width;
    }
    if !min_height.is_nan() {
        min_size.height = min_height;
    }
    if !max_width.is_nan() {
        max_size.width = max_width;
    }
    if !max_height.is_nan() {
        max_size.height = max_height;
    }
    if !fixed_width.is_nan() {
        min_size.width = fixed_width;
        max_size.width = fixed_width;
    }
    if !fixed_height.is_nan() {
        min_size.height = fixed_height;
        max_size.height = fixed_height;
    }
    BoxConstraints::new(min_size, max_size)
}

impl RenderObject<SizedBox> for SizedBoxObject {
    type Action = ();

    fn create(props: SizedBox) -> Self {
        SizedBoxObject { props }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: SizedBox, children: &mut Children) {
        if &self.props != &props {
            ctx.request_layout();
            self.props = props;
            debug!("request layout");
        }
    }
}

impl RenderObjectInterface for SizedBoxObject {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        if !children.is_empty() {
            children[0].event(ctx, event);
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn dry_layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        bc.debug_check("SizedBox");
        let props = &self.props;
        let new_bc = child_constraints(
            &bc,
            props.width.into(),
            props.height.into(),
            props.min_width.into(),
            props.min_height.into(),
            props.max_width.into(),
            props.max_height.into(),
        );
        let child_bc = new_bc.loosen();
        let size = if !children.is_empty() {
            children[0].dry_layout_box(ctx, &(child_bc.into()))
        } else {
            Size::ZERO
        };
        let mut new_size = new_bc.constrain(size);
        if props.width.is_normal() {
            new_size.width = props.width.value();
        }
        if props.height.is_normal() {
            new_size.height = props.height.value();
        }
        new_size
    }

    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        bc.debug_check("SizedBox");
        let props = &self.props;
        let new_bc = child_constraints(
            &bc,
            props.width.into(),
            props.height.into(),
            props.min_width.into(),
            props.min_height.into(),
            props.max_width.into(),
            props.max_height.into(),
        );
        let child_bc = new_bc.loosen();
        let size = if !children.is_empty() {
            children[0].layout_box(ctx, &(child_bc.into()))
        } else {
            Size::ZERO
        };
        let mut new_size = new_bc.constrain(size);
        if props.width.is_normal() {
            new_size.width = props.width.value();
        }
        if props.height.is_normal() {
            new_size.height = props.height.value();
        }
        new_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        if !children.is_empty() {
            // let clip_size = ctx.size().to_rect();
            // ctx.clip(clip_size);
            children[0].paint(ctx);
        }
    }
}
