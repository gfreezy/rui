use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::render_object::{
    AbstractNodeExt, HitTestEntry, Matrix4, Offset, PaintContext, PointerEvent, RenderObject,
    WeakRenderObject,
};
use crate::render_object::render_object::{try_ultimate_next_sibling, try_ultimate_prev_sibling};

use super::{
    layer::Layer,
    pipeline_owner::{PipelineOwner, WeakOwner},
    render_object::{AbstractNode, Constraints, ParentData, Rect},
};

#[derive(Clone)]
pub struct RenderSliver {
    pub(crate) inner: Rc<RefCell<InnerRenderSliver>>,
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

    pub(crate) fn apply_paint_transform(&self, _child: &RenderObject, _transform: &Matrix4) {
        todo!()
    }

    pub(crate) fn is_repaint_bondary(&self) -> bool {
        true
    }
    pub(crate) fn mark_needs_layout(&self) {
        self.mark_needs_layout();
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

#[mixin::insert(RenderObjectState)]
pub(crate) struct InnerRenderSliver {}

#[derive(Clone)]
pub struct WeakRenderSliver {
    pub(crate) inner: Weak<RefCell<InnerRenderSliver>>,
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

impl AbstractNodeExt for RenderSliver {
    fn is_repaint_bondary(&self) -> bool {
        todo!()
    }

    fn handle_event(&self, _event: PointerEvent, _entry: HitTestEntry) {
        todo!()
    }

    fn layout_without_resize(&self) {
        todo!()
    }

    fn paint_bounds(&self) -> Rect {
        todo!()
    }

    fn layout(&self, constraints: Constraints, parent_use_size: bool) {
        todo!()
    }
}
