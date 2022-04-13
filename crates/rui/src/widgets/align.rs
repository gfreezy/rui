use std::panic::Location;

use druid_shell::kurbo::Size;

use crate::{
    key::Key,
    object::{Properties, RenderObject, RenderObjectInterface},
    style::{alignment::Alignment, layout::TextDirection},
    tree::Children,
    ui::Ui,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Align {
    alignment: Alignment,
    width_factor: Option<f64>,
    height_factor: Option<f64>,
    text_direction: TextDirection,
}

impl Align {
    pub fn new(
        alignment: Alignment,
        width_factor: Option<f64>,
        height_factor: Option<f64>,
        text_direction: TextDirection,
    ) -> Self {
        Align {
            alignment,
            width_factor,
            height_factor,
            text_direction,
        }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnMut(&mut Ui)) {
        ui.render_object(Key::current(), self, content)
    }
}

impl Properties for Align {
    type Object = RenderAlign;
}

#[derive(PartialEq)]
pub struct RenderAlign {
    alignment: Alignment,
    width_factor: Option<f64>,
    height_factor: Option<f64>,
    text_direction: TextDirection,
}

impl RenderAlign {}

impl RenderObject<Align> for RenderAlign {
    type Action = ();

    fn create(props: Align) -> Self {
        RenderAlign {
            alignment: props.alignment,
            width_factor: props.width_factor,
            height_factor: props.height_factor,
            text_direction: props.text_direction,
        }
    }

    fn update(
        &mut self,
        ctx: &mut crate::context::UpdateCtx,
        props: Align,
        children: &mut Children,
    ) -> Self::Action {
        let new = Self::create(props);
        if self != &new {
            *self = new;
            ctx.request_layout();
        }
    }
}

impl RenderObjectInterface for RenderAlign {
    fn event(
        &mut self,
        ctx: &mut crate::context::EventCtx,
        event: &crate::event::Event,
        children: &mut crate::tree::Children,
    ) {
        children[0].event(ctx, event)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut crate::context::LifeCycleCtx,
        event: &crate::lifecycle::LifeCycle,
        children: &mut crate::tree::Children,
    ) {
        children[0].lifecycle(ctx, event)
    }

    fn dry_layout_box(
        &mut self,
        ctx: &mut crate::context::LayoutCtx,
        bc: &crate::constraints::BoxConstraints,
        children: &mut crate::tree::Children,
    ) -> druid_shell::kurbo::Size {
        let shrink_wrap_width = self.width_factor.is_some() || bc.max_width().is_infinite();
        let shrink_wrap_height = self.height_factor.is_some() || bc.max_height().is_infinite();
        if children.is_empty() {
            return bc.constrain(Size::new(
                if shrink_wrap_width { 0. } else { f64::INFINITY },
                if shrink_wrap_height {
                    0.0
                } else {
                    f64::INFINITY
                },
            ));
        }
        let child_size = children[0].dry_layout_box(ctx, &bc.loosen().into());
        bc.constrain(Size::new(
            if shrink_wrap_width {
                child_size.width * self.width_factor.unwrap_or(1.0)
            } else {
                f64::INFINITY
            },
            if shrink_wrap_height {
                child_size.height * self.height_factor.unwrap_or(1.0)
            } else {
                f64::INFINITY
            },
        ))
    }

    fn layout_box(
        &mut self,
        ctx: &mut crate::context::LayoutCtx,
        bc: &crate::constraints::BoxConstraints,
        children: &mut crate::tree::Children,
    ) -> Size {
        let shrink_wrap_width = self.width_factor.is_some() || bc.max_width().is_infinite();
        let shrink_wrap_height = self.height_factor.is_some() || bc.max_height().is_infinite();
        if children.is_empty() {
            return bc.constrain(Size::new(
                if shrink_wrap_width { 0. } else { f64::INFINITY },
                if shrink_wrap_height {
                    0.0
                } else {
                    f64::INFINITY
                },
            ));
        }

        let child_size = children[0].layout_box(ctx, &bc.loosen().into(), true);
        let size = bc.constrain(Size::new(
            if shrink_wrap_width {
                child_size.width * self.width_factor.unwrap_or(1.0)
            } else {
                f64::INFINITY
            },
            if shrink_wrap_height {
                child_size.height * self.height_factor.unwrap_or(1.0)
            } else {
                f64::INFINITY
            },
        ));
        let child_pos = self
            .alignment
            .resolve(self.text_direction)
            .along_offset((size - child_size).to_vec2());
        children[0].set_origin(ctx, child_pos);

        size
    }

    fn paint(&mut self, ctx: &mut crate::context::PaintCtx, children: &mut crate::tree::Children) {
        children[0].paint(ctx)
    }
}
