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
    size: Size,
    children: Vec<Layer>,
}

impl Layer {
    pub fn new(piet: &mut Piet, size: Size, offset: Offset) -> Self {
        let layer = piet.create_layer(druid_shell::kurbo::Size {
            width: size.width,
            height: size.height,
        });
        Layer {
            inner: Rc::new(RefCell::new(InnerLayer {
                layer,
                size,
                offset,
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

    pub fn children(&self) -> Vec<Layer> {
        self.inner.borrow().children.clone()
    }

    pub fn decendents(&self) -> Vec<Layer> {
        self.inner
            .borrow()
            .children
            .iter()
            .flat_map(|child| {
                let mut decendents = child.decendents();
                decendents.insert(0, child.clone());
                decendents
            })
            .collect()
    }

    pub fn size(&self) -> Size {
        self.inner.borrow().size
    }

    pub fn offset(&self) -> Offset {
        self.inner.borrow().offset
    }

    pub fn with_piet<T>(&self, f: impl FnOnce(&mut Piet) -> T) -> T {
        let mut borrow = self.inner.borrow_mut();
        let mut piet = Piet::new_from_layer(&mut borrow.layer);
        f(&mut piet)
    }

    pub fn clear(&self) {
        self.inner.borrow().layer.clear();
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

    pub fn draw_in(&self, piet: &mut Piet) {
        let ref_mut = self.inner.borrow_mut();
        piet.draw_layer_at_point(&ref_mut.layer, ref_mut.offset.into());
    }

    pub(crate) fn set_offset(&self, offset: Offset) {
        self.inner.borrow_mut().offset = offset;
    }
}
