use std::os::raw::c_double;
use std::panic::Location;

use druid_shell::kurbo::{Point, Size};

use crate::sliver_constraints::{AxisDirection, CacheExtent, ScrollDirection};
use crate::{
    object::{Properties, RenderObject, RenderObjectInterface},
    ui::Ui,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ViewportOffset {
    pixels: f32,
    user_scroll_direction: ScrollDirection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    axis_direction: AxisDirection,
    cross_axis_direction: AxisDirection,
    anchor: f32,
    offset: ViewportOffset,
    cache_extent: CacheExtent,
}

impl Viewport {
    pub fn new(
        axis_direction: AxisDirection,
        cross_axis_direction: AxisDirection,
        anchor: f32,
        offset: ViewportOffset,
        cache_extent: CacheExtent,
    ) -> Self {
        Viewport {
            axis_direction,
            cross_axis_direction,
            anchor,
            offset,
            cache_extent,
        }
    }

    #[track_caller]
    pub fn build(self, cx: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = Location::caller().into();
        cx.render_object(caller, self, content);
    }
}

impl Properties for Viewport {
    type Object = ViewportObject;
}

pub struct ViewportObject {
    axis_direction: AxisDirection,
    cross_axis_direction: AxisDirection,
    anchor: f32,
    offset: ViewportOffset,
    cache_extent: CacheExtent,
}

macro_rules! eq_props {
    ($l:ident, $r:ident, $( $prop:tt ),* ) => {
        {
            let mut eq = true;
            $(
                if $l.$prop != $r.$prop {
                    eq = false;
                }
            )*
            eq
        }
    };
}

impl RenderObject<Viewport> for ViewportObject {
    type Action = ();

    fn create(props: Viewport) -> Self {
        ViewportObject {
            axis_direction: props.axis_direction,
            cross_axis_direction: props.cross_axis_direction,
            anchor: props.anchor,
            offset: props.offset,
            cache_extent: props.cache_extent,
        }
    }

    fn update(&mut self, ctx: &mut crate::context::UpdateCtx, props: Viewport) -> Self::Action {
        if !eq_props!(
            self,
            props,
            axis_direction,
            cross_axis_direction,
            anchor,
            offset,
            cache_extent
        ) {
            ctx.request_layout();
        }
        self.axis_direction = props.axis_direction;
        self.cross_axis_direction = props.cross_axis_direction;
        self.anchor = props.anchor;
        self.offset = props.offset;
        self.cache_extent = props.cache_extent;
    }
}

impl RenderObjectInterface for ViewportObject {
    fn event(
        &mut self,
        ctx: &mut crate::context::EventCtx,
        event: &crate::event::Event,
        children: &mut crate::tree::Children,
    ) {
        for child in children {
            child.event(ctx, event);
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut crate::context::LifeCycleCtx,
        event: &crate::lifecycle::LifeCycle,
        children: &mut crate::tree::Children,
    ) {
    }

    fn layout(
        &mut self,
        ctx: &mut crate::context::LayoutCtx,
        c: &crate::constraints::Constraints,
        children: &mut crate::tree::Children,
    ) -> Size {
        todo!()
    }

    fn paint(&mut self, ctx: &mut crate::context::PaintCtx, children: &mut crate::tree::Children) {
        todo!()
    }
}
