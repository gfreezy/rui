use druid_shell::piet::Piet;

use super::{
    render_box::{BoxConstraints, Size},
    render_object::{Rect, RenderObject},
};

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::render_object::{
    AbstractNodeExt, HitTestEntry, Matrix4, Offset, PaintContext, PointerEvent, WeakRenderObject,
};
use crate::render_object::render_object::{try_ultimate_next_sibling, try_ultimate_prev_sibling};

use super::{
    layer::Layer,
    pipeline_owner::{PipelineOwner, WeakOwner},
    render_object::{AbstractNode, Constraints, ParentData},
};

#[derive(Clone)]
pub struct RenderView {
    pub(crate) inner: Rc<RefCell<InnerRenderView>>,
}

impl PartialEq for RenderView {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl RenderView {
    pub(crate) fn new_render_object(child: RenderObject, size: Size) -> RenderObject {
        let v = Self {
            inner: Rc::new(RefCell::new(InnerRenderView {
                size,
                ..Default::default()
            })),
        };

        let object = RenderObject::RenderView(v.clone());
        v.set_render_object(&object);
        object.set_first_child(Some(child));
        object.mark_needs_layout();
        object
    }

    pub fn downgrade(&self) -> WeakRenderView {
        WeakRenderView {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn composite_frame(&self, piet: &mut Piet) {
        let child = self.first_child();
        assert!(child.is_repaint_bondary());
        let layer = child.layer();
        layer.draw_at_point(piet, child.render_box().offset());
    }

    fn size(&self) -> Size {
        self.inner.borrow().size
    }

    pub(crate) fn perform_layout(&self) {
        let size = self.size();
        self.first_child()
            .layout(BoxConstraints::tight(size).into(), true);
    }

    pub(crate) fn paint(&self, context: &mut PaintContext, offset: Offset) {
        context.paint_child(&self.first_child(), offset);
    }
}

#[mixin::insert(RenderObjectState)]
pub(crate) struct InnerRenderView {
    size: Size,
}

impl Default for InnerRenderView {
    fn default() -> Self {
        Self {
            size: Size::ZERO,
            first_child: Default::default(),
            last_child: Default::default(),
            next_sibling: Default::default(),
            prev_sibling: Default::default(),
            self_render_object: Default::default(),
            child_count: Default::default(),
            depth: Default::default(),
            parent: Default::default(),
            owner: Default::default(),
            parent_data: Default::default(),
            needs_layout: Default::default(),
            needs_paint: Default::default(),
            relayout_boundary: Default::default(),
            doing_this_layout_with_callback: Default::default(),
            constraints: Default::default(),
            layer: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct WeakRenderView {
    inner: Weak<RefCell<InnerRenderView>>,
}

impl WeakRenderView {
    pub fn upgrade(&self) -> RenderView {
        self.inner
            .upgrade()
            .map(|inner| RenderView { inner })
            .unwrap()
    }
}

impl AbstractNodeExt for RenderView {
    fn is_repaint_bondary(&self) -> bool {
        true
    }

    fn handle_event(&self, event: PointerEvent, entry: HitTestEntry) {
        todo!()
    }

    fn layout_without_resize(&self) {
        self.perform_layout();
        self.clear_needs_layout();
        self.mark_needs_paint();
    }

    fn paint_bounds(&self) -> Rect {
        Rect::from_size(self.inner.borrow().size)
    }

    fn layout(&self, constraints: Constraints, parent_use_size: bool) {
        todo!()
    }

    fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4) {
        todo!()
    }
}
