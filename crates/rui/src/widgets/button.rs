use std::panic::Location;

use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::{Color, PaintBrush, RenderContext};
use druid_shell::MouseButton;
use tracing::debug;

use crate::box_constraints::BoxConstraints;
use crate::constraints::Constraints;
use crate::lifecycle::LifeCycle;
use crate::{
    context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx},
    event::Event,
    object::{RenderObject, RenderObjectInterface},
    tree::Children,
    ui::Ui,
    widgets::text::Text,
};

pub struct Button {
    disabled: bool,
    handler: Box<dyn FnMut() + 'static>,
}

impl PartialEq for Button {
    fn eq(&self, other: &Self) -> bool {
        self.disabled == other.disabled
    }
}

impl Button {
    pub fn new() -> Self {
        Button {
            disabled: false,
            handler: Box::new(|| {}),
        }
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
        ui.render_object::<_, ButtonObject, _>(caller, self.handler(handler), |ui| {
            Text::new(label).build(ui);
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
        ui.render_object::<_, ButtonObject, _>(caller, self.handler(handler), content);
    }
}

pub enum ButtonAction {
    Clicked,
}

pub struct ButtonObject {
    props: Button,
    label_size: Size,
    border_color: Color,
    background_color: Color,
}

impl RenderObject for ButtonObject {
    type Props = Button;
    type Action = ();

    fn create(props: Button) -> Self {
        ButtonObject {
            props,
            label_size: Size::ZERO,
            border_color: Color::BLACK,
            background_color: Color::GREEN,
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: Button) -> Self::Action {
        if self.props != props {
            ctx.request_layout();
        }
        self.props = props;
    }
}

impl RenderObjectInterface for ButtonObject {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        match event {
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    ctx.set_active(true);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse_event) => {
                if ctx.is_active() && mouse_event.button == MouseButton::Left {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        (*self.props.handler)();
                        ctx.request_update();
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

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _children: &mut Children) {
        match event {
            LifeCycle::HotChanged(hot) => {
                if *hot {
                    self.background_color = Color::RED;
                    debug!("on hover");
                } else {
                    self.background_color = Color::WHITE;
                    debug!("off hover");
                }
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, children: &mut Children) -> Size {
        let bc: BoxConstraints = c.into();
        bc.debug_check("Button");

        let padding = Size::new(2.0, 2.0);
        let label_c: Constraints = bc.loosen().shrink(padding).into();
        self.label_size = children[0].layout(ctx, &label_c);

        let required_size = self.label_size + padding;
        let size = bc.constrain(required_size);

        let h_offset = (size.width - self.label_size.width) / 2.0;
        let v_offset = (size.height - self.label_size.height) / 2.0;
        children[0].set_origin(ctx, Point::new(h_offset, v_offset));

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        let size = ctx.size();
        let stroke_width = 2.0;

        let rect = size.to_rect().inset(-stroke_width / 2.0);

        let border_color = PaintBrush::Color(self.border_color.clone());

        ctx.stroke(rect, &border_color, stroke_width);

        ctx.fill(rect, &PaintBrush::Color(self.background_color.clone()));
        debug!(
            "fill {:?}, layout rect: {:?}",
            rect,
            ctx.child_state.layout_rect()
        );
        children[0].paint(ctx);
    }

    fn dry_layout(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &Constraints,
        children: &mut Children,
    ) -> Size {
        let bc: BoxConstraints = c.into();
        bc.debug_check("Button");

        let padding = Size::new(2.0, 2.0);
        let label_c: Constraints = bc.loosen().shrink(padding).into();
        let label_size = children[0].dry_layout(ctx, &label_c);

        let required_size = label_size + padding;
        let size = bc.constrain(required_size);

        size
    }
}
