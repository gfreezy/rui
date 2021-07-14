use context::{LayoutCtx, PaintCtx};
use druid_shell::{
    kurbo::{Point, Rect, Size},
    MouseEvent,
};

use crate::{
    box_constraints::BoxConstraints,
    context::{self, EventCtx},
};

use super::{label::Label, Event, Widget, WidgetState};

pub struct Button {
    label: Label,
    state: WidgetState,
    on_click: Box<dyn FnMut()>,
}

impl Button {
    pub fn new(label: Label, on_click: impl FnMut() + 'static) -> Self {
        Button {
            label,
            on_click: Box::new(on_click),
            state: Default::default(),
        }
    }
}

impl Widget for Button {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::MouseDown(MouseEvent { pos: point, .. }) => {
                let rect = Rect::from((self.state.origin, self.state.size));
                if rect.contains(*point) {
                    (*self.on_click)();
                }
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: BoxConstraints) -> Size {
        let size = self.label.layout(ctx, bc);
        self.state.size = size;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.label.paint(ctx);
    }

    fn set_origin(&mut self, point: Point) {
        self.state.origin = point;
        self.label.set_origin(point);
    }
}
