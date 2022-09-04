use druid_shell::piet::Piet;
use std::fmt::Debug;

use super::{
    render_box::{BoxConstraints, HitTestResult, Size},
    render_object::{Rect, RenderObject},
};

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::render_object::{
    HitTestEntry, Matrix4, Offset, PaintContext, PointerEvent, WeakRenderObject,
};

use super::{
    layer::Layer,
    pipeline_owner::{PipelineOwner, WeakOwner},
    render_object::{Constraints, ParentData},
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

impl Debug for RenderView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderView").finish()
    }
}

impl RenderView {
    pub(crate) fn new_render_object(size: Size) -> RenderObject {
        let v = Self {
            inner: Rc::new(RefCell::new(InnerRenderView {
                size,
                ..Default::default()
            })),
        };

        let root_view = RenderObject::RenderView(v.clone());
        v.set_render_object(&root_view);
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

    pub fn is_alive(&self) -> bool {
        true
    }
}

impl RenderView {
    pub(crate) fn is_repaint_bondary(&self) -> bool {
        true
    }

    pub(crate) fn handle_event(&self, _event: PointerEvent, _entry: HitTestEntry) {}

    pub(crate) fn layout_without_resize(&self) {
        self.perform_layout();
        self.clear_needs_layout();
        self.mark_needs_paint();
    }

    pub(crate) fn paint_bounds(&self) -> Rect {
        Rect::from_size(self.inner.borrow().size)
    }

    pub(crate) fn layout(&self, constraints: Constraints, _parent_use_size: bool) {
        self.set_constraints(constraints);
        self.perform_layout();
        self.clear_needs_layout();
        self.mark_needs_paint();
    }

    pub(crate) fn apply_paint_transform(&self, _child: &RenderObject, _transform: &Matrix4) {
        todo!()
    }

    pub(crate) fn hit_test(&self, result: &mut HitTestResult, position: Offset) -> bool {
        if let Some(child) = self.try_first_child() {
            child.hit_test(result, position);
        }
        result.add(HitTestEntry::new_box_hit_test_entry(
            &self.render_object(),
            position,
        ));
        true
    }
}

impl RenderView {
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
            pub(crate) fn visit_children(&self, visitor: impl FnMut(RenderObject));
            pub(crate) fn depth(&self) -> usize;

            pub(crate) fn redepth_children(&self);

            pub(crate) fn relayout_boundary(&self) -> RenderObject;

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

            pub(crate) fn mark_needs_layout(&self);

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
