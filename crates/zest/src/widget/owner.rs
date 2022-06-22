use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::render_object::{RenderObject, WeakRenderObject};

#[derive(Clone)]
pub(crate) struct Owner {
    inner: Rc<RefCell<InnerOwner>>,
}

#[derive(Clone)]
pub(crate) struct WeakOwner {
    inner: Weak<RefCell<InnerOwner>>,
}

impl WeakOwner {
    pub fn upgrade(&self) -> Owner {
        self.inner.upgrade().map(|inner| Owner { inner }).unwrap()
    }
}

struct InnerOwner {
    nodes_need_layout: Vec<WeakRenderObject>,
    nodes_need_paint: Vec<WeakRenderObject>,
    need_visual_update: bool,
}

impl Owner {
    pub fn add_node_need_layout(&self, node: RenderObject) {
        self.inner
            .borrow_mut()
            .nodes_need_layout
            .push(node.downgrade());
    }

    pub fn add_node_need_paint(&self, node: RenderObject) {
        self.inner
            .borrow_mut()
            .nodes_need_paint
            .push(node.downgrade());
    }

    pub fn downgrade(&self) -> WeakOwner {
        WeakOwner {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn request_visual_update(&self) {
        todo!()
    }

    pub(crate) fn enable_mutations_to_dirty_subtrees(&self, callback: impl FnOnce()) {
        todo!()
    }

    pub(crate) fn root_node(&self) -> RenderObject {
        todo!()
    }
}
