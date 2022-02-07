use std::panic::Location;

use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::{Color, PaintBrush, RenderContext};
use tracing::debug;

use crate::box_constraints::BoxConstraints;
use crate::constraints::Constraints;
use crate::style::alignment::{HorizontalAlignment, VerticalAlignment};
use crate::style::Style;

use crate::style::axis::Axis;
use crate::ui;
use crate::{
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    lifecycle::LifeCycle,
    object::{Properties, RenderObject, RenderObjectInterface},
    tree::Children,
    ui::Ui,
};

use super::background::Background;
use super::padding::Padding;
use super::sized_box::SizedBox;

/// A widget that just adds padding around its child.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Flex {
    style: Style,
}

impl Properties for Flex {
    type Object = FlexObject;
}

impl Flex {
    pub fn new(style: Style) -> Self {
        Flex { style }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let style = self.style.clone();
        let insets = style.insets;
        let background = style.background.clone();
        ui.render_object(
            Location::caller().into(),
            Background::new(background),
            |ui| {
                ui.render_object(
                    Location::caller().into(),
                    SizedBox::new(
                        style.width,
                        style.height,
                        style.min_width,
                        style.max_width,
                        style.min_height,
                        style.max_height,
                    ),
                    |ui| {
                        ui.render_object(Location::caller().into(), Padding::new(insets), |ui| {
                            ui.render_object(Location::caller().into(), self, content)
                        });
                    },
                )
            },
        );
    }
}

pub(crate) struct FlexObject {
    style: Style,
}

impl RenderObject<Flex> for FlexObject {
    type Action = ();

    fn create(props: Flex) -> Self {
        FlexObject { style: props.style }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: Flex) {
        if &self.style != &props.style {
            self.style = props.style;
            ctx.request_layout();
        }
    }
}

impl RenderObjectInterface for FlexObject {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        for child in children {
            child.event(ctx, event)
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children) -> Size {
        let bc: BoxConstraints = c.into();
        bc.debug_check("Flex");

        let mut child_bc = bc.clone().loosen();
        let mut total_width: f64 = 0.;
        let mut total_height: f64 = 0.;
        match self.style.axis {
            Axis::Horizontal => {
                for (i, child) in children.iter().enumerate() {
                    let size = child.layout(ctx, &(child_bc.into()));
                    total_width += size.width;
                    if i != 0 {
                        total_width += self.style.spacing.value();
                    }
                    total_height = total_height.max(size.height);
                    child_bc = child_bc.shrink((total_width, 0.));
                }

                let mut x = 0.;

                for child in children.iter() {
                    let child_size = child.size();

                    let y = match self.style.vertical_alignment {
                        VerticalAlignment::Bottom => total_height - child_size.height,
                        VerticalAlignment::Center => (total_height - child_size.height) / 2.,
                        VerticalAlignment::Top => 0.,
                    };
                    child.set_origin(ctx, Point::new(x, y));

                    x += self.style.spacing.value() + child_size.width;
                }
                Size::new(total_width, total_height)
            }
            Axis::Vertical => {
                for (i, child) in children.iter().enumerate() {
                    let size = child.layout(ctx, &(child_bc.into()));
                    total_height += size.height;
                    if i != 0 {
                        total_height += self.style.spacing.value();
                    }
                    total_width = total_width.max(size.width);
                    child_bc = bc.shrink((0.0, total_height));
                }

                let mut y = 0.;
                for child in children.iter() {
                    let child_size = child.size();
                    let x = match self.style.horizontal_alignment {
                        HorizontalAlignment::Start => 0.,
                        HorizontalAlignment::Center => (total_width - child_size.width) / 2.,
                        HorizontalAlignment::End => total_width - child_size.width,
                    };

                    child.set_origin(ctx, Point::new(x, y));

                    y += self.style.spacing.value() + child_size.height;
                }

                Size::new(total_width, total_height)
            }
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        for child in children {
            child.paint(ctx);
        }
    }
}
