//! A widget with predefined size.

use std::panic::Location;

use druid_shell::kurbo::Size;

use crate::box_constraints::BoxConstraints;
use crate::constraints::Constraints;
use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx};
use crate::event::Event;
use crate::lifecycle::LifeCycle;
use crate::object::{Properties, RenderObject, RenderObjectInterface};
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
    width: Option<f64>,
    height: Option<f64>,
    clip: bool,
}

impl Properties for SizedBox {
    type Object = SizedBoxObject;
}

impl SizedBox {
    /// Construct container with child, and both width and height not set.
    pub fn new() -> Self {
        Self::default()
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = Location::caller().into();
        ui.render_object(caller, self, content);
    }

    #[track_caller]
    pub fn empty(self, cx: &mut Ui) {
        let caller = Location::caller().into();
        cx.render_object(caller, self, |_| {});
    }

    /// Clip area.
    pub fn clip(mut self) -> Self {
        self.clip = true;
        self
    }

    /// Set container's width.
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// Set container's height.
    pub fn height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    /// Expand container to fit the parent.
    ///
    /// Only call this method if you want your widget to occupy all available
    /// space. If you only care about expanding in one of width or height, use
    /// [`expand_width`] or [`expand_height`] instead.
    ///
    /// [`expand_height`]: #method.expand_height
    /// [`expand_width`]: #method.expand_width
    pub fn expand(mut self) -> Self {
        self.width = Some(f64::INFINITY);
        self.height = Some(f64::INFINITY);
        self
    }

    /// Expand the container on the x-axis.
    ///
    /// This will force the child to have maximum width.
    pub fn expand_width(mut self) -> Self {
        self.width = Some(f64::INFINITY);
        self
    }

    /// Expand the container on the y-axis.
    ///
    /// This will force the child to have maximum height.
    pub fn expand_height(mut self) -> Self {
        self.height = Some(f64::INFINITY);
        self
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SizedBoxObject {
    width: Option<f64>,
    height: Option<f64>,
    clip: bool,
}

impl SizedBoxObject {
    fn child_constraints(&self, bc: &BoxConstraints) -> BoxConstraints {
        // if we don't have a width/height, we don't change that axis.
        // if we have a width/height, we clamp it on that axis.
        let (min_width, max_width) = match self.width {
            Some(width) => {
                let w = width.max(bc.min().width).min(bc.max().width);
                (w, w)
            }
            None => (bc.min().width, bc.max().width),
        };

        let (min_height, max_height) = match self.height {
            Some(height) => {
                let h = height.max(bc.min().height).min(bc.max().height);
                (h, h)
            }
            None => (bc.min().height, bc.max().height),
        };

        BoxConstraints::new(
            Size::new(min_width, min_height),
            Size::new(max_width, max_height),
        )
    }
}

impl RenderObject<SizedBox> for SizedBoxObject {
    type Action = ();

    fn create(props: SizedBox) -> Self {
        SizedBoxObject {
            width: props.width,
            height: props.height,
            clip: props.clip,
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: SizedBox) {
        if self.width != props.width || self.height != props.height || self.clip != props.clip {
            ctx.request_layout();
            self.width = props.width;
            self.height = props.height;
            self.clip = props.clip;
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

    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children) -> Size {
        let bc: BoxConstraints = c.into();
        bc.debug_check("SizedBox");

        let _child_bc = self.child_constraints(&bc);
        let size = match children.get_mut(0) {
            Some(inner) => inner.layout(ctx, c),
            None => bc.constrain((self.width.unwrap_or(0.0), self.height.unwrap_or(0.0))),
        };

        if size.width.is_infinite() {
            tracing::warn!("SizedBox is returning an infinite width.");
        }

        if size.height.is_infinite() {
            tracing::warn!("SizedBox is returning an infinite height.");
        }

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        if !children.is_empty() {
            // let clip_size = ctx.size().to_rect();
            // ctx.clip(clip_size);
            children[0].paint(ctx);
        }
    }
}
