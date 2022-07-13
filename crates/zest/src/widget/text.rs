use druid_shell::piet::{
    PietText, PietTextLayout, Text, TextAttribute, TextLayout, TextLayoutBuilder,
};

use crate::render_object::{
    abstract_node::AbstractNode,
    render_box::{RenderBoxWidget, Size},
};

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
        &self,
        _event: crate::render_object::render_object::PointerEvent,
        _entry: crate::render_object::render_box::BoxHitTestEntry,
    ) {
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
        this.set_size(size)
    }

    fn hit_test_self(
        self: &mut RenderText,
        _this: &crate::render_object::render_object::RenderObject,
        _position: crate::render_object::render_object::Offset,
    ) -> bool {
        false
    }

    fn hit_test_children(
        self: &mut RenderText,
        this: &crate::render_object::render_object::RenderObject,
        result: &mut crate::render_object::render_box::BoxHitTestResult,
        position: crate::render_object::render_object::Offset,
    ) -> bool {
        let mut child = this.try_last_child();
        while let Some(c) = child {
            let offset = c.render_box().offset();
            let is_hit = result.add_with_paint_offset(offset, position, |result, transformed| {
                assert_eq!(transformed, position - offset);
                c.render_box().hit_test(result, transformed)
            });
            if is_hit {
                return true;
            }
            child = c.try_prev_sibling();
        }
        false
    }

    fn is_repaint_boundary(&self) -> bool {
        true
    }
}
