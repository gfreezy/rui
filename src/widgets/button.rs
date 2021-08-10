use std::panic::Location;

use crate::box_constraints::BoxConstraints;
use crate::lifecycle::LifeCycle;
use crate::{
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    object::{Properties, RenderObject, RenderObjectInterface},
    tree::Children,
    ui::Ui,
    widgets::label::Label,
};
use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::{Color, PaintBrush, RenderContext};
use druid_shell::MouseButton;

pub struct Button {
    disabled: bool,
    handler: Box<dyn FnMut()>,
}

impl PartialEq for Button {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl Default for Button {
    fn default() -> Self {
        Button {
            disabled: false,
            handler: Box::new(|| {}),
        }
    }
}

impl Properties for Button {
    type Object = ButtonObject;
}

impl Button {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn handler(mut self, h: impl FnMut() + 'static) -> Self {
        self.handler = Box::new(h);
        self
    }

    #[track_caller]
    pub fn labeled(self, ui: &mut Ui, label: impl Into<String>, handler: impl FnMut() + 'static) {
        let caller = Location::caller().into();
        ui.render_object(caller, self.handler(handler), |ui| {
            Label::new(label).build(ui);
        })
    }

    #[track_caller]
    pub fn custom(
        self,
        ui: &mut Ui,
        handler: impl FnMut() + 'static,
        content: impl FnOnce(&mut Ui),
    ) {
        let caller = Location::caller().into();
        ui.render_object(caller, self.handler(handler), content);
    }
}

pub enum ButtonAction {
    Clicked,
}

#[derive(Default)]
pub struct ButtonObject {
    props: Button,
    label_size: Size,
}

impl RenderObject<Button> for ButtonObject {
    type Action = ();

    fn create(props: Button) -> Self {
        ButtonObject {
            props,
            label_size: Size::ZERO,
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: Button) -> Self::Action {
        if self.props != props {
            ctx.request_layout();
            self.props = props;
        }
    }
}

impl RenderObjectInterface for ButtonObject {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        match event {
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    ctx.set_active(true);
                }
            }
            Event::MouseUp(mouse_event) => {
                if ctx.is_active() && mouse_event.button == MouseButton::Left {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        (*self.props.handler)();
                        ctx.set_handled();
                    }
                }
            }
            _ => {}
        }

        for child in children {
            child.event(ctx, event);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle) {
        if let LifeCycle::HotChanged(_) = event {}
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        bc.debug_check("Button");

        let padding = Size::new(2.0, 2.0);
        let label_bc = bc.loosen().shrink(padding);
        self.label_size = children[0].layout(ctx, &label_bc);

        let required_size = self.label_size + padding;
        let size = bc.constrain(required_size);

        let h_offset = (size.width - self.label_size.width) / 2.0;
        let v_offset = (size.height - self.label_size.height) / 2.0;
        children[0].set_origin(ctx, Point::new(h_offset, v_offset));

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        let size = ctx.size();
        let stroke_width = 1.0;

        let rounded_rect = size
            .to_rect()
            .inset(-stroke_width / 2.0)
            .to_rounded_rect(1.0);

        let border_color = PaintBrush::Color(Color::rgb8(0, 0, 0));

        ctx.stroke(rounded_rect, &border_color, stroke_width);

        ctx.fill(rounded_rect, &PaintBrush::Color(Color::rgb8(255, 255, 255)));
        children[0].paint(ctx);
    }
}
