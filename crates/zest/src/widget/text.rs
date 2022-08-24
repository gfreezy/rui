use druid_shell::piet::{
    PietText, PietTextLayout, Text, TextAttribute, TextLayout, TextLayoutBuilder,
};

use crate::render_object::{
    render_box::{RenderBoxProps, RenderBoxWidget, Size},
    render_object::RenderObject,
};

#[derive(Default)]
pub struct RenderTextProps {
    text: String,
    font_size: f64,
    max_width: Option<f64>,
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

    pub fn set_text(&mut self, new_text: String) {
        self.text = new_text;
        self.layout = None;
    }

    pub fn set_font_size(&mut self, font_size: f64) {
        self.font_size = font_size;
        self.layout = None;
    }

    pub fn rebuild_if_needed(&mut self, factory: &mut PietText) {
        if self.layout.is_none() {
            let builder = factory
                .new_text_layout(self.text.clone())
                .default_attribute(TextAttribute::FontSize(self.font_size));
            self.layout = Some(builder.build().unwrap());
        }
    }

    fn layout(&self) -> &PietTextLayout {
        self.layout.as_ref().unwrap()
    }
}

impl RenderBoxProps for RenderText {
    type Props = RenderTextProps;

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
        paint_context.draw_text(self.layout.as_ref().unwrap(), offset);
    }

    fn handle_event(
        &mut self,
        this: &crate::render_object::render_object::RenderObject,
        event: crate::render_object::render_object::PointerEvent,
        entry: crate::render_object::render_box::BoxHitTestEntry,
    ) {
        tracing::debug!("text handle event");
        self.font_size += 1.;
        self.layout = None;
        this.mark_needs_layout();
    }

    fn is_repaint_boundary(&self) -> bool {
        true
    }

    fn sized_by_parent(&self) -> bool {
        false
    }

    fn compute_min_instrinsic_width(
        &self,
        _this: &crate::render_object::render_object::RenderObject,
        _height: f64,
    ) -> f64 {
        0.
    }

    fn compute_max_instrinsic_width(
        &self,
        _this: &crate::render_object::render_object::RenderObject,
        _height: f64,
    ) -> f64 {
        0.0
    }

    fn compute_min_instrinsic_height(
        &self,
        _this: &crate::render_object::render_object::RenderObject,
        _width: f64,
    ) -> f64 {
        0.0
    }

    fn compute_max_instrinsic_height(
        &self,
        _this: &crate::render_object::render_object::RenderObject,
        _width: f64,
    ) -> f64 {
        0.0
    }

    fn compute_dry_layout(
        &mut self,
        this: &crate::render_object::render_object::RenderObject,
        _constraints: crate::render_object::render_box::BoxConstraints,
    ) -> crate::render_object::render_box::Size {
        self.rebuild_if_needed(&mut this.owner().text());
        self.layout().size().into()
    }

    fn perform_resize(&mut self, _this: &crate::render_object::render_object::RenderObject) {}

    fn perform_layout(&mut self, this: &crate::render_object::render_object::RenderObject) {
        self.rebuild_if_needed(&mut this.owner().text());
        let size: Size = self.layout().size().into();
        tracing::debug!("text perform layout: {:?}", size);
        this.render_box().set_size(size)
    }

    fn hit_test_self(
        self: &mut RenderText,
        _this: &crate::render_object::render_object::RenderObject,
        _position: crate::render_object::render_object::Offset,
    ) -> bool {
        true
    }

    fn hit_test_children(
        self: &mut RenderText,
        this: &crate::render_object::render_object::RenderObject,
        result: &mut crate::render_object::render_box::HitTestResult,
        position: crate::render_object::render_object::Offset,
    ) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "RenderText"
    }
}
