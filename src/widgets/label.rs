use crate::box_constraints::BoxConstraints;
use crate::context::{EventCtx, LayoutCtx, PaintCtx};
use crate::widgets::{Event, Widget, WidgetState};
use druid_shell::kurbo::{Point, Rect, Size};
use druid_shell::piet::{
    Color, FontFamily, PietText, PietTextLayout, RenderContext, Text, TextLayout, TextLayoutBuilder,
};
use druid_shell::MouseEvent;

pub struct Label {
    text: String,
    color: Color,
    font_size: f64,
    wrap_width: f64,
    layout: Option<PietTextLayout>,
    state: WidgetState,
}

impl Label {
    pub fn new(text: impl Into<String>, color: Color) -> Self {
        Label {
            text: text.into(),
            color,
            layout: None,
            font_size: 14.,
            state: Default::default(),
            wrap_width: 0.,
        }
    }

    fn rebuild_if_needed(&mut self, factory: &mut PietText, bc: BoxConstraints) {
        let wrap_width = bc.max().width;
        let need_rebuild = self.layout.is_none() || (wrap_width - self.wrap_width).abs() > 0.1;
        if need_rebuild {
            self.wrap_width = wrap_width;
            self.layout = Some(
                factory
                    .new_text_layout(self.text.clone())
                    .text_color(self.color.clone())
                    .font(FontFamily::default(), self.font_size)
                    .max_width(self.wrap_width)
                    .build()
                    .unwrap(),
            );
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        let new_text = text.into();
        if new_text == self.text {
            return;
        }
        self.text = new_text;
        self.layout = None;
    }

    pub fn set_font_size(&mut self, size: f64) {
        if self.font_size != size {
            self.font_size = size;
            self.layout = None;
        }
    }

    pub fn set_color(&mut self, color: Color) {
        if color == self.color {
            return;
        }
        self.color = color;
        self.layout = None;
    }
}

impl Widget for Label {
    fn event(&mut self, _ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::MouseDown(MouseEvent { pos: point, .. }) => {
                let rect = Rect::from((self.state.origin, self.state.size));
                if rect.contains(*point) {
                    println!("clicked");
                }
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: BoxConstraints) -> Size {
        self.rebuild_if_needed(ctx.text(), bc);
        let size = self
            .layout
            .as_ref()
            .map_or(Size::default(), |layout| layout.size());
        self.state.size = size;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        if let Some(layout) = self.layout.as_ref() {
            ctx.with_save(|ctx| {
                let rect = Rect::from_origin_size(self.state.origin, self.state.size);
                ctx.render_ctx.stroke(rect, &Color::rgb8(15, 12, 30), 1.);
                ctx.render_ctx.draw_text(&layout, self.state.origin);
            });
        }
    }

    fn set_origin(&mut self, point: Point) {
        self.state.origin = point;
    }
}
