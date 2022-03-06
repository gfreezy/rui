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

enum LayoutType {
    Box,
    Sliver,
}

pub trait RenderObject {
    const layout_type: LayoutType = LayoutType::Box;

    type Props;
    type Action: Default;

    fn create(props: Self::Props) -> Self;
    fn update(&mut self, ctx: &mut UpdateCtx, props: Self::Props) -> Self::Action;
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

pub trait RenderSliverInterface {
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
    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &Constraints,
        children: &mut Children,
    ) -> Size {
        self.dry_layout(ctx, c, children)
    }
    fn layout_sliver(
        &mut self,
        ctx: &mut LayoutCtx,
        sc: &SliverConstraints,
        children: &mut Children,
    ) -> Size;
    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children);
}

impl<R> AnyRenderObject for R
where
    R: RenderObject + 'static,
{
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        Self::event(self, ctx, event, children)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children) {
        Self::lifecycle(self, ctx, event, children)
    }
    fn dry_layout(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &Constraints,
        children: &mut Children,
    ) -> Size {
        Self::dry_layout(self, ctx, c, children)
    }

    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &Constraints,
        children: &mut Children,
    ) -> Size {
        Self::layout(self, ctx, c, children)
    }

    fn layout_sliver(
        &mut self,
        ctx: &mut LayoutCtx,
        sc: &SliverConstraints,
        children: &mut Children,
    ) -> Size;
    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children);
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
