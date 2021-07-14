use crate::box_constraints::BoxConstraints;
use crate::context::{EventCtx, LayoutCtx, PaintCtx};
use crate::widgets;
use crate::widgets::Event;
use druid::widget::Label;
use druid::{Point, Size};

impl widgets::Widget for Label<String> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        todo!()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: BoxConstraints) -> Size {
        todo!()
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        todo!()
    }

    fn set_origin(&mut self, point: Point) {
        todo!()
    }
}
