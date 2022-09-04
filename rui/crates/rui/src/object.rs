use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use druid_shell::kurbo::Size;

use crate::constraints::BoxConstraints;
use crate::lifecycle::LifeCycle;
use crate::sliver_constraints::{SliverConstraints, SliverGeometry};
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
    fn update(&mut self, ctx: &mut UpdateCtx, props: Props) -> Self::Action {
        Default::default()
    }
}

pub trait RenderObjectInterface {
    fn sized_by_parent(&self) -> bool {
        false
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        for child in children {
            child.event(ctx, event);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children) {
        for child in children {
            child.lifecycle(ctx, event);
        }
    }

    fn dry_layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        unreachable!()
    }

    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        unimplemented!()
    }

    fn layout_sliver(
        &mut self,
        ctx: &mut LayoutCtx,
        sc: &SliverConstraints,
        children: &mut Children,
    ) -> SliverGeometry {
        unimplemented!()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        for child in children {
            child.paint(ctx);
        }
    }

    fn debug_state(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub trait AnyRenderObject: Any {
    fn as_any(&mut self) -> &mut dyn Any;
    fn name(&self) -> &'static str;
    fn sized_by_parent(&self) -> bool;

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children);
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, children: &mut Children);
    fn dry_layout(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &BoxConstraints,
        children: &mut Children,
    ) -> Size;
    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size;
    fn layout_sliver(
        &mut self,
        ctx: &mut LayoutCtx,
        sc: &SliverConstraints,
        children: &mut Children,
    ) -> SliverGeometry;
    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children);
    fn debug_state(&self) -> HashMap<String, String>;
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

    fn sized_by_parent(&self) -> bool {
        R::sized_by_parent(self)
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
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        R::dry_layout_box(self, ctx, bc, children)
    }

    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        R::layout_box(self, ctx, bc, children)
    }

    fn layout_sliver(
        &mut self,
        ctx: &mut LayoutCtx,
        sc: &SliverConstraints,
        children: &mut Children,
    ) -> SliverGeometry {
        R::layout_sliver(self, ctx, sc, children)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        R::paint(self, ctx, children)
    }

    fn debug_state(&self) -> HashMap<String, String> {
        R::debug_state(self)
    }
}

pub trait AnyParentData: Debug {
    fn as_any(&self) -> &dyn Any;
    fn to_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn eql(&self, other: &dyn AnyParentData) -> bool;
}

impl<T: PartialEq + Debug + 'static> AnyParentData for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn to_any_box(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn eql(&self, other: &dyn AnyParentData) -> bool {
        Some(self) == other.as_any().downcast_ref::<T>()
    }
}
