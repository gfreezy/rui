use std::panic::Location;

use druid_shell::kurbo::Size;
use druid_shell::piet::{Color, PaintBrush, RenderContext};

use crate::box_constraints::BoxConstraints;
use crate::key::Key;
use crate::style::draw;
use crate::{
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    lifecycle::LifeCycle,
    object::{Properties, RenderObject, RenderObjectInterface},
    tree::Children,
    ui::Ui,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Background {
    background: draw::Background,
}

impl Properties for Background {
    type Object = Self;
}

impl Background {
    pub fn new(background: draw::Background) -> Self {
        Background { background }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        ui.render_object(Key::current(), self, content);
    }
}

impl RenderObject<Background> for Background {
    type Action = ();

    fn create(props: Background) -> Self {
        props
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: Background, children: &mut Children) {
        if self != &props {
            *self = props;
            ctx.request_layout();
        }
    }
}

impl RenderObjectInterface for Background {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        children[0].event(ctx, event)
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        if self.background.color != Color::TRANSPARENT {
            let paint_rect = ctx.child_state.layout_rect();
            let paint_brush: PaintBrush = self.background.color.clone().into();

            ctx.fill(paint_rect, &paint_brush);
        }
        children[0].paint(ctx);
    }

    fn dry_layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        children[0].layout_box(ctx, bc)
    }
}
