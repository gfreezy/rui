use druid_shell::piet::{
    PietText, PietTextLayout, Text as _, TextAttribute, TextLayout, TextLayoutBuilder,
};

use crate::render_object::{
    render_box::{RenderBoxProps, RenderBoxWidget, Size},
    render_object::RenderObject,
};

#[derive(Default)]
pub struct Text {
    text: String,
    font_size: f64,
    max_width: Option<f64>,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Text {
            text: text.into(),
            font_size: 16.,
            max_width: None,
        }
    }

    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn max_width(mut self, max_width: f64) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn build(self) -> RenderObject {
        RenderObject::new_render_box(Box::new(RenderText::create(self)))
    }
}

pub struct RenderText {
    text: String,
    font_size: f64,
    max_width: Option<f64>,
    layout: Option<PietTextLayout>,
}

impl RenderText {
    pub fn new(text: String) -> Self {
        RenderText {
            text,
            font_size: 16.0,
            max_width: None,
            layout: None,
        }
    }

    pub fn rebuild_if_needed(&mut self, factory: &mut PietText) {
        if self.layout.is_none() {
            let builder = factory
                .new_text_layout(self.text.clone())
                .default_attribute(TextAttribute::FontSize(self.font_size))
                .max_width(self.max_width.unwrap_or_default());

            self.layout = Some(builder.build().unwrap());
        }
    }

    fn layout(&self) -> &PietTextLayout {
        self.layout.as_ref().unwrap()
    }
}

impl RenderBoxProps for RenderText {
    type Props = Text;

    fn create(props: Self::Props) -> Self {
        RenderText {
            text: props.text,
            font_size: props.font_size,
            max_width: props.max_width,
            layout: None,
        }
    }

    fn update(&mut self, this: &RenderObject, props: Self::Props) {
        *self = RenderText {
            text: props.text,
            font_size: props.font_size,
            max_width: props.max_width,
            layout: None,
        };
        this.mark_needs_layout();
    }
}

impl RenderBoxWidget for RenderText {
    fn paint(
        self: &mut RenderText,
        _this: &crate::render_object::render_object::RenderObject,
        paint_context: &mut crate::render_object::render_object::PaintContext,
        offset: crate::render_object::render_object::Offset,
    ) {
        tracing::debug!("paint text: {}, offset: {:?}", self.text, offset);
        paint_context.draw_text(self.layout.as_ref().unwrap(), offset);
    }

    fn handle_event(
        &mut self,
        this: &crate::render_object::render_object::RenderObject,
        _event: crate::render_object::render_object::PointerEvent,
        _entry: crate::render_object::render_box::BoxHitTestEntry,
    ) {
        tracing::debug!("text handle event");
        self.font_size += 2.;
        self.layout = None;
        this.mark_needs_layout();
    }

    fn is_repaint_boundary(&self) -> bool {
        true
    }

    fn sized_by_parent(&self) -> bool {
        false
    }

    fn compute_dry_layout(
        &mut self,
        this: &crate::render_object::render_object::RenderObject,
        constraints: crate::render_object::render_box::BoxConstraints,
    ) -> crate::render_object::render_box::Size {
        self.rebuild_if_needed(&mut this.owner().text());
        constraints.constrain(self.layout().size().into())
    }

    fn perform_layout(&mut self, this: &crate::render_object::render_object::RenderObject) {
        self.rebuild_if_needed(&mut this.owner().text());
        let size: Size = this
            .constraints()
            .box_constraints()
            .constrain(self.layout().size().into());

        tracing::debug!("text perform layout: {:?}", size);
        this.render_box().set_size(size)
    }

    fn hit_test_children(
        self: &mut RenderText,
        _this: &crate::render_object::render_object::RenderObject,
        _result: &mut crate::render_object::render_box::HitTestResult,
        _position: crate::render_object::render_object::Offset,
    ) -> bool {
        false
    }

    fn name(&self) -> String {
        format!("Text: {}", self.text)
    }
}
