mod scroll_metrics;

use std::panic::Location;

use druid_shell::kurbo::{Affine, Insets, Point, Size, Vec2};
use druid_shell::piet::RenderContext;
use druid_shell::MouseEvent;

use crate::context::LayoutCtx;
use crate::event::Event;
use crate::key::{Key, LocalKey};
use crate::physics::tolerance::{near_equal, Tolerance};
use crate::sliver_constraints::{
    apply_growth_direction_to_axis_direction, apply_growth_direction_to_scroll_direction,
    axis_direction_to_axis, AxisDirection, CacheExtent, GrowthDirection, ScrollDirection,
    SliverConstraints, SliverGeometry,
};
use crate::style::axis::Axis;
use crate::style::layout::TextDirection;
use crate::tree::{Children, Element};
use crate::{
    object::{Properties, RenderObject, RenderObjectInterface},
    ui::Ui,
};

use self::scroll_metrics::{IScrollMetrics, ScrollMetrics};

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportOffset {
    scroll_metrics: ScrollMetrics,
    did_change_viewport_dimension_or_receive_correction: bool,
    axis: Axis,
    last_axis: Axis,
    has_dimensions: bool,
    pending_dimensions: bool,
}

impl ViewportOffset {
    pub fn new() -> Self {
        Self {
            scroll_metrics: Default::default(),
            did_change_viewport_dimension_or_receive_correction: false,
            axis: Axis::Vertical,
            last_axis: Axis::Vertical,
            has_dimensions: false,
            pending_dimensions: false,
        }
    }
}

impl IScrollMetrics for ViewportOffset {
    fn base(&self) -> &dyn IScrollMetrics {
        &self.scroll_metrics
    }

    fn base_mut(&mut self) -> &mut dyn IScrollMetrics {
        &mut self.scroll_metrics
    }
}

impl ViewportOffset {
    pub fn apply_viewport_dimension(&mut self, viewport_dimension: f64) -> bool {
        if self.scroll_metrics.viewport_dimension != viewport_dimension {
            self.scroll_metrics.viewport_dimension = viewport_dimension;
        }
        true
    }

    pub fn apply_content_dimensions(
        &mut self,
        min_scroll_extent: f64,
        max_scroll_extent: f64,
    ) -> bool {
        if !near_equal(
            self.scroll_metrics.min_scroll_extent,
            min_scroll_extent,
            Tolerance::DEFAULT.distance,
        ) || !near_equal(
            self.scroll_metrics.max_scroll_extent,
            max_scroll_extent,
            Tolerance::DEFAULT.distance,
        ) || self.did_change_viewport_dimension_or_receive_correction
            || self.last_axis != self.axis
        {
            self.scroll_metrics.min_scroll_extent = min_scroll_extent;
            self.scroll_metrics.max_scroll_extent = max_scroll_extent;
            self.last_axis = self.axis;
            return true;
            // todo: more logics
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Viewport {
    axis_direction: AxisDirection,
    cross_axis_direction: AxisDirection,
    anchor: f64,
    offset: ViewportOffset,
    center: Option<LocalKey>,
    cache_extent: CacheExtent,
}

impl Viewport {
    pub fn new(
        axis_direction: AxisDirection,
        cross_axis_direction: AxisDirection,
        anchor: f64,
        center: Option<LocalKey>,
        cache_extent: CacheExtent,
    ) -> Self {
        assert!(anchor >= 0. && anchor <= 1.);
        Viewport {
            axis_direction,
            cross_axis_direction,
            anchor,
            offset: ViewportOffset::new(),
            center,
            cache_extent,
        }
    }

    #[track_caller]
    pub fn build(self, cx: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = crate::key::Key::current();
        cx.render_object(caller, self, content);
    }
}

impl Properties for Viewport {
    type Object = ViewportObject;
}

pub struct ViewportObject {
    axis_direction: AxisDirection,
    cross_axis_direction: AxisDirection,
    offset: ViewportOffset,
    anchor: f64,
    center: Option<LocalKey>,
    cache_extent: CacheExtent,

    /// This value is set during layout based on the [CacheExtentStyle].
    ///
    /// When the style is [CacheExtentStyle.viewport], it is the main axis extent
    /// of the viewport multiplied by the requested cache extent, which is still
    /// expressed in pixels.
    calculated_cache_extent: f64,

    // Out-of-band data computed during layout.
    min_scroll_extent: f64,
    max_scroll_extent: f64,
    has_visual_overflow: bool,
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
    fn layout_child_sequence<'a, T: Iterator<Item = &'a mut Element>>(
        &mut self,
        ctx: &mut LayoutCtx,
        children: T,
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
        let self_size = ctx.child_state.size();
        let initial_layout_offset = layout_offset;
        let adjusted_user_scroll_direction = apply_growth_direction_to_scroll_direction(
            self.offset.user_scroll_direction(),
            growth_direction,
        );
        let mut max_paint_offset = layout_offset + overlap;
        let mut preceding_scroll_extent: f64 = 0.;
        for child in children {
            let sliver_scroll_offset = scroll_offset.max(0.);
            let corrected_cache_origin = cache_origin.max(-sliver_scroll_offset);
            let cache_extent_correction = cache_origin - corrected_cache_origin;

            assert!(sliver_scroll_offset >= corrected_cache_origin.abs());
            assert!(corrected_cache_origin <= 0.0);
            assert!(sliver_scroll_offset >= 0.0);
            assert!(cache_extent_correction <= 0.0);

            let sc = SliverConstraints {
                axis_direction: self.axis_direction,
                growth_direction,
                user_scroll_direction: adjusted_user_scroll_direction,
                scroll_offset: sliver_scroll_offset,
                preceding_scroll_extent,
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
            };
            let child_layout_geometry = child.layout_sliver(ctx, &sc);

            // tracing::debug!(
            //     "{}: {:#?} \n {:#?}",
            //     &child.custom_key,
            //     sc,
            //     child_layout_geometry
            // );
            if child_layout_geometry.scroll_offset_correction != 0. {
                return child_layout_geometry.scroll_offset_correction;
            }

            let effective_layout_offset = layout_offset + child_layout_geometry.paint_origin;
            if child_layout_geometry.visible || scroll_offset > 0.0 {
                self.update_child_layout_offset(
                    ctx,
                    child,
                    self_size,
                    effective_layout_offset,
                    child_layout_geometry.paint_extent,
                    growth_direction,
                );
            } else {
                self.update_child_layout_offset(
                    ctx,
                    child,
                    self_size,
                    -scroll_offset + initial_layout_offset,
                    child_layout_geometry.paint_extent,
                    growth_direction,
                );
            }
            max_paint_offset = (effective_layout_offset + child_layout_geometry.paint_extent)
                .max(max_paint_offset);
            scroll_offset -= child_layout_geometry.scroll_extent;
            preceding_scroll_extent += child_layout_geometry.scroll_extent;
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

    /// Called during `layout_child_sequence` to store the layout offset for the
    /// given child.
    ///
    /// Different subclasses using different representations for their children's
    /// layout offset (e.g., logical or physical coordinates). This function lets
    /// subclasses transform the child's layout offset before storing it in the
    /// child's parent data.
    pub(crate) fn update_child_layout_offset(
        &self,
        ctx: &mut LayoutCtx,
        child: &mut Element,
        self_size: Size,
        layout_offset: f64,
        paint_extent: f64,
        growth_direction: GrowthDirection,
    ) {
        let (origin, insets) =
            match apply_growth_direction_to_axis_direction(self.axis_direction, growth_direction) {
                AxisDirection::Down => (
                    Point::new(0.0, layout_offset),
                    Insets::new(0.0, 0.0, 0.0, paint_extent),
                ),
                AxisDirection::Left => (
                    Point::new(
                        self_size.width - (layout_offset + child.paint_rect().width()),
                        0.0,
                    ),
                    Insets::new(paint_extent, 0.0, 0.0, 0.0),
                ),
                AxisDirection::Right => (
                    Point::new(layout_offset, 0.0),
                    Insets::new(0.0, 0.0, paint_extent, 0.0),
                ),
                AxisDirection::Up => (
                    Point::new(
                        0.0,
                        self_size.height - (layout_offset + child.layout_rect().height()),
                    ),
                    Insets::new(0.0, paint_extent, 0.0, 0.0),
                ),
            };
        child.set_viewport_offset(self.offset.scroll_offset());
        child.set_origin(ctx, origin);
        child.set_paint_insets(insets);
    }

    pub(crate) fn update_out_of_band_data(
        &mut self,
        growth_direction: GrowthDirection,
        child_layout_geometry: crate::sliver_constraints::SliverGeometry,
    ) {
        match growth_direction {
            GrowthDirection::Forward => {
                self.max_scroll_extent += child_layout_geometry.scroll_extent;
            }
            GrowthDirection::Reverse => {
                self.min_scroll_extent -= child_layout_geometry.scroll_extent;
            }
        }
        if child_layout_geometry.has_visual_overflow {
            self.has_visual_overflow = true
        }
    }

    fn axis(&self) -> Axis {
        axis_direction_to_axis(self.axis_direction)
    }

    fn pointer_signal_event_delta(&self, wheel_delta: Vec2) -> f64 {
        let mut delta = match self.axis() {
            Axis::Horizontal => wheel_delta.x,
            Axis::Vertical => wheel_delta.y,
        };
        if self.axis_direction.is_reversed() {
            delta *= -1.;
        }
        delta
    }

    fn target_scroll_offset_for_pointer_scroll(&self, delta: f64) -> f64 {
        ((self.offset.pixels() + delta).max(self.offset.min_scroll_extent()))
            .min(self.offset.max_scroll_extent())
    }

    fn handle_pointer_scroll(&mut self, event: &MouseEvent) {
        let delta = self.pointer_signal_event_delta(event.wheel_delta);
        let target_scroll_offset = self.target_scroll_offset_for_pointer_scroll(delta);
        if delta != 0.0 && target_scroll_offset != self.offset.pixels() {
            self.offset.pointer_scroll(delta);
        }
    }

    pub(crate) fn attempt_layout(
        &mut self,
        main_axis_extent: f64,
        cross_axis_extent: f64,
        corrected_offset: f64,
        ctx: &mut LayoutCtx,
        children: &mut Children,
    ) -> f64 {
        assert!(main_axis_extent >= 0.);
        assert!(cross_axis_extent.is_finite());
        assert!(cross_axis_extent >= 0.0);
        assert!(corrected_offset.is_finite());

        self.min_scroll_extent = 0.0;
        self.max_scroll_extent = 0.0;
        self.has_visual_overflow = false;

        // centerOffset is the offset from the leading edge of the RenderViewport
        // to the zero scroll offset (the line between the forward slivers and the
        // reverse slivers).
        let center_offset = main_axis_extent * self.anchor - corrected_offset;
        let reverse_direction_remaining_paint_extent = center_offset.clamp(0.0, main_axis_extent);
        let forward_direction_remaining_paint_extent =
            (main_axis_extent - center_offset).clamp(0.0, main_axis_extent);

        self.calculated_cache_extent = match self.cache_extent {
            CacheExtent::Pixel(cache_extent) => cache_extent,
            CacheExtent::Viewport(cache_extent) => main_axis_extent * cache_extent,
        };

        let full_cache_extent = main_axis_extent + 2.0 * self.calculated_cache_extent;
        let center_cache_offset = center_offset + self.calculated_cache_extent;
        let reverse_direction_remaining_cache_extent =
            center_cache_offset.clamp(0.0, full_cache_extent);
        let forward_direction_remaining_cache_extent =
            (full_cache_extent - center_cache_offset).clamp(0.0, full_cache_extent);

        let mut leading_negative_children: Vec<_> = children
            .iter()
            .take_while(|c| Some(&c.custom_key) != self.center.as_ref())
            .collect();
        leading_negative_children.reverse();
        let leading_negative_count = leading_negative_children.len();

        if leading_negative_count != 0 {
            let result = self.layout_child_sequence(
                ctx,
                leading_negative_children.into_iter(),
                main_axis_extent.max(center_offset) - main_axis_extent,
                0.0,
                forward_direction_remaining_paint_extent,
                reverse_direction_remaining_paint_extent,
                main_axis_extent,
                cross_axis_extent,
                GrowthDirection::Reverse,
                reverse_direction_remaining_cache_extent,
                (main_axis_extent - center_offset).clamp(-self.calculated_cache_extent, 0.0),
            );
            if result != 0.0 {
                return -result;
            }
        }
        let following_children: Vec<_> = children
            .iter()
            .skip_while(|c| Some(&c.custom_key) != self.center.as_ref())
            .collect();
        return self.layout_child_sequence(
            ctx,
            following_children.into_iter(),
            (-center_offset).max(0.0),
            if leading_negative_count == 0 {
                (-center_offset).min(0.0)
            } else {
                0.0
            },
            if center_offset >= main_axis_extent {
                center_offset
            } else {
                reverse_direction_remaining_paint_extent
            },
            forward_direction_remaining_paint_extent,
            main_axis_extent,
            cross_axis_extent,
            GrowthDirection::Forward,
            forward_direction_remaining_cache_extent,
            center_offset.clamp(-self.calculated_cache_extent, 0.0),
        );
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
            calculated_cache_extent: 0.0,
            min_scroll_extent: 0.,
            max_scroll_extent: 0.,
            has_visual_overflow: false,
        }
    }

    fn update(&mut self, ctx: &mut crate::context::UpdateCtx, props: Viewport) -> Self::Action {
        if assign_if_not_eq!(
            self,
            props,
            axis_direction,
            cross_axis_direction,
            anchor,
            cache_extent
        ) {
            ctx.request_layout();
        }

        let should_assign = match (&self.center, &props.center) {
            (None, None) => false,
            (None, Some(_)) => true,
            // No center is provided, we used the first child.
            (Some(_), None) => false,
            (Some(_), Some(_)) => true,
        };
        if should_assign && self.center != props.center {
            self.center = props.center;
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
        match event {
            Event::Wheel(mouse_event) => {
                self.handle_pointer_scroll(mouse_event);
                // for child in children {
                //     child.state.set_viewport_offset(self.offset.scroll_offset());
                // }
                // tracing::debug!("scroll offset: {}, children len: {}", self.offset.pixels(), children.len());
                ctx.request_layout();
                ctx.set_handled();
                return;
            }
            _ => {}
        }
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
        for child in children {
            child.lifecycle(ctx, event);
        }
    }

    fn dry_layout_box(
        &mut self,
        _ctx: &mut crate::context::LayoutCtx,
        bc: &crate::constraints::BoxConstraints,
        _children: &mut crate::tree::Children,
    ) -> Size {
        bc.max()
    }

    fn layout_box(
        &mut self,
        ctx: &mut crate::context::LayoutCtx,
        bc: &crate::constraints::BoxConstraints,
        children: &mut crate::tree::Children,
    ) -> Size {
        // tracing::debug!("layout_box, offset: {:?}", self.offset.pixels());
        let self_size = bc.max();

        match self.axis() {
            Axis::Horizontal => self.offset.apply_viewport_dimension(self_size.width),
            Axis::Vertical => self.offset.apply_viewport_dimension(self_size.height),
        };

        if self.center.is_none() {
            self.center = children.first().map(|c| c.custom_key.clone());
        }

        if self.center.is_none() {
            self.min_scroll_extent = 0.0;
            self.max_scroll_extent = 0.0;
            self.has_visual_overflow = false;
            self.offset.apply_content_dimensions(0.0, 0.0);
            return self_size;
        }

        let (main_axis_extent, cross_axis_extent) = match self.axis() {
            Axis::Horizontal => (self_size.width, self_size.height),
            Axis::Vertical => (self_size.height, self_size.width),
        };
        // todo:
        // let center_offset_adjustment = center_child.center_offset_adjustment;
        let center_offset_adjustment = 0.;
        let mut correction;
        const MAX_LAYOUT_CYCLES: usize = 10;
        for i in 0..MAX_LAYOUT_CYCLES {
            correction = self.attempt_layout(
                main_axis_extent,
                cross_axis_extent,
                self.offset.pixels() + center_offset_adjustment,
                ctx,
                children,
            );
            // tracing::debug!("attempt_layout: {}, correction: {}", i, correction);

            if correction != 0. {
                self.offset.correct_by(correction);
            } else {
                if self.offset.apply_content_dimensions(
                    (self.min_scroll_extent + main_axis_extent * self.anchor).min(0.),
                    (self.max_scroll_extent - main_axis_extent * (1.0 - self.anchor)).max(0.),
                ) {
                    break;
                }
            }
        }
        self_size
    }

    fn paint(&mut self, ctx: &mut crate::context::PaintCtx, children: &mut crate::tree::Children) {
        if children.is_empty() {
            return;
        }

        let clip = ctx.size().to_rect();
        ctx.clip(clip);

        for child in children {
            if child.state.geometry.visible {
                child.paint(ctx);
            }
        }
    }
}
