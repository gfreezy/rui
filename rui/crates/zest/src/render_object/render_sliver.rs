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
    pub fn downgrade(&self) -> WeakRenderSliver {
        WeakRenderSliver {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn is_repaint_bondary(&self) -> bool {
        true
    }

    pub(crate) fn mark_needs_layout(&self) {
        self.inner.borrow_mut().mark_needs_layout();
    }

    pub(crate) fn paint(&self, _context: &mut PaintContext, _offset: Offset) {
        todo!()
    }

    pub(crate) fn sized_by_parent(&self) -> bool {
        todo!()
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
                Rect::fromLTWH(0., 0., geometry.paint_extent, constraints.cross_axis_extent)
            }
            style::axis::Axis::Vertical => {
                Rect::fromLTWH(0., 0., constraints.cross_axis_extent, geometry.paint_extent)
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
        if !self.needs_layout() && Some(constraints) == self.try_constraints() {
            if relayout_boundary != self.try_relayout_boundary() {
                self.set_relayout_boundary(relayout_boundary);
                self.visit_children(|child| {
                    child.propagate_relayout_bondary();
                });
            }

            return;
        }

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
        self.clear_needs_paint();
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
}

impl RenderSliver {
    delegate::delegate! {
        // region: delegate to immutable inner
        to self.inner.borrow() {
            pub(crate) fn id(&self) -> usize;
            pub(crate) fn parent(&self) -> RenderObject;

            pub(crate) fn try_parent(&self) -> Option<RenderObject>;

            pub(crate) fn parent_data(&self) -> ParentData;

            pub(crate) fn try_parent_data(&self) -> Option<ParentData>;
            pub(crate) fn with_parent_data<T: 'static, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R>;
            pub(crate) fn first_child(&self) -> RenderObject;

            pub(crate) fn try_first_child(&self) -> Option<RenderObject>;

            pub(crate) fn last_child(&self) -> RenderObject;

            pub(crate) fn try_last_child(&self) -> Option<RenderObject>;

            pub(crate) fn next_sibling(&self) -> RenderObject;

            pub(crate) fn prev_sibling(&self) -> RenderObject;

            pub(crate) fn try_next_sibling(&self) -> Option<RenderObject>;

            pub(crate) fn try_prev_sibling(&self) -> Option<RenderObject>;

            pub(crate) fn child_count(&self) -> usize;

            pub(crate) fn depth(&self) -> usize;

            pub(crate) fn redepth_children(&self);

            pub(crate) fn relayout_boundary(&self) -> RenderObject;

            pub(crate) fn visit_children(&self, visitor: impl FnMut(RenderObject));

            pub(crate) fn try_relayout_boundary(&self) -> Option<RenderObject> ;

            pub(crate) fn owner(&self) -> PipelineOwner ;

            pub(crate) fn try_owner(&self) -> Option<PipelineOwner> ;

            pub(crate) fn needs_layout(&self) -> bool ;

            pub(crate) fn needs_paint(&self) -> bool ;

            pub(crate) fn try_constraints(&self) -> Option<Constraints> ;

            pub(crate) fn constraints(&self) -> Constraints ;

            pub(crate) fn doing_this_layout_with_callback(&self) -> bool ;

            pub(crate) fn try_layer(&self) -> Option<Layer> ;

            pub(crate) fn layer(&self) -> Layer ;
            pub(crate)fn render_object(&self) -> RenderObject;

            pub(crate) fn to_string_short(&self) -> String;
            pub(crate) fn to_string_deep(&self) -> String;
        }
        // endregion: delete to immutable inner

        // region: delegate to mutable inner
        to self.inner.borrow_mut() {
            pub(crate) fn set_id(&self, id: usize);
            pub(crate) fn set_parent(&self, element: Option<RenderObject>);

            pub(crate) fn set_next_sibling(&self, element: Option<RenderObject>);

            pub(crate) fn set_prev_sibling(&self, element: Option<RenderObject>);

            pub(crate) fn set_first_child(&self, element: Option<RenderObject>);

            pub(crate) fn set_last_child(&self, element: Option<RenderObject>);

            pub(crate) fn set_last_child_if_none(&self, element: Option<RenderObject>);

            pub(crate) fn attach(&self, owner: PipelineOwner);

            pub(crate) fn detach(&self);

            /// Mark the given node as being a child of this node.
            ///
            /// Subclasses should call this function when they acquire a new child.
            pub(crate) fn adopt_child(&self, child: &RenderObject);

            /// Disconnect the given node from this node.
            ///
            /// Subclasses should call this function when they lose a child.
            pub(crate) fn drop_child(&self, child: &RenderObject);

            /// Insert child into this render object's child list after the given child.
            ///
            /// If `after` is null, then this inserts the child at the start of the list,
            /// and the child becomes the new [firstChild].
            pub(crate) fn insert(&self, child: RenderObject, after: Option<RenderObject>);

            pub(crate) fn add(&self, child: RenderObject);

            pub(crate) fn remove(&self, child: &RenderObject);

            pub(crate) fn remove_all(&self);

            pub(crate) fn move_(&self, child: RenderObject, after: Option<RenderObject>);

            pub(crate) fn set_relayout_boundary(&self, relayout_boundary: Option<RenderObject>) ;

            pub(crate) fn clean_relayout_boundary(&self) ;

            pub(crate) fn propagate_relayout_bondary(&self) ;


            pub(crate) fn clear_needs_layout(&self) ;

            pub(crate) fn mark_parent_needs_layout(&self) ;

            pub(crate) fn set_owner(&self, owner: Option<PipelineOwner>) ;

            pub(crate) fn clear_needs_paint(&self) ;

            pub(crate) fn mark_needs_paint(&self) ;

            pub(crate) fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints)) ;

            pub(crate) fn set_layer(&self, layer: Option<Layer>) ;

            pub(crate) fn incr_depth(&self) ;

            pub(crate) fn clear_child_count(&self) ;

            pub(crate) fn incr_child_count(&self) ;

            pub(crate) fn decr_child_count(&self) ;

            pub(crate) fn set_constraints(&self, c: Constraints);

            pub(crate) fn set_render_object(&self, render_object: &RenderObject);

        }
        // endregion: delegate to mutable inner

    }
}
