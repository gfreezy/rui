use std::panic::Location;

use druid_shell::kurbo::{Point, Size};

use crate::box_constraints::BoxConstraints;
use crate::{
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    lifecycle::LifeCycle,
    object::{Properties, RenderObject, RenderObjectInterface},
    tree::Children,
    ui::Ui,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlignment {
    Bottom,
    Center,
    // FirstTextBaseline,
    // LastTextBaseline,
    Top,
}

/// A widget that just adds padding around its child.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HStack {
    spacing: f64,
    alignment: VerticalAlignment,
}

impl Properties for HStack {
    type Object = Self;
}

impl HStack {
    pub fn new(spacing: f64, alignment: VerticalAlignment) -> Self {
        HStack { spacing, alignment }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = Location::caller().into();
        ui.render_object(caller, self, content);
    }
}

impl RenderObject<HStack> for HStack {
    type Action = ();

    fn create(props: HStack) -> Self {
        props
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: HStack) {
        if self != &props {
            *self = props;
            ctx.request_layout();
        }
    }
}

impl RenderObjectInterface for HStack {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        for child in children {
            child.event(ctx, event)
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        bc.debug_check("HStack");
        let mut child_bc = bc.clone();
        let mut total_width: f64 = 0.;
        let mut total_height: f64 = 0.;
        for (i, child) in children.iter().enumerate() {
            let size = child.layout(ctx, &child_bc);
            total_width += size.width;
            if i != 0 {
                total_width += self.spacing;
            }
            total_height = total_height.max(size.height);
            child_bc = child_bc.shrink((total_width, 0.));
        }

        let mut x = 0.;
        for child in children.iter() {
            let child_size = child.size();

            let y = match self.alignment {
                VerticalAlignment::Bottom => total_height - child_size.height,
                VerticalAlignment::Center => (total_height - child_size.height) / 2.,
                VerticalAlignment::Top => 0.,
            };
            child.set_origin(ctx, Point::new(x, y));

            x += self.spacing + child_size.width;
        }
        Size::new(total_width, total_height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        for child in children {
            child.paint(ctx);
        }
    }
}
