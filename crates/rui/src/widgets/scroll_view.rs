use std::panic::Location;

use druid_shell::kurbo::{Point, Rect, Size, Vec2};
use druid_shell::piet::{Color, PaintBrush, RenderContext};

use crate::box_constraints::BoxConstraints;

use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx};
use crate::event::Event;
use crate::lifecycle::LifeCycle;
use crate::object::{Properties, RenderObject, RenderObjectInterface};
use crate::tree::Children;
use crate::ui::Ui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollView {
    content_offset: Point,
    viewport: Size,
}

impl ScrollView {
    pub fn new(content_offset: Point, viewport: Size) -> Self {
        ScrollView {
            content_offset,
            viewport,
        }
    }

    #[track_caller]
    pub fn build(self, cx: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let caller = crate::key::Key::current();
        cx.render_object(caller, self, content);
    }
}

impl Properties for ScrollView {
    type Object = ScrollViewObject;
}

pub struct ScrollViewObject {
    content_offset: Point,
    viewport: Size,
    content_size: Size,
}

impl ScrollViewObject {
    fn should_show_scrollbar(&self) -> bool {
        self.content_size.height > self.viewport.height
            || self.content_size.width > self.viewport.width
    }

    fn scrollbar_size(&self) -> Size {
        let content_height = self.content_size.height;
        let viewport_height = self.viewport.height;
        let scrollbar_height = viewport_height * viewport_height / content_height;
        Size::new(4., scrollbar_height)
    }

    fn scrollbar_offset(&self) -> Point {
        let scrollbar_offset_y =
            self.content_offset.y / self.content_size.height * self.viewport.height;
        let scrollbar_offset_x = self.viewport.width - self.scrollbar_size().width - 2.;
        Point::new(scrollbar_offset_x, scrollbar_offset_y)
    }

    fn update_content_offset(&mut self, wheel_delta: Vec2) {
        let x = (self.content_offset.x + wheel_delta.x)
            .min(self.content_size.width - self.viewport.width)
            .max(0.0);
        let y = (self.content_offset.y + wheel_delta.y)
            .min(self.content_size.height - self.viewport.height)
            .max(0.0);
        self.content_offset = Point::new(x, y);
    }
}

impl RenderObject<ScrollView> for ScrollViewObject {
    type Action = ();

    fn create(props: ScrollView) -> Self {
        ScrollViewObject {
            content_offset: props.content_offset,
            viewport: props.viewport,
            content_size: Size::ZERO,
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: ScrollView) -> Self::Action {
        if self.viewport != props.viewport {
            self.viewport = props.viewport;
            ctx.request_layout();
        }
    }
}

impl RenderObjectInterface for ScrollViewObject {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, children: &mut Children) {
        match event {
            Event::Wheel(mouse_event) => {
                self.update_content_offset(mouse_event.wheel_delta);
                ctx.request_layout();
                ctx.set_handled();
                return;
            }
            _ => {}
        }
        children[0].event(ctx, event)
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn dry_layout_box(
        &mut self,
        _ctx: &mut LayoutCtx,
        _c: &BoxConstraints,
        _children: &mut Children,
    ) -> Size {
        self.viewport
    }

    fn layout_box(
        &mut self,
        ctx: &mut LayoutCtx,
        _c: &BoxConstraints,
        children: &mut Children,
    ) -> Size {
        self.content_size = children[0].layout_box(ctx, &BoxConstraints::UNBOUNDED);
        self.update_content_offset(Vec2::ZERO);
        children[0].set_origin(
            ctx,
            Point::new(-self.content_offset.x, -self.content_offset.y),
        );
        self.viewport
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut Children) {
        ctx.clip(self.viewport.to_rect());
        children[0].paint(ctx);

        if self.should_show_scrollbar() {
            let brush = PaintBrush::Color(Color::rgb8(0, 0, 0));
            ctx.fill(
                Rect::from_origin_size(self.scrollbar_offset(), self.scrollbar_size())
                    .to_rounded_rect(4.),
                &brush,
            );
        }
    }
}
