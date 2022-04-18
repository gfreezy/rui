use druid_shell::kurbo::Vec2;

use crate::{
    sliver_constraints::{axis_direction_to_axis, AxisDirection, ScrollDirection},
    style::axis::Axis,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScrollMetrics {
    pub(crate) min_scroll_extent: f64,
    pub(crate) max_scroll_extent: f64,
    pub(crate) pixels: f64,
    pub(crate) has_pixels: bool,
    pub(crate) viewport_dimension: f64,
    pub(crate) has_viewport_dimension: bool,
    pub(crate) axis_direction: AxisDirection,
    pub(crate) user_scroll_direction: ScrollDirection,
    pub(crate) implied_velocity: f64,
}

pub trait IScrollMetrics {
    fn base(&self) -> &dyn IScrollMetrics;
    fn base_mut(&mut self) -> &mut dyn IScrollMetrics;

    fn pixels(&self) -> f64 {
        self.base().pixels()
    }

    fn scroll_offset(&self) -> Vec2 {
        self.base().scroll_offset()
    }

    fn user_scroll_direction(&self) -> ScrollDirection {
        self.base().user_scroll_direction()
    }

    fn pointer_scroll(&mut self, delta: f64) {
        self.base_mut().pointer_scroll(delta)
    }

    fn min_scroll_extent(&self) -> f64 {
        self.base().min_scroll_extent()
    }

    fn max_scroll_extent(&self) -> f64 {
        self.base().max_scroll_extent()
    }

    fn axis(&self) -> Axis {
        self.base().axis()
    }

    fn out_of_range(&self) -> bool {
        self.base().out_of_range()
    }

    fn at_edge(&self) -> bool {
        self.base().at_edge()
    }
    fn extent_before(&self) -> f64 {
        self.base().extent_before()
    }
    fn extent_inside(&self) -> f64 {
        self.base().extent_inside()
    }

    fn extent_after(&self) -> f64 {
        self.base().extent_after()
    }

    fn correct_by(&mut self, correction: f64) {
        self.base_mut().correct_by(correction)
    }

    fn correct_pixels(&mut self, pixels: f64) {
        self.base_mut().correct_pixels(pixels)
    }
}

impl IScrollMetrics for ScrollMetrics {
    fn base(&self) -> &dyn IScrollMetrics {
        self
    }

    fn base_mut(&mut self) -> &mut dyn IScrollMetrics {
        self
    }

    fn user_scroll_direction(&self) -> ScrollDirection {
        self.user_scroll_direction
    }

    fn pixels(&self) -> f64 {
        self.pixels
    }
    fn scroll_offset(&self) -> Vec2 {
        match self.axis() {
            Axis::Horizontal => Vec2::new(self.pixels, 0.0),
            Axis::Vertical => Vec2::new(0.0, self.pixels),
        }
    }

    fn pointer_scroll(&mut self, delta: f64) {
        assert!(delta != 0.0);

        let target_pixels =
            ((self.pixels + delta).max(self.min_scroll_extent)).min(self.max_scroll_extent);
        if target_pixels != self.pixels {
            // update user scroll direction
            let user_scroll_direction = if -delta > 0.0 {
                ScrollDirection::Forward
            } else {
                ScrollDirection::Reverse
            };
            if self.user_scroll_direction != user_scroll_direction {
                self.user_scroll_direction = user_scroll_direction;
            }

            self.implied_velocity = target_pixels - self.pixels;
            // todo: clear implied_velocity after current frame.
            self.pixels = target_pixels;
        }
    }

    fn min_scroll_extent(&self) -> f64 {
        self.min_scroll_extent
    }

    fn max_scroll_extent(&self) -> f64 {
        self.max_scroll_extent
    }

    fn axis(&self) -> Axis {
        axis_direction_to_axis(self.axis_direction)
    }

    fn out_of_range(&self) -> bool {
        self.pixels < self.min_scroll_extent || self.pixels > self.max_scroll_extent
    }

    fn at_edge(&self) -> bool {
        self.pixels == self.min_scroll_extent || self.pixels == self.max_scroll_extent
    }

    fn extent_before(&self) -> f64 {
        (self.pixels - self.min_scroll_extent).max(0.0)
    }

    fn extent_inside(&self) -> f64 {
        self.viewport_dimension
            - (self.min_scroll_extent - self.pixels).clamp(0., self.viewport_dimension)
            - (self.pixels - self.max_scroll_extent).clamp(0., self.viewport_dimension)
    }

    fn extent_after(&self) -> f64 {
        (self.max_scroll_extent - self.pixels).max(0.0)
    }

    fn correct_by(&mut self, correction: f64) {
        self.pixels += correction;
    }

    fn correct_pixels(&mut self, pixels: f64) {
        self.pixels = pixels;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixedScrollMetrics {
    scroll_metrics: ScrollMetrics,
}

impl IScrollMetrics for FixedScrollMetrics {
    fn base(&self) -> &dyn IScrollMetrics {
        &self.scroll_metrics
    }

    fn base_mut(&mut self) -> &mut dyn IScrollMetrics {
        &mut self.scroll_metrics
    }
}
