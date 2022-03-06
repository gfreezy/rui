use druid_shell::kurbo::{Point, Size};

use crate::{
    constraints::Constraints,
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx},
    event::Event,
    lifecycle::LifeCycle,
    object::{RenderObject, RenderObjectInterface},
    tree::Children,
};

pub(crate) struct WindowContainer;

impl RenderObject for WindowContainer {
    type Props = ();

    type Action = ();

    fn create(_props: Self::Props) -> Self {
        Self
    }

    fn update(
        &mut self,
        _ctx: &mut crate::context::UpdateCtx,
        _propss: Self::Props,
    ) -> Self::Action {
        ()
    }
}
impl RenderObjectInterface for WindowContainer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        for child in children {
            child.event(ctx, event);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children) {
        for child in children {
            child.lifecycle(ctx, event)
        }
    }

    fn dry_layout(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &Constraints,
        children: &mut Children,
    ) -> Size {
        let mut size = Size::ZERO;
        for child in children {
            let child_size = child.dry_layout(ctx, c);
            size = Size::new(
                child_size.width.max(size.width),
                child_size.height.max(size.height),
            );
        }
        size
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children) -> Size {
        let mut size = Size::ZERO;
        for child in children {
            let child_size = child.layout(ctx, c);
            child.set_origin(ctx, Point::ZERO);
            size = Size::new(
                child_size.width.max(size.width),
                child_size.height.max(size.height),
            );
        }
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        for child in children {
            child.paint(ctx)
        }
    }
}
