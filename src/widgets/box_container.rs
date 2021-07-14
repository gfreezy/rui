use crate::box_constraints::BoxConstraints;
use crate::context::{EventCtx, LayoutCtx, PaintCtx};
use crate::widgets::{Event, Widget, WidgetState};
use druid_shell::kurbo::{Affine, Insets, Point, Rect, Size};
use druid_shell::piet::{Color, RenderContext};

pub struct BoxContainer {
    pub padding: Insets,
    state: WidgetState,
    child: Box<dyn Widget>,
}

impl BoxContainer {
    pub fn new(widget: Box<dyn Widget>) -> Self {
        BoxContainer {
            padding: Insets::ZERO,
            child: widget,
            state: Default::default(),
        }
    }

    pub fn set_padding(&mut self, insets: Insets) {
        self.padding = insets;
    }
}

impl Widget for BoxContainer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        self.child.event(ctx, event)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: BoxConstraints) -> Size {
        let hpad = self.padding.x_value();
        let vpad = self.padding.y_value();
        let child_bc = bc.shrink((hpad, vpad));
        let size = self.child.layout(ctx, child_bc);
        self.child
            .set_origin(Point::new(self.padding.x0, self.padding.y0));
        self.state.size = size + self.padding.size();
        self.state.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_save(|ctx| {
            let rect = Rect::from_origin_size(self.state.origin, self.state.size);
            ctx.render_ctx.stroke(rect, &Color::rgb8(15, 12, 30), 1.);
            ctx.render_ctx
                .transform(Affine::translate(self.state.origin.to_vec2()));
            self.child.paint(ctx)
        });
    }

    fn set_origin(&mut self, point: Point) {
        self.state.origin = point;
    }
}
