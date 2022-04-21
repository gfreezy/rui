use druid_shell::kurbo::Size;

use crate::{box_constraints::BoxConstraints, physics::tolerance::default_near_equal};
use style::{axis::Axis, layout::AxisDirection};

/// The direction in which a sliver's contents are ordered, relative to the
/// scroll offset axis.
///
/// For example, a vertical alphabetical list that is going [AxisDirection.down]
/// with a [GrowthDirection.forward] would have the A at the top and the Z at
/// the bottom, with the A adjacent to the origin, as would such a list going
/// [AxisDirection.up] with a [GrowthDirection.reverse]. On the other hand, a
/// vertical alphabetical list that is going [AxisDirection.down] with a
/// [GrowthDirection.reverse] would have the Z at the top (at scroll offset
/// zero) and the A below it.
///
/// The direction in which the scroll offset increases is given by
/// [applyGrowthDirectionToAxisDirection].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GrowthDirection {
    /// This sliver's contents are ordered in the same direction as the
    /// [AxisDirection].
    Forward,

    /// This sliver's contents are ordered in the opposite direction of the
    /// [AxisDirection].
    Reverse,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScrollDirection {
    Forward,
    Idle,
    Reverse,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Idle
    }
}

impl ScrollDirection {
    pub fn flip(&self) -> ScrollDirection {
        match self {
            ScrollDirection::Forward => ScrollDirection::Reverse,
            ScrollDirection::Idle => ScrollDirection::Idle,
            ScrollDirection::Reverse => ScrollDirection::Forward,
        }
    }
}
/// Flips the [ScrollDirection] if the [GrowthDirection] is [GrowthDirection.reverse].
///
/// Specifically, returns `scrollDirection` if `scrollDirection` is
/// [GrowthDirection.forward], otherwise returns [flipScrollDirection] applied to
/// `scrollDirection`.
///
/// This function is useful in [RenderSliver] subclasses that are given both an
/// [ScrollDirection] and a [GrowthDirection] and wish to compute the
/// [ScrollDirection] in which growth will occur.
pub fn apply_growth_direction_to_scroll_direction(
    scroll_direction: ScrollDirection,
    growth_direction: GrowthDirection,
) -> ScrollDirection {
    match growth_direction {
        GrowthDirection::Forward => scroll_direction,
        GrowthDirection::Reverse => scroll_direction.flip(),
    }
}
pub fn apply_growth_direction_to_axis_direction(
    axis_direction: AxisDirection,
    growth_direction: GrowthDirection,
) -> AxisDirection {
    match growth_direction {
        GrowthDirection::Forward => axis_direction,
        GrowthDirection::Reverse => axis_direction.flip(),
    }
}
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum CacheExtent {
    Pixel(f64),
    Viewport(f64),
}

#[derive(Clone, Debug)]
pub struct SliverConstraints {
    /// The direction in which the [scrollOffset] and [remainingPaintExtent]
    /// increase.
    pub axis_direction: AxisDirection,

    /// The direction in which the contents of slivers are ordered, relative to
    /// the [axisDirection].
    ///
    /// For example, if the [axisDirection] is [AxisDirection.up], and the
    /// [growthDirection] is [GrowthDirection.forward], then an alphabetical list
    /// will have A at the bottom, then B, then C, and so forth, with Z at the
    /// top, with the bottom of the A at scroll offset zero, and the top of the Z
    /// at the highest scroll offset.
    ///
    /// If a viewport has an overall [AxisDirection] of [AxisDirection.down], then
    /// slivers above the absolute zero offset will have an axis of
    /// [AxisDirection.up] and a growth direction of [GrowthDirection.reverse],
    /// while slivers below the absolute zero offset will have the same axis
    /// direction as the viewport and a growth direction of
    /// [GrowthDirection.forward]. (The slivers with a reverse growth direction
    /// still see only positive scroll offsets; the scroll offsets are reversed as
    /// well, with zero at the absolute zero point, and positive numbers going
    /// away from there.)
    ///
    /// Normally, the absolute zero offset is determined by the viewport's
    /// [RenderViewport.center] and [RenderViewport.anchor] properties.
    pub growth_direction: GrowthDirection,

    /// The direction in which the user is attempting to scroll, relative to the
    /// [axisDirection] and [growthDirection].
    ///
    /// For example, if [growthDirection] is [GrowthDirection.reverse] and
    /// [axisDirection] is [AxisDirection.down], then a
    /// [ScrollDirection.forward] means that the user is scrolling up, in the
    /// positive [scrollOffset] direction.
    ///
    /// If the _user_ is not scrolling, this will return [ScrollDirection.idle]
    /// even if there is (for example) a [ScrollActivity] currently animating the
    /// position.
    ///
    /// This is used by some slivers to determine how to react to a change in
    /// scroll offset. For example, [RenderSliverFloatingPersistentHeader] will
    /// only expand a floating app bar when the [userScrollDirection] is in the
    /// positive scroll offset direction.
    pub user_scroll_direction: ScrollDirection,

    /// The scroll offset, in this sliver's coordinate system, that corresponds to
    /// the earliest visible part of this sliver in the [AxisDirection] if
    /// [growthDirection] is [GrowthDirection.forward] or in the opposite
    /// [AxisDirection] direction if [growthDirection] is [GrowthDirection.reverse].
    ///
    /// For example, if [AxisDirection] is [AxisDirection.down] and [growthDirection]
    /// is [GrowthDirection.forward], then scroll offset is the amount the top of
    /// the sliver has been scrolled past the top of the viewport.
    ///
    /// This value is typically used to compute whether this sliver should still
    /// protrude into the viewport via [SliverGeometry.paintExtent] and
    /// [SliverGeometry.layoutExtent] considering how far the beginning of the
    /// sliver is above the beginning of the viewport.
    ///
    /// For slivers whose top is not past the top of the viewport, the
    /// [scrollOffset] is `0` when [AxisDirection] is [AxisDirection.down] and
    /// [growthDirection] is [GrowthDirection.forward]. The set of slivers with
    /// [scrollOffset] `0` includes all the slivers that are below the bottom of the
    /// viewport.
    ///
    /// [SliverConstraints.remainingPaintExtent] is typically used to accomplish
    /// the same goal of computing whether scrolled out slivers should still
    /// partially 'protrude in' from the bottom of the viewport.
    ///
    /// Whether this corresponds to the beginning or the end of the sliver's
    /// contents depends on the [growthDirection].
    pub scroll_offset: f64,

    /// The scroll distance that has been consumed by all [RenderSliver]s that
    /// came before this [RenderSliver].
    ///
    /// # Edge Cases
    ///
    /// [RenderSliver]s often lazily create their internal content as layout
    /// occurs, e.g., [SliverList]. In this case, when [RenderSliver]s exceed the
    /// viewport, their children are built lazily, and the [RenderSliver] does not
    /// have enough information to estimate its total extent,
    /// [precedingScrollExtent] will be [double.infinity] for all [RenderSliver]s
    /// that appear after the lazily constructed child. This is because a total
    /// [SliverGeometry.scrollExtent] cannot be calculated unless all inner
    /// children have been created and sized, or the number of children and
    /// estimated extents are provided. The infinite [SliverGeometry.scrollExtent]
    /// will become finite as soon as enough information is available to estimate
    /// the overall extent of all children within the given [RenderSliver].
    ///
    /// [RenderSliver]s may legitimately be infinite, meaning that they can scroll
    /// content forever without reaching the end. For any [RenderSliver]s that
    /// appear after the infinite [RenderSliver], the [precedingScrollExtent] will
    /// be [double.infinity].
    pub preceding_scroll_extent: f64,

    /// The number of pixels from where the pixels corresponding to the
    /// [scrollOffset] will be painted up to the first pixel that has not yet been
    /// painted on by an earlier sliver, in the [axisDirection].
    ///
    /// For example, if the previous sliver had a [SliverGeometry.paintExtent] of
    /// 100.0 pixels but a [SliverGeometry.layoutExtent] of only 50.0 pixels,
    /// then the [overlap] of this sliver will be 50.0.
    ///
    /// This is typically ignored unless the sliver is itself going to be pinned
    /// or floating and wants to avoid doing so under the previous sliver.
    pub overlap: f64,

    /// The number of pixels of content that the sliver should consider providing.
    /// (Providing more pixels than this is inefficient.)
    ///
    /// The actual number of pixels provided should be specified in the
    /// [RenderSliver.geometry] as [SliverGeometry.paintExtent].
    ///
    /// This value may be infinite, for example if the viewport is an
    /// unconstrained [RenderShrinkWrappingViewport].
    ///
    /// This value may be 0.0, for example if the sliver is scrolled off the
    /// bottom of a downwards vertical viewport.
    pub remaining_paint_extent: f64,

    /// The number of pixels in the cross-axis.
    ///
    /// For a vertical list, this is the width of the sliver.
    pub cross_axis_extent: f64,

    /// The direction in which children should be placed in the cross axis.
    ///
    /// Typically used in vertical lists to describe whether the ambient
    /// [TextDirection] is [TextDirection.rtl] or [TextDirection.ltr].
    pub cross_axis_direction: AxisDirection,

    /// The number of pixels the viewport can display in the main axis.
    ///
    /// For a vertical list, this is the height of the viewport.
    pub viewport_main_axis_extent: f64,

    /// Where the cache area starts relative to the [scrollOffset].
    ///
    /// Slivers that fall into the cache area located before the leading edge and
    /// after the trailing edge of the viewport should still render content
    /// because they are about to become visible when the user scrolls.
    ///
    /// The [cacheOrigin] describes where the [remainingCacheExtent] starts relative
    /// to the [scrollOffset]. A cache origin of 0 means that the sliver does not
    /// have to provide any content before the current [scrollOffset]. A
    /// [cacheOrigin] of -250.0 means that even though the first visible part of
    /// the sliver will be at the provided [scrollOffset], the sliver should
    /// render content starting 250.0 before the [scrollOffset] to fill the
    /// cache area of the viewport.
    ///
    /// The [cacheOrigin] is always negative or zero and will never exceed
    /// -[scrollOffset]. In other words, a sliver is never asked to provide
    /// content before its zero [scrollOffset].
    ///
    /// See also:
    ///
    ///  * [RenderViewport.cacheExtent] for a description of a viewport's cache area.
    pub cache_origin: f64,

    /// Describes how much content the sliver should provide starting from the
    /// [cacheOrigin].
    ///
    /// Not all content in the [remainingCacheExtent] will be visible as some
    /// of it might fall into the cache area of the viewport.
    ///
    /// Each sliver should start laying out content at the [cacheOrigin] and
    /// try to provide as much content as the [remainingCacheExtent] allows.
    ///
    /// The [remainingCacheExtent] is always larger or equal to the
    /// [remainingPaintExtent]. Content, that falls in the [remainingCacheExtent],
    /// but is outside of the [remainingPaintExtent] is currently not visible
    /// in the viewport.
    ///
    /// See also:
    ///
    ///  * [RenderViewport.cacheExtent] for a description of a viewport's cache area.
    pub remaining_cache_extent: f64,
}

impl PartialEq for SliverConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.axis_direction == other.axis_direction
            && self.growth_direction == other.growth_direction
            && default_near_equal(self.scroll_offset, other.scroll_offset)
            && default_near_equal(self.overlap, other.overlap)
            && default_near_equal(self.remaining_paint_extent, other.remaining_paint_extent)
            && default_near_equal(self.cross_axis_extent, other.cross_axis_extent)
            && self.cross_axis_direction == other.cross_axis_direction
            && default_near_equal(
                self.viewport_main_axis_extent,
                other.viewport_main_axis_extent,
            )
            && default_near_equal(self.cache_origin, other.cache_origin)
            && default_near_equal(self.remaining_cache_extent, other.remaining_cache_extent)
    }
}

pub fn axis_direction_to_axis(axis_direction: AxisDirection) -> Axis {
    match axis_direction {
        AxisDirection::Up | AxisDirection::Down => Axis::Vertical,
        AxisDirection::Left | AxisDirection::Right => Axis::Horizontal,
    }
}

impl SliverConstraints {
    pub fn axis(&self) -> Axis {
        axis_direction_to_axis(self.axis_direction)
    }

    pub fn normalized_growth_direction(&self) -> GrowthDirection {
        match self.axis_direction {
            AxisDirection::Down | AxisDirection::Right => self.growth_direction,
            AxisDirection::Left | AxisDirection::Up => match self.growth_direction {
                GrowthDirection::Forward => GrowthDirection::Reverse,
                GrowthDirection::Reverse => GrowthDirection::Forward,
            },
        }
    }

    pub fn is_normalized(&self) -> bool {
        self.scroll_offset >= 0.0
            && self.cross_axis_extent >= 0.0
            && axis_direction_to_axis(self.axis_direction)
                != axis_direction_to_axis(self.cross_axis_direction)
            && self.viewport_main_axis_extent >= 0.0
            && self.remaining_paint_extent >= 0.0
    }

    pub fn as_box_constraints(
        &self,
        min_extent: f64,
        max_extent: f64,
        cross_axis_extent: Option<f64>,
    ) -> BoxConstraints {
        let cross_axis_extent = cross_axis_extent.unwrap_or(self.cross_axis_extent);
        match self.axis() {
            Axis::Horizontal => BoxConstraints::new(
                Size::new(min_extent, cross_axis_extent),
                Size::new(max_extent, cross_axis_extent),
            ),
            Axis::Vertical => BoxConstraints::new(
                Size::new(cross_axis_extent, min_extent),
                Size::new(cross_axis_extent, max_extent),
            ),
        }
    }

    pub(crate) fn is_tight(&self) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct SliverGeometry {
    /// The (estimated) total scrollable extent that this sliver has content for.
    ///
    /// This is the amount of scrolling the user needs to do to get from the
    /// beginning of this sliver to the end of this sliver.
    ///
    /// The value is used to calculate the [SliverConstraints.scrollOffset] of
    /// all slivers in the scrollable and thus should be provided whether the
    /// sliver is currently in the viewport or not.
    ///
    /// In a typical scrolling scenario, the [scrollExtent] is constant for a
    /// sliver throughout the scrolling while [paintExtent] and [layoutExtent]
    /// will progress from `0` when offscreen to between `0` and [scrollExtent]
    /// as the sliver scrolls partially into and out of the screen and is
    /// equal to [scrollExtent] while the sliver is entirely on screen. However,
    /// these relationships can be customized to achieve more special effects.
    ///
    /// This value must be accurate if the [paintExtent] is less than the
    /// [SliverConstraints.remainingPaintExtent] provided during layout.
    pub scroll_extent: f64,

    /// The visual location of the first visible part of this sliver relative to
    /// its layout position.
    ///
    /// For example, if the sliver wishes to paint visually before its layout
    /// position, the [paintOrigin] is negative. The coordinate system this sliver
    /// uses for painting is relative to this [paintOrigin]. In other words,
    /// when [RenderSliver.paint] is called, the (0, 0) position of the [Offset]
    /// given to it is at this [paintOrigin].
    ///
    /// The coordinate system used for the [paintOrigin] itself is relative
    /// to the start of this sliver's layout position rather than relative to
    /// its current position on the viewport. In other words, in a typical
    /// scrolling scenario, [paintOrigin] remains constant at 0.0 rather than
    /// tracking from 0.0 to [SliverConstraints.viewportMainAxisExtent] as the
    /// sliver scrolls past the viewport.
    ///
    /// This value does not affect the layout of subsequent slivers. The next
    /// sliver is still placed at [layoutExtent] after this sliver's layout
    /// position. This value does affect where the [paintExtent] extent is
    /// measured from when computing the [SliverConstraints.overlap] for the next
    /// sliver.
    ///
    /// Defaults to 0.0, which means slivers start painting at their layout
    /// position by default.
    pub paint_origin: f64,

    /// The amount of currently visible visual space that was taken by the sliver
    /// to render the subset of the sliver that covers all or part of the
    /// [SliverConstraints.remainingPaintExtent] in the current viewport.
    ///
    /// This value does not affect how the next sliver is positioned. In other
    /// words, if this value was 100 and [layoutExtent] was 0, typical slivers
    /// placed after it would end up drawing in the same 100 pixel space while
    /// painting.
    ///
    /// This must be between zero and [SliverConstraints.remainingPaintExtent].
    ///
    /// This value is typically 0 when outside of the viewport and grows or
    /// shrinks from 0 or to 0 as the sliver is being scrolled into and out of the
    /// viewport unless the sliver wants to achieve a special effect and paint
    /// even when scrolled away.
    ///
    /// This contributes to the calculation for the next sliver's
    /// [SliverConstraints.overlap].
    pub paint_extent: f64,

    /// The distance from the first visible part of this sliver to the first
    /// visible part of the next sliver, assuming the next sliver's
    /// [SliverConstraints.scrollOffset] is zero.
    ///
    /// This must be between zero and [paintExtent]. It defaults to [paintExtent].
    ///
    /// This value is typically 0 when outside of the viewport and grows or
    /// shrinks from 0 or to 0 as the sliver is being scrolled into and out of the
    /// viewport unless the sliver wants to achieve a special effect and push
    /// down the layout start position of subsequent slivers before the sliver is
    /// even scrolled into the viewport.
    pub layout_extent: f64,

    /// The (estimated) total paint extent that this sliver would be able to
    /// provide if the [SliverConstraints.remainingPaintExtent] was infinite.
    ///
    /// This is used by viewports that implement shrink-wrapping.
    ///
    /// By definition, this cannot be less than [paintExtent].
    pub max_paint_extent: f64,

    /// The maximum extent by which this sliver can reduce the area in which
    /// content can scroll if the sliver were pinned at the edge.
    ///
    /// Slivers that never get pinned at the edge, should return zero.
    ///
    /// A pinned app bar is an example for a sliver that would use this setting:
    /// When the app bar is pinned to the top, the area in which content can
    /// actually scroll is reduced by the height of the app bar.
    pub max_scroll_obstruction_extent: f64,

    /// The distance from where this sliver started painting to the bottom of
    /// where it should accept hits.
    ///
    /// This must be between zero and [paintExtent]. It defaults to [paintExtent].
    pub hit_test_extent: f64,

    /// Whether this sliver should be painted.
    ///
    /// By default, this is true if [paintExtent] is greater than zero, and
    /// false if [paintExtent] is zero.
    pub visible: bool,

    /// Whether this sliver has visual overflow.
    ///
    /// By default, this is false, which means the viewport does not need to clip
    /// its children. If any slivers have visual overflow, the viewport will apply
    /// a clip to its children.
    pub has_visual_overflow: bool,

    /// If this is non-zero after [RenderSliver.performLayout] returns, the scroll
    /// offset will be adjusted by the parent and then the entire layout of the
    /// parent will be rerun.
    ///
    /// When the value is non-zero, the [RenderSliver] does not need to compute
    /// the rest of the values when constructing the [SliverGeometry] or call
    /// [RenderObject.layout] on its children since [RenderSliver.performLayout]
    /// will be called again on this sliver in the same frame after the
    /// [SliverConstraints.scrollOffset] correction has been applied, when the
    /// proper [SliverGeometry] and layout of its children can be computed.
    ///
    /// If the parent is also a [RenderSliver], it must propagate this value
    /// in its own [RenderSliver.geometry] property until a viewport which adjusts
    /// its offset based on this value.
    pub scroll_offset_correction: f64,

    /// How many pixels the sliver has consumed in the
    /// [SliverConstraints.remainingCacheExtent].
    ///
    /// This value should be equal to or larger than the [layoutExtent] because
    /// the sliver always consumes at least the [layoutExtent] from the
    /// [SliverConstraints.remainingCacheExtent] and possibly more if it falls
    /// into the cache area of the viewport.
    ///
    /// See also:
    ///
    ///  * [RenderViewport.cacheExtent] for a description of a viewport's cache area.
    pub cache_extent: f64,
}

impl SliverGeometry {
    pub const ZERO: SliverGeometry = SliverGeometry {
        scroll_extent: 0.,
        paint_origin: 0.,
        paint_extent: 0.0,
        layout_extent: 0.0,
        max_paint_extent: 0.0,
        max_scroll_obstruction_extent: 0.0,
        hit_test_extent: 0.0,
        visible: false,
        has_visual_overflow: false,
        scroll_offset_correction: 0.0,
        cache_extent: 0.0,
    };

    pub fn new(
        scroll_extent: impl Into<Option<f64>>,
        paint_origin: impl Into<Option<f64>>,
        paint_extent: impl Into<Option<f64>>,
        layout_extent: impl Into<Option<f64>>,
        max_paint_extent: impl Into<Option<f64>>,
        max_scroll_obstruction_extent: impl Into<Option<f64>>,
        hit_test_extent: impl Into<Option<f64>>,
        visible: impl Into<Option<bool>>,
        has_visual_overflow: impl Into<Option<bool>>,
        scroll_offset_correction: impl Into<Option<f64>>,
        cache_extent: impl Into<Option<f64>>,
    ) -> Self {
        let scroll_extent = scroll_extent.into().unwrap_or(0.);
        let paint_origin = paint_origin.into().unwrap_or(0.);
        let paint_extent = paint_extent.into().unwrap_or(0.);
        let layout_extent_option = layout_extent.into();
        let cache_extent = cache_extent
            .into()
            .or_else(|| layout_extent_option)
            .unwrap_or(paint_extent);
        let layout_extent = layout_extent_option.unwrap_or(paint_extent);
        let max_paint_extent = max_paint_extent.into().unwrap_or(0.);
        let max_scroll_obstruction_extent = max_scroll_obstruction_extent.into().unwrap_or(0.);
        let hit_test_extent = hit_test_extent.into().unwrap_or(paint_extent);
        let visible = visible.into().unwrap_or(paint_extent > 0.);
        let has_visual_overflow = has_visual_overflow.into().unwrap_or(false);
        let scroll_offset_correction = scroll_offset_correction.into().unwrap_or(0.);

        SliverGeometry {
            scroll_extent,
            paint_origin,
            paint_extent,
            layout_extent,
            max_paint_extent,
            max_scroll_obstruction_extent,
            hit_test_extent,
            visible,
            has_visual_overflow,
            scroll_offset_correction,
            cache_extent,
        }
    }
}
