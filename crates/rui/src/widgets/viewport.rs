use std::panic::Location;

use druid_shell::kurbo::Size;

use crate::context::LayoutCtx;
use crate::key::Key;
use crate::sliver_constraints::{
    apply_growth_direction_to_scroll_direction, AxisDirection, CacheExtent, GrowthDirection,
    ScrollDirection, SliverConstraints,
};
use crate::style::layout::TextDirection;
use crate::tree::Children;
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
    center: Key,
    cache_extent: CacheExtent,
}

impl Viewport {
    pub fn new(
        axis_direction: AxisDirection,
        cross_axis_direction: AxisDirection,
        anchor: f32,
        offset: ViewportOffset,
        center: Key,
        cache_extent: CacheExtent,
    ) -> Self {
        assert!(anchor >= 0. && anchor <= 1.);
        Viewport {
            axis_direction,
            cross_axis_direction,
            anchor,
            offset,
            center,
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
    center: Key,
    cache_extent: CacheExtent,

    /// This value is set during layout based on the [CacheExtentStyle].
    ///
    /// When the style is [CacheExtentStyle.viewport], it is the main axis extent
    /// of the viewport multiplied by the requested cache extent, which is still
    /// expressed in pixels.
    calculated_cache_extent: Option<f64>,
}

///
/// return true if assigned
/// ```
/// assign_if_not_eq!(self, other, a, b, c);
/// ```
macro_rules! assign_if_not_eq {
    ($l:ident, $r:ident, $( $prop:tt ),* ) => {
        {
            let mut assigned = false;
            $(
                if $l.$prop != $r.$prop {
                    assigned = true;
                    $l.$prop = $r.$prop;
                }
            )*
            assigned
        }
    };
}

impl ViewportObject {
    fn get_default_cross_axis_direction(&self, axis_direction: AxisDirection) -> AxisDirection {
        let text_direction = TextDirection::Ltr;
        match axis_direction {
            AxisDirection::Down => text_direction.to_axis_direction(),
            AxisDirection::Up => text_direction.to_axis_direction(),
            AxisDirection::Left | AxisDirection::Right => AxisDirection::Down,
        }
    }

    /// Determines the size and position of some of the children of the viewport.
    ///
    /// This function is the workhorse of `performLayout` implementations in
    /// subclasses.
    ///
    /// Layout starts with `child`, proceeds according to the `advance` callback,
    /// and stops once `advance` returns null.
    ///
    ///  * `scrollOffset` is the [SliverConstraints.scrollOffset] to pass the
    ///    first child. The scroll offset is adjusted by
    ///    [SliverGeometry.scrollExtent] for subsequent children.
    ///  * `overlap` is the [SliverConstraints.overlap] to pass the first child.
    ///    The overlay is adjusted by the [SliverGeometry.paintOrigin] and
    ///    [SliverGeometry.paintExtent] for subsequent children.
    ///  * `layoutOffset` is the layout offset at which to place the first child.
    ///    The layout offset is updated by the [SliverGeometry.layoutExtent] for
    ///    subsequent children.
    ///  * `remainingPaintExtent` is [SliverConstraints.remainingPaintExtent] to
    ///    pass the first child. The remaining paint extent is updated by the
    ///    [SliverGeometry.layoutExtent] for subsequent children.
    ///  * `mainAxisExtent` is the [SliverConstraints.viewportMainAxisExtent] to
    ///    pass to each child.
    ///  * `crossAxisExtent` is the [SliverConstraints.crossAxisExtent] to pass to
    ///    each child.
    ///  * `growthDirection` is the [SliverConstraints.growthDirection] to pass to
    ///    each child.
    ///
    /// Returns the first non-zero [SliverGeometry.scrollOffsetCorrection]
    /// encountered, if any. Otherwise returns 0.0. Typical callers will call this
    /// function repeatedly until it returns 0.0.
    fn layout_child_sequence(
        self,
        ctx: &mut LayoutCtx,
        children: &mut Children,
        mut scroll_offset: f64,
        overlap: f64,
        mut layout_offset: f64,
        remaining_paint_extent: f64,
        main_axis_extent: f64,
        cross_axis_extent: f64,
        growth_direction: GrowthDirection,
        mut remainting_cache_extent: f64,
        mut cache_origin: f64,
    ) -> f64 {
        assert!(scroll_offset.is_finite());
        assert!(scroll_offset >= 0.0);
        let initial_layout_offset = layout_offset;
        let adjusted_user_scroll_direction = apply_growth_direction_to_scroll_direction(
            self.offset.user_scroll_direction,
            growth_direction,
        );
        let mut max_paint_offset = layout_offset + overlap;
        let mut preceing_scroll_extent: f64 = 0.;
        for child in children {
            let sliver_scroll_offset = scroll_offset.max(0.);
            let corrected_cache_origin = cache_origin.max(-sliver_scroll_offset);
            let cache_extent_correction = cache_origin - corrected_cache_origin;

            assert!(sliver_scroll_offset >= corrected_cache_origin.abs());
            assert!(corrected_cache_origin <= 0.0);
            assert!(sliver_scroll_offset >= 0.0);
            assert!(cache_extent_correction <= 0.0);

            let child_layout_geometry = child.layout_sliver(
                ctx,
                &SliverConstraints {
                    axis_direction: self.axis_direction,
                    growth_direction,
                    user_scroll_direction: adjusted_user_scroll_direction,
                    scroll_offset: sliver_scroll_offset,
                    preceding_scroll_extent: preceing_scroll_extent,
                    overlap: max_paint_offset - layout_offset,
                    remaining_paint_extent: (remaining_paint_extent - layout_offset
                        + initial_layout_offset)
                        .max(0.),
                    cross_axis_extent,
                    cross_axis_direction: self.cross_axis_direction,
                    viewport_main_axis_extent: main_axis_extent,
                    remaining_cache_extent: (remainting_cache_extent + cache_extent_correction)
                        .max(0.0),
                    cache_origin: corrected_cache_origin,
                },
            );
            if child_layout_geometry.scroll_offset_correction != 0. {
                return child_layout_geometry.scroll_offset_correction;
            }

            let effective_layout_offset = layout_offset + child_layout_geometry.paint_origin;
            if child_layout_geometry.visible || scroll_offset > 0.0 {
                self.update_child_layout_offset(child, effective_layout_offset, growth_direction);
            } else {
                self.update_child_layout_offset(
                    child,
                    -scroll_offset + initial_layout_offset,
                    growth_direction,
                );
            }
            max_paint_offset = (effective_layout_offset + child_layout_geometry.paint_extent)
                .max(max_paint_offset);
            scroll_offset -= child_layout_geometry.scroll_extent;
            preceing_scroll_extent += child_layout_geometry.scroll_extent;
            layout_offset += child_layout_geometry.layout_extent;
            if child_layout_geometry.cache_extent != 0.0 {
                remainting_cache_extent -=
                    child_layout_geometry.cache_extent - cache_extent_correction;
                cache_origin =
                    (corrected_cache_origin + child_layout_geometry.cache_extent).min(0.0);
            }
            self.update_out_of_band_data(growth_direction, child_layout_geometry);
        }
        0.0
    }

    pub(crate) fn update_child_layout_offset(
        &self,
        _child: &mut crate::tree::Element,
        _effective_layout_offset: f64,
        _growth_direction: GrowthDirection,
    ) {
        todo!()
    }

    pub(crate) fn update_out_of_band_data(
        &self,
        _growth_direction: GrowthDirection,
        _child_layout_geometry: crate::sliver_constraints::SliverGeometry,
    ) {
        todo!()
    }
}

impl RenderObject<Viewport> for ViewportObject {
    type Action = ();

    fn create(props: Viewport) -> Self {
        ViewportObject {
            axis_direction: props.axis_direction,
            cross_axis_direction: props.cross_axis_direction,
            anchor: props.anchor,
            offset: props.offset,
            center: props.center,
            cache_extent: props.cache_extent,
            calculated_cache_extent: None,
        }
    }

    fn update(&mut self, ctx: &mut crate::context::UpdateCtx, props: Viewport) -> Self::Action {
        if assign_if_not_eq!(
            self,
            props,
            axis_direction,
            cross_axis_direction,
            anchor,
            offset,
            center,
            cache_extent
        ) {
            ctx.request_layout();
        }
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
        _ctx: &mut crate::context::LifeCycleCtx,
        _event: &crate::lifecycle::LifeCycle,
        _children: &mut crate::tree::Children,
    ) {
    }

    fn layout_box(
        &mut self,
        _ctx: &mut crate::context::LayoutCtx,
        _c: &crate::constraints::BoxConstraints,
        _children: &mut crate::tree::Children,
    ) -> Size {
        Size::ZERO
    }

    fn dry_layout_box(
        &mut self,
        _ctx: &mut crate::context::LayoutCtx,
        bc: &crate::constraints::BoxConstraints,
        _children: &mut crate::tree::Children,
    ) -> Size {
        bc.max()
    }

    fn paint(
        &mut self,
        _ctx: &mut crate::context::PaintCtx,
        _children: &mut crate::tree::Children,
    ) {
        todo!()
    }
}
