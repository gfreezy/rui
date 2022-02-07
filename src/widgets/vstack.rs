//! A widget that just adds padding during layout.

use std::panic::Location;

use druid_shell::kurbo::{Point, Size};

use crate::box_constraints::BoxConstraints;
use crate::constraints::Constraints;

use crate::style::alignment::HorizontalAlignment;
use crate::{
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    lifecycle::LifeCycle,
    object::{Properties, RenderObject, RenderObjectInterface},
    tree::Children,
    ui::Ui,
};

/// A widget that just adds padding around its child.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VStack {
    spacing: f64,
    alignment: HorizontalAlignment,
}

impl Properties for VStack {
    type Object = Self;
}

impl VStack {
    pub fn new(spacing: f64, alignment: HorizontalAlignment) -> Self {
        VStack { spacing, alignment }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = Location::caller().into();
        ui.render_object(caller, self, content);
    }
}

impl RenderObject<VStack> for VStack {
    type Action = ();

    fn create(props: VStack) -> Self {
        props
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: VStack) {
        if self != &props {
            *self = props;
            ctx.request_layout();
        }
    }
}

impl RenderObjectInterface for VStack {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        for child in children {
            child.event(ctx, event)
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children) -> Size {
        let bc: BoxConstraints = c.into();
        bc.debug_check("VStack");
        let mut child_bc = bc.clone();
        let mut total_width: f64 = 0.;
        let mut total_height: f64 = 0.;
        for (i, child) in children.iter().enumerate() {
            let size = child.layout(ctx, &(child_bc.into()));
            total_height += size.height;
            if i != 0 {
                total_height += self.spacing;
            }
            total_width = total_width.max(size.width);
            child_bc = bc.shrink((0.0, total_height));
        }

        let mut y = 0.;
        for child in children.iter() {
            let child_size = child.size();
            let x = match self.alignment {
                HorizontalAlignment::Start => 0.,
                HorizontalAlignment::Center => (total_width - child_size.width) / 2.,
                HorizontalAlignment::End => total_width - child_size.width,
            };

            child.set_origin(ctx, Point::new(x, y));

            y += self.spacing + child_size.height;
        }

        let size = Size::new(total_width, total_height);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        for child in children {
            child.paint(ctx);
        }
    }
}
