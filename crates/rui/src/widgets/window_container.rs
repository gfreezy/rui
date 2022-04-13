use std::time::Instant;

use druid_shell::kurbo::{Point, Size};

use crate::{
    box_constraints::BoxConstraints,
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx},
    event::Event,
    lifecycle::LifeCycle,
    object::RenderObjectInterface,
    tree::Children,
};

pub(crate) struct WindowContainer;

const DEBUG: bool = false;

impl RenderObjectInterface for WindowContainer {
    fn sized_by_parent(&self) -> bool {
        true
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        let instant = Instant::now();

        for child in children {
            child.event(ctx, event);
        }

        if DEBUG {
            tracing::debug!(
                "windowcontainer event took {} us",
                instant.elapsed().as_micros()
            );
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children) {
        let instant = Instant::now();

        for child in children {
            child.lifecycle(ctx, event)
        }

        if DEBUG {
            tracing::debug!(
                "windowcontainer lifecycle took {} us",
                instant.elapsed().as_micros()
            );
        }
    }

    fn dry_layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        let instant = Instant::now();

        let mut size = Size::ZERO;
        for child in children {
            let child_size = child.dry_layout_box(ctx, bc);
            size = Size::new(
                child_size.width.max(size.width),
                child_size.height.max(size.height),
            );
        }
        if DEBUG {
            tracing::debug!(
                "windowcontainer dry_layout_box took {} us",
                instant.elapsed().as_micros()
            );
        }
        size
    }

    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        let instant = Instant::now();
        for child in children {
            let child_size = child.layout_box(ctx, bc, false);
            child.set_origin(ctx, Point::ZERO);
        }

        if DEBUG {
            tracing::debug!(
                "windowcontainer layout_box took {} us",
                instant.elapsed().as_micros()
            );
        }
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        let instant = Instant::now();

        for child in children {
            child.paint(ctx)
        }

        if DEBUG {
            tracing::debug!(
                "windowcontainer paint took {} us",
                instant.elapsed().as_micros()
            );
        }
    }
}
