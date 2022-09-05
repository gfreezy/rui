use druid_shell::{
    kurbo::Circle,
    piet::{Color, Piet, PietTextLayout, RenderContext},
};

use crate::{
    geometry::{Offset, Rect, Size},
    render_object::{layer::Layer, render_object::RenderObject},
};

pub struct PaintContext {
    paint_bounds: Rect,
    layer: Layer,
}

impl PaintContext {
    pub(crate) fn new(layer: Layer, rect: Rect) -> Self {
        Self {
            layer,
            paint_bounds: rect,
        }
    }

    pub(crate) fn paint_child(&mut self, child: &RenderObject, offset: Offset) {
        if child.is_repaint_bondary() {
            self.composite_child(child, offset);
        } else {
            child.paint_with_context(self, offset)
        }
    }

    pub fn draw_text(&mut self, layout: &PietTextLayout, offset: Offset) {
        self.layer.with_piet(|p| p.draw_text(layout, offset))
    }

    pub fn fill(&mut self) {
        self.layer
            .with_piet(|p| p.fill(Circle::new((10., 10.), 10.), &Color::BLACK));
    }

    pub(crate) fn repaint_composited_child(child: &RenderObject, offset: Offset, piet: &mut Piet) {
        assert!(child.needs_paint());
        assert!(child.is_repaint_bondary());
        let child_bounds = child.paint_bounds();
        let child_layer = match child.try_layer() {
            Some(layer) if layer.size() == child_bounds.size() => {
                layer.clear_children();
                layer.clear();
                layer.set_offset(offset);
                layer
            }
            _ => {
                let bounds = &child_bounds;
                let size = Size {
                    width: bounds.width(),
                    height: bounds.height(),
                };

                let child_layer = Layer::new(piet, size, offset);
                child.set_layer(Some(child_layer.clone()));
                child_layer
            }
        };

        let mut paint_context = PaintContext::new(child_layer, child_bounds);
        child.paint_with_context(&mut paint_context, Offset::ZERO);
    }

    fn composite_child(&mut self, child: &RenderObject, offset: Offset) {
        assert!(child.is_repaint_bondary());
        if child.needs_paint() {
            self.layer.with_piet(|p| {
                Self::repaint_composited_child(child, offset, p);
            });
        }
        let child_layer = child.layer();
        child_layer.set_offset(offset);
        self.layer.add_child(child_layer);
    }
}
