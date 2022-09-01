use druid_shell::piet::{
    PietText, PietTextLayout, Text as _, TextAttribute, TextLayout, TextLayoutBuilder,
};

use crate::render_object::{
    render_box::{RenderBoxWidget, Size},
    render_object::RenderObject,
};

pub struct RenderText {
    text: String,
    font_size: f64,
    max_width: Option<f64>,
    layout: Option<PietTextLayout>,
}

impl RenderText {
    pub fn new(text: String, font_size: f64, max_width: Option<f64>) -> Self {
        RenderText {
            text,
            font_size,
            max_width,
            layout: None,
        }
    }

    fn rebuild_if_needed(&mut self, factory: &mut PietText) {
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

    pub fn set_font_size(&mut self, ctx: &RenderObject, font_size: f64) {
        self.font_size = font_size;
        self.layout = None;
        ctx.mark_needs_layout();
    }

    pub fn font_size(&self) -> f64 {
        self.font_size
    }
}

impl RenderBoxWidget for RenderText {
    fn paint(
        self: &mut RenderText,
        _ctx: &crate::render_object::render_object::RenderObject,
        paint_context: &mut crate::render_object::render_object::PaintContext,
        offset: crate::render_object::render_object::Offset,
    ) {
        tracing::debug!("paint text: {}, offset: {:?}", self.text, offset);
        paint_context.draw_text(self.layout.as_ref().unwrap(), offset);
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

    fn perform_layout(&mut self, ctx: &crate::render_object::render_object::RenderObject) {
        self.rebuild_if_needed(&mut ctx.owner().text());
        let size: Size = ctx.box_constraints().constrain(self.layout().size().into());

        tracing::debug!("text perform layout: {:?}", size);
        ctx.render_box().set_size(size)
    }

    fn hit_test_children(
        self: &mut RenderText,
        _ctx: &crate::render_object::render_object::RenderObject,
        _result: &mut crate::render_object::render_box::HitTestResult,
        _position: crate::render_object::render_object::Offset,
    ) -> bool {
        false
    }

    fn name(&self) -> String {
        format!("Text: {}", self.text)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
