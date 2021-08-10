//! A widget that just adds padding during layout.

use std::panic::Location;

use druid_shell::kurbo::{Insets, Point, Size};

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
pub enum Alignment {
    Left,
    Right,
    Center,
}

/// A widget that just adds padding around its child.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Column {
    spacing: f64,
    alignment: Alignment,
}

impl Properties for Column {
    type Object = Self;
}

impl Column {
    pub fn new(spacing: f64, alignment: Alignment) -> Self {
        Column { spacing, alignment }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = Location::caller().into();
        ui.render_object(caller, self, content);
    }
}

impl RenderObject<Column> for Column {
    type Action = ();

    fn create(props: Column) -> Self {
        props
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: Column) {}
}

impl RenderObjectInterface for Column {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        for child in children {
            child.event(ctx, event)
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle) {}

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        bc.debug_check("Column");
        let mut child_bc = bc.clone();
        let mut total_width: f64 = 0.;
        let mut total_height: f64 = 0.;
        for (i, child) in children.iter().enumerate() {
            let size = child.layout(ctx, &child_bc);
            total_height += size.height;
            if i != 0 {
                total_height += self.spacing;
            }
            total_width = total_width.max(size.width);
            child_bc = child_bc.shrink((0.0, total_height));
        }
        match self.alignment {
            Alignment::Left => {
                let mut y = 0.;
                for child in children.iter() {
                    let child_size = child.size();
                    child.set_origin(ctx, Point::new(0., y));
                    y += self.spacing + child_size.height;
                }
            }
            Alignment::Right => {
                let mut y = 0.;
                for child in children.iter() {
                    let child_size = child.size();
                    child.set_origin(ctx, Point::new(total_width - child_size.width, y));
                    y += self.spacing + child_size.height;
                }
            }
            Alignment::Center => {
                let mut y = 0.;
                for child in children.iter() {
                    let child_size = child.size();
                    child.set_origin(ctx, Point::new((total_width - child_size.width) / 2., y));
                    y += self.spacing + child_size.height;
                }
            }
        }
        Size::new(total_width, total_height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        for child in children {
            child.paint(ctx);
        }
    }
}
