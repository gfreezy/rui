use std::{cell::RefCell, rc::Rc};

use druid_shell::piet::{Piet, PietLayer};

use super::{
    render_box::Size,
    render_object::{Offset, Rect},
};

#[derive(Clone)]
pub(crate) struct Layer {
    inner: Rc<RefCell<InnerLayer>>,
}

struct InnerLayer {
    layer: PietLayer,
    offset: Offset,
    children: Vec<Layer>,
}

impl Layer {
    pub fn new(piet: &mut Piet, size: Size) -> Self {
        let layer = piet.create_layer(druid_shell::kurbo::Size {
            width: size.width,
            height: size.height,
        });
        Layer {
            inner: Rc::new(RefCell::new(InnerLayer {
                layer,
                offset: Offset::ZERO,
                children: vec![],
            })),
        }
    }
    pub fn add_child(&self, layer: Layer) {
        self.inner.borrow_mut().children.push(layer);
    }

    pub fn clear_children(&self) {
        self.inner.borrow_mut().children.clear();
    }

    pub fn with_piet<T>(&self, f: impl FnOnce(&mut Piet) -> T) -> T {
        let mut borrow = self.inner.borrow_mut();
        let mut piet = Piet::new_from_layer(&mut borrow.layer);
        f(&mut piet)
    }

    pub fn draw_in_rect(&self, piet: &mut Piet, rect: Rect) {
        let ref_mut = self.inner.borrow_mut();
        piet.draw_layer_in_rect(
            &ref_mut.layer,
            druid_shell::kurbo::Rect {
                x0: rect.x0,
                y0: rect.y0,
                x1: rect.x1,
                y1: rect.y1,
            },
        );
    }

    pub fn draw_at_point(&self, piet: &mut Piet, point: Offset) {
        let ref_mut = self.inner.borrow_mut();
        piet.draw_layer_at_point(
            &ref_mut.layer,
            druid_shell::kurbo::Point {
                x: point.dx,
                y: point.dy,
            },
        );
    }
}
