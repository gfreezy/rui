use std::{
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use super::{
    render_box::{HitTestResult, Size},
    render_object::{
        HitTestEntry, Matrix4, Offset, PaintContext, PointerEvent, RenderObject, WeakRenderObject,
    },
};

use super::{
    layer::Layer,
    pipeline_owner::{PipelineOwner, WeakOwner},
    render_object::{Constraints, ParentData, Rect},
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
        self.mark_needs_layout();
    }

    pub(crate) fn paint(&self, _context: &mut PaintContext, _offset: Offset) {
        todo!()
    }

    pub(crate) fn sized_by_parent(&self) -> bool {
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

impl RenderSliver {
    pub fn name(&self) -> String {
        "RenderSliver".to_string()
    }

    pub(crate) fn handle_event(&self, _event: PointerEvent, _entry: HitTestEntry) {
        todo!()
    }

    pub(crate) fn layout_without_resize(&self) {
        todo!()
    }

    pub(crate) fn paint_bounds(&self) -> Rect {
        todo!()
    }

    pub(crate) fn layout(&self, _constraints: Constraints, _parent_use_size: bool) {
        todo!()
    }

    pub(crate) fn apply_paint_transform(&self, _child: &RenderObject, _transform: &Matrix4) {
        todo!()
    }

    pub(crate) fn hit_test(&self, _result: &mut HitTestResult, _position: Offset) -> bool {
        todo!()
    }
}

impl_method! {
    RenderSliver {
        pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
            self.clear_needs_paint();
            self.paint(context, offset);
            assert!(!self.needs_layout());
            assert!(!self.needs_paint());
        }

        pub(crate) fn get_dry_layout(&self, _constraints: Constraints) -> Size {
            todo!()
        }

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
}
