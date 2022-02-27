use std::panic::Location;

use crate::{
    object::{Properties, RenderObject, RenderObjectInterface},
    style::{alignment::Alignment, layout::TextDirection},
    ui::Ui,
};

#[derive(Debug, PartialEq, Clone)]
struct Align {
    alignment: Alignment,
    width_factor: f64,
    height_factor: f64,
}

impl Align {
    pub fn new(alignment: Alignment, width_factor: f64, height_factor: f64) -> Self {
        Align {
            alignment,
            width_factor,
            height_factor,
        }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnMut(&mut Ui)) {
        ui.render_object(Location::caller().into(), self, content)
    }
}

impl Properties for Align {
    type Object = RenderAlign;
}

struct RenderAlign {
    alignment: Alignment,
    width_factor: f64,
    height_factor: f64,
}

impl RenderAlign {}

impl RenderObject<Align> for RenderAlign {
    type Action = ();

    fn create(props: Align) -> Self {
        RenderAlign {
            alignment: props.alignment,
            width_factor: props.width_factor,
            height_factor: props.height_factor,
        }
    }

    fn update(&mut self, ctx: &mut crate::context::UpdateCtx, props: Align) -> Self::Action {
        todo!()
    }
}

impl RenderObjectInterface for RenderAlign {
    fn event(
        &mut self,
        ctx: &mut crate::context::EventCtx,
        event: &crate::event::Event,
        children: &mut crate::tree::Children,
    ) {
        todo!()
    }

    fn lifecycle(
        &mut self,
        ctx: &mut crate::context::LifeCycleCtx,
        event: &crate::lifecycle::LifeCycle,
        children: &mut crate::tree::Children,
    ) {
        todo!()
    }

    fn dry_layout(
        &mut self,
        ctx: &mut crate::context::LayoutCtx,
        c: &crate::constraints::Constraints,
        children: &mut crate::tree::Children,
    ) -> druid_shell::kurbo::Size {
        todo!()
    }

    fn paint(&mut self, ctx: &mut crate::context::PaintCtx, children: &mut crate::tree::Children) {
        todo!()
    }
}
