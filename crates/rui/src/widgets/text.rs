//! A label widget.

use std::panic::Location;

use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::RenderContext;

use crate::box_constraints::BoxConstraints;
use crate::constraints::Constraints;
use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx, UpdateCtx};
use crate::event::Event;
use crate::lifecycle::LifeCycle;
use crate::object::{Properties, RenderObject, RenderObjectInterface};
use crate::style::text::LineBreaking;
use crate::style::Style;
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
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    text: String,
    style: Style,
}

impl Properties for Text {
    type Object = TextObject;
}

impl Text {
    /// Create a new `Label`.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui) {
        let caller = Location::caller().into();
        ui.render_object(caller, self, |_| {});
    }
}

pub struct TextObject {
    style: Style,
    layout: TextLayout<String>,
}

impl TextObject {
    fn new(text: Text) -> Self {
        let layout = TextLayout::from_text(&text.text);

        let mut obj = TextObject {
            style: text.style,
            layout,
        };
        obj.update_style();
        obj
    }

    fn update_style(&mut self) {
        let font_descriptor = FontDescriptor::new(self.style.font_family.clone())
            .with_size(self.style.font_size.into())
            .with_style(self.style.font_style)
            .with_weight(self.style.font_weight);
        self.layout.set_font(font_descriptor);
        self.layout.set_text_alignment(self.style.text_alignment);
        self.layout.set_text_color(self.style.color.clone());
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
        if self.layout.text() != Some(&props.text) {
            self.layout.set_text(props.text);
            ctx.request_layout();
        }

        if self.style != props.style {
            self.style = props.style;
            self.update_style();
            ctx.request_layout();
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

    fn dry_layout(
        &mut self,
        ctx: &mut LayoutCtx,
        c: &Constraints,
        _children: &mut Children,
    ) -> Size {
        // tracing::debug!("layout for text {:?}", self.layout.text());
        let bc: BoxConstraints = c.into();
        bc.debug_check("Label");

        let width = match self.style.line_breaking {
            LineBreaking::WordWrap => bc.max().width - LABEL_X_PADDING * 2.0,
            _ => f64::INFINITY,
        };

        self.layout.set_wrap_width(width);
        self.layout.rebuild_if_needed(&mut ctx.text());

        let text_metrics = self.layout.layout_metrics();
        ctx.set_baseline_offset(text_metrics.size.height - text_metrics.first_baseline);
        bc.constrain(Size::new(
            text_metrics.size.width + 2. * LABEL_X_PADDING,
            text_metrics.size.height,
        ))
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints, _children: &mut Children) -> Size {
        // tracing::debug!("layout for text {:?}", self.layout.text());
        let bc: BoxConstraints = c.into();
        bc.debug_check("Label");

        let width = match self.style.line_breaking {
            LineBreaking::WordWrap => bc.max().width - LABEL_X_PADDING * 2.0,
            _ => f64::INFINITY,
        };

        self.layout.set_wrap_width(width);
        self.layout.rebuild_if_needed(&mut ctx.text());

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

        if self.style.line_breaking == LineBreaking::Clip {
            ctx.clip(label_size.to_rect());
        }
        self.draw_at(ctx, origin)
    }
}
