use crate::box_constraints::BoxConstraints;
use crate::constraints::Constraints;

use crate::style::axis::Axis;
use crate::style::layout::{
    CrossAxisAlignment, FlexFit, MainAxisAlignment, MainAxisSize, TextDirection, VerticalDirection,
};
use crate::tree::Element;
use crate::{
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    lifecycle::LifeCycle,
    object::{Properties, RenderObject, RenderObjectInterface},
    tree::Children,
    ui::Ui,
};
use druid_shell::kurbo::{Point, Size};
use std::any::Any;
use std::panic::Location;

struct LayoutSize {
    main_size: f64,
    cross_size: f64,
    allocated_size: f64,
}

/// A widget that just adds padding around its child.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Flex {
    direction: Axis,
    main_axis_size: MainAxisSize,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    text_direction: TextDirection,
    vertical_direction: VerticalDirection,
}

impl Properties for Flex {
    type Object = RenderFlex;
}

impl Flex {
    pub fn new(
        direction: Axis,
        main_axis_size: MainAxisSize,
        main_axis_alignment: MainAxisAlignment,
        cross_axis_alignment: CrossAxisAlignment,
        text_direction: TextDirection,
        vertical_direction: VerticalDirection,
    ) -> Self {
        Flex {
            direction,
            main_axis_size,
            main_axis_alignment,
            cross_axis_alignment,
            text_direction,
            vertical_direction,
        }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        ui.render_object(Location::caller().into(), self, content);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Flexible {
    flex: f64,
    fit: FlexFit,
}

impl Flexible {
    pub fn new(flex: f64, fit: FlexFit) -> Self {
        Flexible { flex, fit }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        ui.set_parent_data(Some(Box::new(FlexParentData {
            flex: self.flex as f64,
            fit: self.fit,
        })));
        content(ui);
    }
}

#[derive(Debug, PartialEq, Clone)]
struct FlexParentData {
    flex: f64,
    fit: FlexFit,
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
pub(crate) struct RenderFlex {
    direction: Axis,
    main_axis_size: MainAxisSize,
    main_axis_alianment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    text_direction: TextDirection,
    vertical_direction: VerticalDirection,
}

impl RenderFlex {
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
        ctx: &mut LayoutCtx,
        constraints: &BoxConstraints,
        children: &mut Children,
        mut layout_child: impl FnMut(&mut LayoutCtx, &mut Element, &BoxConstraints) -> Size,
    ) -> LayoutSize {
        let mut total_flex = 0.0;
        let max_main_size = match self.direction {
            Axis::Horizontal => constraints.max_width(),
            Axis::Vertical => constraints.max_height(),
        };
        let can_flex = !max_main_size.is_infinite();
        let mut cross_size: f64 = 0.0;
        let mut allocated_size: f64 = 0.0;
        let total_child = children.len();
        for child in children.iter() {
            let flex = self.get_flex(child);
            if flex > 0.0 {
                total_flex += flex;
            } else {
                let inner_constrains = match (self.cross_axis_alignment, self.direction) {
                    (CrossAxisAlignment::Stretch, Axis::Horizontal) => {
                        BoxConstraints::tight_for(None, Some(constraints.max_height()))
                    }
                    (CrossAxisAlignment::Stretch, Axis::Vertical) => {
                        BoxConstraints::tight_for(Some(constraints.max_width()), None)
                    }
                    (_, Axis::Horizontal) => BoxConstraints::new(
                        Size::ZERO,
                        Size::new(f64::INFINITY, constraints.max_height()),
                    ),
                    (_, Axis::Vertical) => BoxConstraints::new(
                        Size::ZERO,
                        Size::new(constraints.max_width(), f64::INFINITY),
                    ),
                };
                let child_size = layout_child(ctx, child, &inner_constrains.into());
                allocated_size += self.get_main_size(child_size);
                cross_size = cross_size.max(self.get_cross_size(child_size));
            }
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
            for (i, child) in children.iter().enumerate() {
                let flex = self.get_flex(child);
                if flex > 0.0 {
                    let max_child_extent = if can_flex {
                        if i == total_child - 1 {
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
                        .parent_data::<FlexParentData>()
                        .map(|d| d.fit)
                        .unwrap_or(FlexFit::Loose);

                    let min_child_extent = match child_fit {
                        FlexFit::Tight => {
                            assert!(max_child_extent < f64::INFINITY);
                            max_child_extent
                        }
                        FlexFit::Loose => 0.0,
                    };

                    let inner_constrains = match (self.cross_axis_alignment, self.direction) {
                        (CrossAxisAlignment::Stretch, Axis::Horizontal) => BoxConstraints::new(
                            Size::new(min_child_extent, constraints.max_height()),
                            Size::new(max_child_extent, constraints.max_height()),
                        ),
                        (CrossAxisAlignment::Stretch, Axis::Vertical) => BoxConstraints::new(
                            Size::new(constraints.max_width(), min_child_extent),
                            Size::new(constraints.max_width(), max_child_extent),
                        ),
                        (_, Axis::Horizontal) => BoxConstraints::new(
                            Size::new(min_child_extent, 0.),
                            Size::new(max_child_extent, constraints.max_height()),
                        ),
                        (_, Axis::Vertical) => BoxConstraints::new(
                            Size::new(0.0, min_child_extent),
                            Size::new(constraints.max_width(), max_child_extent),
                        ),
                    };
                    let child_size = layout_child(ctx, child, &inner_constrains.into());
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

    fn get_flex(&self, child: &Element) -> f64 {
        child
            .parent_data::<FlexParentData>()
            .map(|d| d.flex)
            .unwrap_or(1.)
    }
}

impl RenderObject<Flex> for RenderFlex {
    type Action = ();

    fn create(props: Flex) -> Self {
        let Flex {
            direction,
            main_axis_size,
            main_axis_alignment: main_axis_alianment,
            cross_axis_alignment,
            text_direction,
            vertical_direction,
        } = props;

        RenderFlex {
            direction,
            main_axis_size,
            main_axis_alianment,
            cross_axis_alignment,
            text_direction,
            vertical_direction,
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: Flex) {
        let render_flex = Self::create(props);
        if self != &render_flex {
            *self = render_flex;
            ctx.request_layout();
        }
    }
}

impl RenderObjectInterface for RenderFlex {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        for child in children {
            child.event(ctx, event)
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn dry_layout(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &Constraints,
        children: &mut Children,
    ) -> Size {
        let bc = c.to_box();
        let sizes = self.compute_sizes(ctx, &bc, children, |ctx, child, bc| {
            child.dry_layout(ctx, &bc.into())
        });
        match self.direction {
            Axis::Horizontal => bc.constrain(Size::new(sizes.main_size, sizes.cross_size)),
            Axis::Vertical => bc.constrain(Size::new(sizes.cross_size, sizes.main_size)),
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children) -> Size {
        let constraints: BoxConstraints = c.into();
        constraints.debug_check("Flex");
        let LayoutSize {
            allocated_size,
            main_size: mut actual_size,
            mut cross_size,
        } = self.compute_sizes(ctx, &constraints, children, |ctx, child, bc| {
            child.layout(ctx, &bc.into())
        });

        if self.cross_axis_alignment == CrossAxisAlignment::Baseline {
            unimplemented!("baseline");
        }

        let size = match self.direction {
            Axis::Horizontal => {
                let size = constraints.constrain(Size::new(actual_size, cross_size));
                actual_size = size.width;
                cross_size = size.height;
                size
            }
            Axis::Vertical => {
                let size = constraints.constrain(Size::new(cross_size, actual_size));
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
        let child_count = children.len();
        let leading_space;
        let between_space;
        match self.main_axis_alianment {
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
        for child in children {
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
                        cross_size - self.get_cross_size(child.size())
                    }
                }
                CrossAxisAlignment::Center => {
                    cross_size / 2.0 - self.get_cross_size(child.size()) / 2.0
                }
                CrossAxisAlignment::Stretch => 0.0,
                CrossAxisAlignment::Baseline => unimplemented!(),
            };
            if flip_main_axis {
                child_main_position -= self.get_main_size(child.size());
            }
            match self.direction {
                Axis::Horizontal => {
                    child.set_origin(ctx, Point::new(child_main_position, child_cross_position));
                }
                Axis::Vertical => {
                    child.set_origin(ctx, Point::new(child_cross_position, child_main_position));
                }
            }
            if flip_main_axis {
                child_main_position -= between_space;
            } else {
                child_main_position += self.get_main_size(child.size()) + between_space;
            }
        }
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        for child in children {
            child.paint(ctx);
        }
    }
}
