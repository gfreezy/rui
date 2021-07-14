pub mod box_container;
pub mod button;
pub mod label;

use std::any::Any;

use crate::box_constraints::BoxConstraints;
use crate::context::{EventCtx, LayoutCtx, PaintCtx};
use druid_shell::kurbo::{Point, Size};
use druid_shell::MouseEvent;

#[derive(Debug)]
pub enum Event {
    MouseDown(MouseEvent),
}

pub trait Widget: Any {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event);

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: BoxConstraints) -> Size;

    fn paint(&mut self, ctx: &mut PaintCtx);

    fn set_origin(&mut self, point: Point);

    fn mutate(&mut self) {}
}

pub type AnyWidget = Box<dyn Widget + 'static>;

#[derive(Default, Debug)]
pub struct WidgetState {
    /// The size of the child; this is the value returned by the child's layout
    /// method.
    pub(crate) size: Size,
    pub(crate) origin: Point,
}

#[cfg(test)]
mod tests {
    use druid_shell::kurbo::Point;

    #[test]
    fn test_point_add() {
        let p = Point::new(0., 0.) + Point::new(20., 20.).to_vec2();
        assert_eq!(p, Point::new(20., 20.))
    }
}
