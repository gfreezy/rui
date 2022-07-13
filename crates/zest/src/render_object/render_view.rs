use druid_shell::piet::Piet;

use super::{
    abstract_node::AbstractNode,
    render_box::{BoxConstraints, Size},
    render_object::{Rect, RenderObject},
};

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::{
    render_object::{Offset, PaintContext},
    render_object_state::RenderObjectState,
};

#[derive(Clone)]
pub struct RenderView {
    inner: Rc<RefCell<InnerRenderView>>,
}

impl PartialEq for RenderView {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl AbstractNode for RenderView {
    fn node<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R {
        process(&mut self.inner.borrow_mut().state)
    }
}

impl RenderView {
    pub(crate) fn new_render_object(child: RenderObject, size: Size) -> RenderObject {
        let v = Self {
            inner: Rc::new(RefCell::new(InnerRenderView {
                size,
                state: Default::default(),
            })),
        };
        let object = RenderObject::RenderView(v);
        object.set_this(object.clone());
        object.set_first_child(Some(child));
        object.mark_needs_layout();
        object
    }

    pub fn downgrade(&self) -> WeakRenderView {
        WeakRenderView {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn state<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R {
        process(&mut self.inner.borrow_mut().state)
    }

    pub(crate) fn set_relayout_boundary(&self, boundary: RenderObject) {
        self.state(|s| s.set_relayout_boundary(Some(boundary)));
    }

    pub(crate) fn needs_layout(&self) -> bool {
        self.state(|s| s.needs_layout)
    }

    pub(crate) fn mark_needs_layout(&self) {
        self.state(|s| s.mark_needs_layout())
    }

    pub(crate) fn mark_needs_paint(&self) {
        self.state(|s| s.mark_needs_paint())
    }

    pub(crate) fn needs_paint(&self) -> bool {
        self.state(|s| s.needs_paint)
    }

    pub(crate) fn composite_frame(&self, piet: &mut Piet) {
        let child = self.first_child();
        assert!(child.is_repaint_bondary());
        let layer = child.layer().unwrap();
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

    pub(crate) fn layout_without_resize(&self) {
        self.perform_layout();
        self.state(|s| s.needs_layout = false);
        self.mark_needs_paint();
    }

    pub(crate) fn layer(&self) -> Option<super::layer::Layer> {
        self.state(|s| s.layer.clone())
    }

    pub(crate) fn paint_bounds(&self) -> Rect {
        Rect::from_size(self.inner.borrow().size)
    }

    pub(crate) fn set_layer(&self, child_layer: super::layer::Layer) {
        self.state(|s| s.layer = Some(child_layer));
    }

    pub(crate) fn paint(&self, context: &mut PaintContext, offset: Offset) {
        context.paint_child(&self.first_child(), offset);
    }

    pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
        self.state(|s| s.needs_paint = false);
        self.paint(context, offset);
        assert!(!self.needs_layout());
        assert!(!self.needs_paint());
    }
}

struct InnerRenderView {
    state: RenderObjectState,
    size: Size,
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

    pub fn is_alive(&self) -> bool {
        self.inner.upgrade().is_some()
    }
}
