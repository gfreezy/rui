use std::any::Any;

use druid_shell::kurbo::Size;

use crate::{
    constraints::Constraints,
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    lifecycle::LifeCycle,
};

pub trait Properties: Sized {
    type Widget: Render<Self> + 'static;
}

pub trait Render<Props>: RenderInterface {
    fn create(props: Props) -> Self;
    fn update(&mut self, ctx: &mut UpdateCtx, props: Props);
}

pub trait RenderInterface {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event);
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle);
    fn dry_layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints) -> Size;
    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints) -> Size {
        self.dry_layout(ctx, c)
    }
    fn paint(&mut self, ctx: &mut PaintCtx);
}

pub trait AnyRenderObject: Any {
    fn as_any(&mut self) -> &mut dyn Any;
    fn name(&self) -> &'static str;

    fn event(&mut self, ctx: &mut EventCtx, event: &Event);
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle);
    fn dry_layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints) -> Size;
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &Constraints) -> Size {
        self.dry_layout(ctx, bc)
    }
    fn paint(&mut self, ctx: &mut PaintCtx);
}

impl<R> AnyRenderObject for R
where
    R: RenderInterface + Any,
{
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        R::event(self, ctx, event)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle) {
        R::lifecycle(self, ctx, event)
    }

    fn dry_layout(&mut self, ctx: &mut LayoutCtx, bc: &Constraints) -> Size {
        R::dry_layout(self, ctx, bc)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &Constraints) -> Size {
        R::layout(self, ctx, bc)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        R::paint(self, ctx)
    }
}
