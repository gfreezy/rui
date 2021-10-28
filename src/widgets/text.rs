//! A label widget.

use std::panic::Location;

use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::{Color, RenderContext, TextAlignment};

use crate::box_constraints::BoxConstraints;
use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx};
use crate::event::Event;
use crate::lifecycle::LifeCycle;
use crate::object::{Properties, RenderObject, RenderObjectInterface};
use crate::text::font_descriptor::FontDescriptor;
use crate::text::layout::TextLayout;
use crate::tree::Children;
use crate::ui::Ui;

// added padding between the edges of the widget and the text.
const LABEL_X_PADDING: f64 = 2.0;

/// A widget that displays text data.
///
/// This requires the `Data` to be `ArcStr`; to handle static, dynamic, or
/// localized text, use [`Label`].
///
/// [`Label`]: struct.Label.html
#[derive(Debug, Clone)]
pub struct Text {
    text: String,
    line_break_mode: LineBreaking,
    text_color: Color,
    text_size: f64,
    font: FontDescriptor,
    alignment: TextAlignment,
}

impl Properties for Text {
    type Object = TextObject;
}

/// Options for handling lines that are too wide for the label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineBreaking {
    /// Lines are broken at word boundaries.
    WordWrap,
    /// Lines are truncated to the width of the label.
    Clip,
    /// Lines overflow the label.
    Overflow,
}

impl Text {
    /// Create a new `Label`.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            line_break_mode: LineBreaking::Overflow,
            text_color: Color::BLACK,
            text_size: 14.,
            font: FontDescriptor::default(),
            alignment: TextAlignment::Start,
        }
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn text_size(mut self, size: f64) -> Self {
        self.text_size = size;
        self
    }

    pub fn font(mut self, font: FontDescriptor) -> Self {
        self.font = font;
        self
    }

    pub fn line_break_mode(mut self, mode: LineBreaking) -> Self {
        self.line_break_mode = mode;
        self
    }

    pub fn text_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui) {
        let caller = Location::caller().into();
        ui.render_object(caller, self, |_| {});
    }
}

pub struct TextObject {
    text: Text,
    layout: TextLayout<String>,
}

impl TextObject {
    fn new(text: Text) -> Self {
        let layout = TextLayout::from_text(&text.text);

        let mut obj = TextObject { text, layout };
        obj.update_attrs();
        obj
    }

    fn set_text_color(&mut self, color: Color) {
        self.layout.set_text_color(color);
    }

    fn update_attrs(&mut self) {
        self.layout.set_text(self.text.text.clone());
        self.layout.set_font(self.text.font.clone());
        self.layout.set_text_size(self.text.text_size);
        self.layout.set_text_alignment(self.text.alignment);
    }

    /// Draw this label's text at the provided `Point`, without internal padding.
    ///
    /// This is a convenience for widgets that want to use Label as a way
    /// of managing a dynamic or localized string, but want finer control
    /// over where the text is drawn.
    fn draw_at(&self, ctx: &mut PaintCtx, origin: impl Into<Point>) {
        debug_assert!(
            self.layout.layout().is_some(),
            "TextLayout::draw called without rebuilding layout object. Text was '{}'",
            self.layout
                .text()
                .as_ref()
                .map(|t| t.as_str())
                .unwrap_or("layout is missing text")
        );
        if let Some(layout) = self.layout.layout() {
            ctx.draw_text(layout, origin);
        }
    }

    #[allow(dead_code)]
    // TODO: Find out what this was good for.
    /// Return the offset of the first baseline relative to the bottom of the widget.
    fn baseline_offset(&self) -> f64 {
        let text_metrics = self.layout.layout_metrics();
        text_metrics.size.height - text_metrics.first_baseline
    }
}

impl RenderObject<Text> for TextObject {
    type Action = ();

    fn create(props: Text) -> Self {
        TextObject::new(props)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, props: Text) {
        if self.text.text_color != props.text_color {
            self.text = props;
            self.set_text_color(self.text.text_color.clone());
            ctx.request_paint();
        } else {
            self.text = props;
            ctx.request_layout();
            self.update_attrs();
        }
        if self.layout.layout().is_none() {
            ctx.request_layout();
        }
    }
}

impl RenderObjectInterface for TextObject {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _children: &mut Children) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _children: &mut Children) {
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _children: &mut Children,
    ) -> Size {
        // tracing::debug!("layout for text {:?}", self.layout.text());

        bc.debug_check("Label");

        let width = match self.text.line_break_mode {
            LineBreaking::WordWrap => bc.max().width - LABEL_X_PADDING * 2.0,
            _ => f64::INFINITY,
        };

        self.layout.set_wrap_width(width);
        self.layout.rebuild_if_needed(ctx.text());

        let text_metrics = self.layout.layout_metrics();
        ctx.set_baseline_offset(text_metrics.size.height - text_metrics.first_baseline);
        bc.constrain(Size::new(
            text_metrics.size.width + 2. * LABEL_X_PADDING,
            text_metrics.size.height,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _children: &mut Children) {
        // tracing::debug!("paint for text {:?}", self.layout.text());

        let origin = Point::new(LABEL_X_PADDING, 0.0);
        let label_size = ctx.size();

        if self.text.line_break_mode == LineBreaking::Clip {
            ctx.clip(label_size.to_rect());
        }
        self.draw_at(ctx, origin)
    }
}
