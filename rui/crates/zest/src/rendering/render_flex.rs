use std::fmt::format;

use crate::{
    arithmatic::{near_equal, Tolerance},
    render_object::{
        render_box::{BoxConstraints, RenderBoxWidget, Size},
        render_object::{Offset, RenderObject},
    },
};
use style::{
    axis::Axis,
    layout::{
        CrossAxisAlignment, FlexFit, MainAxisAlignment, MainAxisSize, TextDirection,
        VerticalDirection,
    },
};

struct LayoutSize {
    main_size: f64,
    cross_size: f64,
    allocated_size: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flexible {
    flex: f64,
    fit: FlexFit,
}

impl Flexible {
    pub fn new(flex: f64, fit: FlexFit) -> Self {
        Flexible { flex, fit }
    }
}

#[derive(Debug, Clone)]
struct FlexParentData {
    flex: f64,
    fit: FlexFit,
}

impl PartialEq for FlexParentData {
    fn eq(&self, other: &Self) -> bool {
        near_equal(self.flex, other.flex, Tolerance::DEFAULT.distance) && self.fit == other.fit
    }
}

impl Default for FlexParentData {
    fn default() -> Self {
        Self {
            flex: 1.,
            fit: FlexFit::Loose,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RenderFlex {
    direction: Axis,
    main_axis_size: MainAxisSize,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    text_direction: TextDirection,
    vertical_direction: VerticalDirection,
}

impl Default for RenderFlex {
    fn default() -> Self {
        Self::new(
            Axis::Vertical,
            MainAxisSize::Max,
            MainAxisAlignment::Start,
            CrossAxisAlignment::Start,
            TextDirection::Ltr,
            VerticalDirection::Down,
        )
    }
}

impl RenderFlex {
    pub fn new(
        direction: Axis,
        main_axis_size: MainAxisSize,
        main_axis_alignment: MainAxisAlignment,
        cross_axis_alignment: CrossAxisAlignment,
        text_direction: TextDirection,
        vertical_direction: VerticalDirection,
    ) -> Self {
        RenderFlex {
            direction,
            main_axis_size,
            main_axis_alignment,
            cross_axis_alignment,
            text_direction,
            vertical_direction,
        }
    }

    fn get_main_size(&self, size: Size) -> f64 {
        match self.direction {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn get_cross_size(&self, size: Size) -> f64 {
        match self.direction {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        }
    }

    fn start_is_top_left(
        &self,
        direction: Axis,
        text_direction: Option<TextDirection>,
        vertical_direction: Option<VerticalDirection>,
    ) -> Option<bool> {
        match (direction, text_direction, vertical_direction) {
            (Axis::Horizontal, Some(TextDirection::Ltr), _) => Some(true),
            (Axis::Horizontal, Some(TextDirection::Rtl), _) => Some(false),
            (Axis::Vertical, _, Some(VerticalDirection::Down)) => Some(true),
            (Axis::Vertical, _, Some(VerticalDirection::Up)) => Some(false),
            (_, _, _) => None,
        }
    }

    fn compute_sizes(
        &self,
        ctx: &RenderObject,
        constraints: &BoxConstraints,
        mut layout_child: impl FnMut(&RenderObject, &BoxConstraints) -> Size,
    ) -> LayoutSize {
        let mut total_flex = 0.0;
        let max_main_size = match self.direction {
            Axis::Horizontal => constraints.max_width,
            Axis::Vertical => constraints.max_height,
        };
        let can_flex = !max_main_size.is_infinite();
        let mut cross_size: f64 = 0.0;
        let mut allocated_size: f64 = 0.0;
        let mut next_child = ctx.try_first_child();
        while let Some(child) = next_child {
            let flex = self.get_flex(&child);
            if flex > 0.0 {
                total_flex += flex;
            } else {
                let inner_constrains = match (self.cross_axis_alignment, self.direction) {
                    (CrossAxisAlignment::Stretch, Axis::Horizontal) => {
                        BoxConstraints::tight_for(None, Some(constraints.max_height))
                    }
                    (CrossAxisAlignment::Stretch, Axis::Vertical) => {
                        BoxConstraints::tight_for(Some(constraints.max_width), None)
                    }
                    (_, Axis::Horizontal) => BoxConstraints {
                        min_width: 0.,
                        max_width: f64::INFINITY,
                        min_height: 0.,
                        max_height: constraints.max_height,
                    },
                    (_, Axis::Vertical) => BoxConstraints {
                        min_width: 0.,
                        max_width: constraints.max_width,
                        min_height: 0.,
                        max_height: f64::INFINITY,
                    },
                };
                let child_size = layout_child(&child, &inner_constrains.into());
                allocated_size += self.get_main_size(child_size);
                cross_size = cross_size.max(self.get_cross_size(child_size));
            }
            next_child = child.try_next_sibling();
        }

        let free_space = if can_flex {
            (max_main_size - allocated_size).max(0.0)
        } else {
            0.
        };
        let mut allocated_flex_space = 0.0;
        if total_flex > 0.0 {
            let space_per_flex = if can_flex {
                (free_space / total_flex).ceil()
            } else {
                f64::NAN
            };
            let mut next_child = ctx.try_first_child();
            while let Some(child) = next_child {
                let flex = self.get_flex(&child);
                if flex > 0.0 {
                    let max_child_extent = if can_flex {
                        if child == ctx.last_child() {
                            // last child
                            free_space - allocated_flex_space
                        } else {
                            space_per_flex * flex
                        }
                    } else {
                        f64::INFINITY
                    };

                    // get child fit
                    let child_fit = child
                        .with_parent_data::<FlexParentData, _>(|p| p.fit)
                        .unwrap_or(FlexFit::Loose);

                    let min_child_extent = match child_fit {
                        FlexFit::Tight => {
                            assert!(max_child_extent < f64::INFINITY);
                            max_child_extent
                        }
                        FlexFit::Loose => 0.0,
                    };

                    let inner_constrains = match (self.cross_axis_alignment, self.direction) {
                        (CrossAxisAlignment::Stretch, Axis::Horizontal) => BoxConstraints {
                            min_width: min_child_extent,
                            max_width: max_child_extent,
                            min_height: constraints.max_height,
                            max_height: constraints.max_height,
                        },
                        (CrossAxisAlignment::Stretch, Axis::Vertical) => BoxConstraints {
                            min_width: constraints.max_width,
                            max_width: constraints.max_width,
                            min_height: min_child_extent,
                            max_height: max_child_extent,
                        },

                        (_, Axis::Horizontal) => BoxConstraints {
                            min_width: min_child_extent,
                            max_width: max_child_extent,
                            min_height: 0.,
                            max_height: constraints.max_height,
                        },
                        (_, Axis::Vertical) => BoxConstraints {
                            min_width: 0.,
                            max_width: constraints.max_width,
                            min_height: min_child_extent,
                            max_height: max_child_extent,
                        },
                    };
                    let child_size = layout_child(&child, &inner_constrains.into());
                    let child_main_size = self.get_main_size(child_size);
                    assert!(
                        child_main_size <= max_child_extent,
                        "child_main_size: {}, max_child_extent: {}",
                        child_main_size,
                        max_child_extent
                    );
                    allocated_size += child_main_size;
                    allocated_flex_space += max_child_extent;
                    cross_size = cross_size.max(self.get_cross_size(child_size));
                }
                next_child = child.try_next_sibling();
            }
        }
        let ideal_size = if can_flex && self.main_axis_size == MainAxisSize::Max {
            max_main_size
        } else {
            allocated_size
        };

        LayoutSize {
            main_size: ideal_size,
            cross_size,
            allocated_size,
        }
    }

    fn get_flex(&self, child: &RenderObject) -> f64 {
        child
            .with_parent_data::<FlexParentData, _>(|p| p.flex)
            .unwrap_or(1.)
    }
}

impl RenderBoxWidget for RenderFlex {
    fn compute_dry_layout(&mut self, ctx: &RenderObject, bc: BoxConstraints) -> Size {
        let sizes = self.compute_sizes(ctx, &bc, |child, bc| {
            child.render_box().get_dry_layout(bc.clone())
        });
        match self.direction {
            Axis::Horizontal => bc.constrain(Size::new(sizes.main_size, sizes.cross_size)),
            Axis::Vertical => bc.constrain(Size::new(sizes.cross_size, sizes.main_size)),
        }
    }

    fn perform_layout(&mut self, ctx: &RenderObject) {
        let constraints = ctx.constraints();
        let bc = constraints.box_constraints();
        let LayoutSize {
            allocated_size,
            main_size: mut actual_size,
            mut cross_size,
        } = self.compute_sizes(ctx, &bc, |child, bc| {
            child.layout(bc.clone().into(), true);
            child.render_box().size()
        });

        if self.cross_axis_alignment == CrossAxisAlignment::Baseline {
            unimplemented!("baseline");
        }

        let size = match self.direction {
            Axis::Horizontal => {
                let size = bc.constrain(Size::new(actual_size, cross_size));
                actual_size = size.width;
                cross_size = size.height;
                size
            }
            Axis::Vertical => {
                let size = bc.constrain(Size::new(cross_size, actual_size));
                actual_size = size.height;
                cross_size = size.width;
                size
            }
        };
        let actual_size_delta = actual_size - allocated_size;
        let remaining_space = actual_size_delta.max(0.);
        let flip_main_axis = !self
            .start_is_top_left(
                self.direction,
                Some(self.text_direction),
                Some(self.vertical_direction),
            )
            .unwrap_or(true);
        let child_count = ctx.child_count();
        let leading_space;
        let between_space;
        match self.main_axis_alignment {
            MainAxisAlignment::Start => {
                leading_space = 0.;
                between_space = 0.;
            }
            MainAxisAlignment::End => {
                leading_space = remaining_space;
                between_space = 0.;
            }
            MainAxisAlignment::Center => {
                leading_space = remaining_space / 2.;
                between_space = 0.;
            }
            MainAxisAlignment::SpaceBetween => {
                leading_space = 0.;
                between_space = if child_count > 1 {
                    remaining_space / (child_count - 1) as f64
                } else {
                    0.
                };
            }
            MainAxisAlignment::SpaceAround => {
                between_space = if child_count > 0 {
                    remaining_space / child_count as f64
                } else {
                    0.
                };
                leading_space = between_space / 2.;
            }
            MainAxisAlignment::SpaceEvenly => {
                between_space = if child_count > 0 {
                    remaining_space / (child_count + 1) as f64
                } else {
                    0.
                };
                leading_space = between_space;
            }
        };

        let mut child_main_position = if flip_main_axis {
            actual_size - leading_space
        } else {
            leading_space
        };
        let mut next_child = ctx.try_first_child();
        while let Some(child) = next_child {
            let child_size = child.render_box().size();
            let child_cross_position = match self.cross_axis_alignment {
                CrossAxisAlignment::Start | CrossAxisAlignment::End => {
                    if self.start_is_top_left(
                        self.direction.flip(),
                        Some(self.text_direction),
                        Some(self.vertical_direction),
                    ) == Some(self.cross_axis_alignment == CrossAxisAlignment::Start)
                    {
                        0.
                    } else {
                        cross_size - self.get_cross_size(child_size)
                    }
                }
                CrossAxisAlignment::Center => {
                    cross_size / 2.0 - self.get_cross_size(child_size) / 2.0
                }
                CrossAxisAlignment::Stretch => 0.0,
                CrossAxisAlignment::Baseline => unimplemented!(),
            };
            if flip_main_axis {
                child_main_position -= self.get_main_size(child_size);
            }
            match self.direction {
                Axis::Horizontal => {
                    child
                        .render_box()
                        .set_offset(Offset::new(child_main_position, child_cross_position));
                }
                Axis::Vertical => {
                    child
                        .render_box()
                        .set_offset(Offset::new(child_cross_position, child_main_position));
                }
            }
            if flip_main_axis {
                child_main_position -= between_space;
            } else {
                child_main_position += self.get_main_size(child_size) + between_space;
            }
            next_child = child.try_next_sibling();
        }
        ctx.render_box().set_size(size);
    }

    fn is_repaint_boundary(&self) -> bool {
        false
    }

    fn sized_by_parent(&self) -> bool {
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
