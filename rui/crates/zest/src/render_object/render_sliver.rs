use crate::constraints::{Constraints, SliverConstraints, SliverGeometry};
use crate::geometry::{Matrix4, Offset, Rect, Size};
use crate::hit_test::{HitTestEntry, HitTestResult};
use crate::paint_context::PaintContext;
use crate::pointer_event::PointerEvent;
use crate::render_object::render_object::{RenderObject, WeakRenderObject};
use std::{
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use super::parent_data::ParentData;
use super::{
    layer::Layer,
    pipeline_owner::{PipelineOwner, WeakOwner},
};

#[derive(Clone)]
pub struct RenderSliver {
    pub(crate) inner: Rc<RefCell<InnerRenderSliver>>,
}

impl Debug for RenderSliver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderSliver")
            .field("name", &self.name())
            .finish()
    }
}

impl PartialEq for RenderSliver {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

#[mixin::insert(RenderObjectState)]
pub(crate) struct InnerRenderSliver {
    geometry: SliverGeometry,
}

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

impl RenderSliver {
    pub(crate) fn render_object(&self) -> RenderObject {
        RenderObject::RenderSliver(self.clone())
    }

    pub fn downgrade(&self) -> WeakRenderSliver {
        WeakRenderSliver {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn is_repaint_bondary(&self) -> bool {
        false
    }

    pub(crate) fn paint(&self, _context: &mut PaintContext, _offset: Offset) {
        todo!()
    }

    pub(crate) fn sized_by_parent(&self) -> bool {
        false
    }

    pub fn name(&self) -> String {
        "RenderSliver".to_string()
    }

    pub(crate) fn handle_event(&self, _event: PointerEvent, _entry: HitTestEntry) {
        todo!()
    }

    pub(crate) fn layout_without_resize(&self) {
        todo!()
    }

    pub fn sliver_constraints(&self) -> SliverConstraints {
        self.constraints().sliver_constraints()
    }

    pub fn geometry(&self) -> SliverGeometry {
        self.inner.borrow().geometry.clone()
    }

    pub(crate) fn paint_bounds(&self) -> Rect {
        let geometry = self.geometry();
        let constraints = self.sliver_constraints();
        match constraints.axis() {
            style::axis::Axis::Horizontal => {
                Rect::from_ltwh(0., 0., geometry.paint_extent, constraints.cross_axis_extent)
            }
            style::axis::Axis::Vertical => {
                Rect::from_ltwh(0., 0., constraints.cross_axis_extent, geometry.paint_extent)
            }
        }
    }

    pub(crate) fn layout(&self, constraints: Constraints, parent_use_size: bool) {
        let sc = constraints.sliver_constraints();
        let is_relayout_boundary = !parent_use_size
            || self.sized_by_parent()
            || sc.is_tight()
            || self.try_parent().is_none();
        let relayout_boundary = if is_relayout_boundary {
            Some(self.render_object())
        } else {
            self.parent().try_relayout_boundary()
        };
        if !self.needs_layout() && Some(&constraints) == self.try_constraints().as_ref() {
            if relayout_boundary != self.try_relayout_boundary() {
                self.set_relayout_boundary(relayout_boundary);
                self.visit_children(|child| {
                    child.propagate_relayout_bondary();
                });
            }

            return;
        }

        self.set_constraints(constraints);
        if self.try_relayout_boundary().is_some()
            && self.try_relayout_boundary() != relayout_boundary
        {
            self.visit_children(|child| {
                child.clean_relayout_boundary();
            });
        }

        self.perform_layout();
        self.set_needs_layout(false);
        self.mark_needs_paint();
        // todo: continue
    }

    pub(crate) fn apply_paint_transform(&self, _child: &RenderObject, _transform: &Matrix4) {
        todo!()
    }

    pub(crate) fn hit_test(
        &self,
        result: &mut HitTestResult,
        main_axis_position: f64,
        cross_axis_position: f64,
    ) -> bool {
        let geometry = self.geometry();
        let sc = self.sliver_constraints();
        if main_axis_position >= 0.0
            && main_axis_position < geometry.hit_test_extent
            && cross_axis_position >= 0.0
            && cross_axis_position < sc.cross_axis_extent
        {
            if self.hit_test_children(result, main_axis_position, cross_axis_position)
                || self.hit_test_self(main_axis_position, cross_axis_position)
            {
                result.add(HitTestEntry::new_sliver_hit_test_entry(
                    &self.render_object(),
                    main_axis_position,
                    cross_axis_position,
                ));
                return true;
            }
        }
        false
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

    pub(crate) fn center_offset_adjustment(&self) -> f64 {
        0.0
    }

    fn hit_test_children(
        &self,
        result: &mut HitTestResult,
        main_axis_position: f64,
        cross_axis_position: f64,
    ) -> bool {
        false
    }

    fn hit_test_self(&self, main_axis_position: f64, cross_axis_position: f64) -> bool {
        false
    }

    fn perform_layout(&self) {
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
