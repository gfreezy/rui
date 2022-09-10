use druid_shell::piet::Piet;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use crate::constraints::{BoxConstraints, Constraints};
use crate::geometry::{Matrix4, Offset, Rect, Size};
use crate::hit_test::{HitTestEntry, HitTestResult};
use crate::paint_context::PaintContext;
use crate::pointer_event::PointerEvent;
use crate::render_object::layer::Layer;
use crate::render_object::parent_data::ParentData;
use crate::render_object::pipeline_owner::{PipelineOwner, WeakOwner};
use crate::render_object::render_object::RenderObject;
use crate::render_object::render_object::WeakRenderObject;

#[derive(Clone)]
pub struct RenderView {
    pub(crate) inner: Rc<RefCell<InnerRenderView>>,
}

impl PartialEq for RenderView {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Debug for RenderView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderView").finish()
    }
}

#[mixin::insert(RenderObjectState)]
pub(crate) struct InnerRenderView {
    size: Size,
}

impl Default for InnerRenderView {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_string(),
            size: Size::ZERO,
            first_child: Default::default(),
            last_child: Default::default(),
            next_sibling: Default::default(),
            prev_sibling: Default::default(),
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

    pub fn is_alive(&self) -> bool {
        true
    }
}

impl RenderView {
    pub(crate) fn render_object(&self) -> RenderObject {
        RenderObject::RenderView(self.clone())
    }

    pub(crate) fn new_render_object(size: Size) -> RenderObject {
        let v = Self {
            inner: Rc::new(RefCell::new(InnerRenderView {
                size,
                ..Default::default()
            })),
        };

        let root_view = RenderObject::RenderView(v.clone());
        root_view.mark_needs_layout();
        root_view
    }

    pub fn downgrade(&self) -> WeakRenderView {
        WeakRenderView {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn composite_frame(&self, piet: &mut Piet) {
        let root_layer = self.layer();
        root_layer.draw_in(piet);
        for layer in root_layer.decendents() {
            layer.draw_in(piet);
        }
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

    pub(crate) fn is_repaint_bondary(&self) -> bool {
        true
    }

    pub(crate) fn handle_event(&self, _event: PointerEvent, _entry: HitTestEntry) {}

    pub(crate) fn layout_without_resize(&self) {
        self.perform_layout();
        self.set_needs_layout(false);
        self.mark_needs_paint();
    }

    pub(crate) fn paint_bounds(&self) -> Rect {
        Rect::from_size(self.inner.borrow().size)
    }

    pub(crate) fn layout(&self, constraints: Constraints, _parent_use_size: bool) {
        self.set_constraints(constraints);
        self.perform_layout();
        self.set_needs_layout(false);
        self.mark_needs_paint();
    }

    pub(crate) fn apply_paint_transform(&self, _child: &RenderObject, _transform: &Matrix4) {
        todo!()
    }

    pub(crate) fn hit_test(&self, result: &mut HitTestResult, position: Offset) -> bool {
        if let Some(child) = self.try_first_child() {
            child.hit_test(result, crate::hit_test::HitTestPosition::Box(position));
        }
        result.add(HitTestEntry::new_box_hit_test_entry(
            &self.render_object(),
            position,
        ));

        true
    }

    pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
        self.set_needs_paint(false);
        self.paint(context, offset);
        assert!(!self.needs_layout());
        assert!(!self.needs_paint());
    }
    pub(crate) fn get_dry_layout(&self, _constraints: Constraints) -> Size {
        todo!()
    }

    pub(crate) fn attach(&self, owner: PipelineOwner) {
        self._attach(owner)
    }

    pub(crate) fn detach(&self) {
        self._detach()
    }

    /// Mark the given node as being a child of this node.
    ///
    /// Subclasses should call this function when they acquire a new child.
    pub(crate) fn adopt_child(&self, child: &RenderObject) {
        self._adopt_child(child)
    }

    /// Disconnect the given node from this node.
    ///
    /// Subclasses should call this function when they lose a child.
    pub(crate) fn drop_child(&self, child: &RenderObject) {
        self._drop_child(child)
    }

    pub(crate) fn mark_needs_layout(&self) {
        self._mark_needs_layout()
    }

    pub(crate) fn mark_parent_needs_layout(&self) {
        self._mark_parent_needs_layout()
    }
    pub(crate) fn mark_needs_paint(&self) {
        self._mark_needs_paint()
    }
}
