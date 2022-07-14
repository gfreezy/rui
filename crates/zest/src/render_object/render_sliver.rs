use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::{
    abstract_node::AbstractNode,
    render_object::{Matrix4, Offset, PaintContext, RenderObject},
    render_object_state::RenderObjectState,
};

#[derive(Clone)]
pub struct RenderSliver {
    inner: Rc<RefCell<InnerRenderSliver>>,
}

impl PartialEq for RenderSliver {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl AbstractNode for RenderSliver {
    fn state<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R {
        process(&mut self.inner.borrow_mut().state)
    }
}

impl RenderSliver {
    pub fn downgrade(&self) -> WeakRenderSliver {
        WeakRenderSliver {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn apply_paint_transform(&self, _child: &RenderObject, _transform: &Matrix4) {
        todo!()
    }

    pub(crate) fn is_repaint_bondary(&self) -> bool {
        true
    }
    pub(crate) fn mark_needs_layout(&self) {
        self.state(|s| s.mark_needs_layout());
    }

    pub(crate) fn layout(
        &self,
        _constraints: super::render_object::Constraints,
        _parent_use_size: bool,
    ) {
        todo!()
    }

    pub(crate) fn paint(&self, _context: &mut PaintContext, offset: Offset) {
        todo!()
    }

    pub(crate) fn sized_by_parent(&self) -> bool {
        todo!()
    }

    pub(crate) fn handle_event(
        &self,
        event: super::render_object::PointerEvent,
        entry: super::render_object::HitTestEntry,
    ) {
        todo!()
    }
}

struct InnerRenderSliver {
    state: RenderObjectState,
}

#[derive(Clone)]
pub struct WeakRenderSliver {
    inner: Weak<RefCell<InnerRenderSliver>>,
}

impl WeakRenderSliver {
    pub fn upgrade(&self) -> RenderSliver {
        self.inner
            .upgrade()
            .map(|inner| RenderSliver { inner })
            .unwrap()
    }

    pub fn is_alive(&self) -> bool {
        self.inner.upgrade().is_some()
    }
}
