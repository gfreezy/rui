use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::{
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

impl RenderSliver {
    pub fn downgrade(&self) -> WeakRenderSliver {
        WeakRenderSliver {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn state<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R {
        process(&mut self.inner.borrow_mut().state)
    }

    pub(crate) fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4) {
        todo!()
    }

    pub(crate) fn mark_needs_paint(&self) {
        self.state(|s| s.mark_needs_paint())
    }

    pub(crate) fn clean_relayout_boundary(&self) {
        self.state(|s| s.clean_relayout_boundary())
    }

    pub(crate) fn propagate_relayout_bondary(&self) {
        self.state(|s| s.propagate_relayout_bondary())
    }

    pub(crate) fn relayout_boundary(&self) -> RenderObject {
        self.state(|s| s.relayout_boundary())
    }

    pub(crate) fn mark_needs_layout(&self) {
        self.state(|s| s.mark_needs_layout())
    }

    pub(crate) fn invoke_layout_callback(
        &self,
        callback: impl FnOnce(&super::render_object::Constraints),
    ) {
        self.state(|s| s.invoke_layout_callback(callback))
    }

    pub(crate) fn layout(
        &self,
        constraints: super::render_object::Constraints,
        parent_use_size: bool,
    ) {
        todo!()
    }

    pub(crate) fn sized_by_parent(&self) -> bool {
        todo!()
    }

    pub(crate) fn needs_layout(&self) -> bool {
        self.state(|s| s.needs_layout)
    }

    pub(crate) fn needs_paint(&self) -> bool {
        self.state(|s| s.needs_paint)
    }

    pub(crate) fn is_repaint_bondary(&self) -> bool {
        todo!()
    }
    pub(crate) fn paint(&self, context: &mut PaintContext, offset: Offset) {
        todo!()
    }
    pub(crate) fn paint_with_context(
        &self,
        context: &mut super::render_object::PaintContext,
        offset: super::render_object::Offset,
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
