use std::any::Any;

use druid_shell::kurbo::Size;

use crate::constraints::Constraints;
use crate::lifecycle::LifeCycle;
use crate::sliver_constraints::SliverConstraints;
use crate::{
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    tree::Children,
};

pub trait Properties: Sized {
    type Object: RenderObject<Self>;
}

pub trait RenderObject<Props>: RenderObjectInterface {
    type Action: Default;

    fn create(props: Props) -> Self;
    fn update(&mut self, ctx: &mut UpdateCtx, props: Props) -> Self::Action;
}

pub trait RenderObjectInterface {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children);
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children);
    fn dry_layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children)
        -> Size;
    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children) -> Size {
        self.dry_layout(ctx, c, children)
    }
    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children);
}

pub trait SliverRenderObjectInterface {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children);
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children);
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        sc: &SliverConstraints,
        children: &mut Children,
    ) -> Size;
    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children);
}

pub trait AnyRenderObject: Any {
    fn as_any(&mut self) -> &mut dyn Any;
    fn name(&self) -> &'static str;

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children);
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children);
    fn dry_layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children)
        -> Size;
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &Constraints, children: &mut Children) -> Size {
        self.dry_layout(ctx, bc, children)
    }
    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children);
}

impl<R> AnyRenderObject for R
where
    R: RenderObjectInterface + Any,
{
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        R::event(self, ctx, event, children)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children) {
        R::lifecycle(self, ctx, event, children)
    }

    fn dry_layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &Constraints,
        children: &mut Children,
    ) -> Size {
        R::dry_layout(self, ctx, bc, children)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &Constraints, children: &mut Children) -> Size {
        R::layout(self, ctx, bc, children)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        R::paint(self, ctx, children)
    }
}

pub trait AnyParentData {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn eql(&self, other: &dyn AnyParentData) -> bool;
}

impl<T: PartialEq + 'static> AnyParentData for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn eql(&self, other: &dyn AnyParentData) -> bool {
        Some(self) == other.as_any().downcast_ref::<T>()
    }
}
