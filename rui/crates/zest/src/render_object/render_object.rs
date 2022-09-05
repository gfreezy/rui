use std::{
    any::Any,
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use druid_shell::{
    kurbo::Circle,
    piet::{Color, Piet, PietTextLayout, RenderContext},
    MouseEvent,
};

use crate::{
    constraints::{BoxConstraints, Constraints},
    geometry::{Matrix4, Offset, Rect, Size, Vector3},
    hit_test::{HitTestEntry, HitTestResult},
    paint_context::PaintContext,
    pointer_event::PointerEvent,
};

use crate::render_object::{
    layer::Layer,
    pipeline_owner::PipelineOwner,
    render_box::{RenderBox, RenderBoxWidget, WeakRenderBox},
    render_sliver::{RenderSliver, WeakRenderSliver},
    render_view::{RenderView, WeakRenderView},
};

use super::parent_data::ParentData;

#[derive(Clone, PartialEq)]
pub enum RenderObject {
    RenderBox(RenderBox),
    RenderSliver(RenderSliver),
    RenderView(RenderView),
}

impl Debug for RenderObject {
    delegate::delegate! {
            to match self {
                RenderObject::RenderBox(box_) => box_,
                RenderObject::RenderSliver(sliver) => sliver,
                RenderObject::RenderView(view) => view,
            } {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
        }
    }
}

impl RenderObject {
    pub fn new_render_box(name: String, widget: impl RenderBoxWidget) -> RenderObject {
        RenderBox::new_render_object(name, Box::new(widget))
    }

    pub(crate) fn new_render_view(size: Size) -> RenderObject {
        RenderView::new_render_object(size)
    }

    pub fn downgrade(&self) -> WeakRenderObject {
        match self {
            RenderObject::RenderBox(boxed) => WeakRenderObject::RenderBox(boxed.downgrade()),
            RenderObject::RenderSliver(boxed) => WeakRenderObject::RenderSliver(boxed.downgrade()),
            RenderObject::RenderView(boxed) => WeakRenderObject::RenderView(boxed.downgrade()),
        }
    }

    pub(crate) fn render_box(&self) -> RenderBox {
        match self {
            RenderObject::RenderBox(boxed) => boxed.clone(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn render_sliver(&self) -> RenderSliver {
        match self {
            RenderObject::RenderBox(_) => unreachable!(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn render_view(&self) -> RenderView {
        match self {
            RenderObject::RenderBox(_boxed) => unreachable!(),
            RenderObject::RenderSliver(_) => unreachable!(),
            RenderObject::RenderView(boxed) => boxed.clone(),
        }
    }

    pub(crate) fn schedule_initial_layout(&self) {
        self.set_relayout_boundary(Some(self.clone()));
        self.owner().add_node_need_layout(self.clone());
    }

    /// Bootstrap the rendering pipeline by scheduling the very first paint.
    ///
    /// Requires that this render object is attached, is the root of the render
    /// tree, and has a composited layer.
    ///
    /// See [RenderView] for an example of how this function is used.
    fn schedule_initial_paint(&self) {
        self.owner().add_node_need_paint(self.clone());
    }

    pub(crate) fn prepare_initial_frame(&self) {
        self.schedule_initial_layout();
        self.schedule_initial_paint();
    }

    pub(crate) fn global_to_local(&self, point: Offset, ancestor: Option<RenderObject>) -> Offset {
        let mut transform = self.get_transform_to(ancestor);
        let det = transform.invert();
        if det == 0.0 {
            return Offset::ZERO;
        }

        let n = Vector3::new(0.0, 0.0, 1.0);
        let _i = transform.perspective_transform(Vector3::new(0.0, 0.0, 0.0));
        let d = transform.perspective_transform(Vector3::new(0.0, 0.0, 0.0));
        let s = transform.perspective_transform(Vector3::new(point.dx, point.dy, 0.0));
        let p = s - d * (n.dot(s) / n.dot(d));
        Offset::new(p.x, p.y)
    }

    pub(crate) fn get_transform_to(&self, ancestor: Option<RenderObject>) -> Matrix4 {
        let ancestor = match ancestor {
            Some(a) => a,
            None => self.owner().root_node(),
        };
        let mut renderers = vec![self.clone()];
        let mut renderer = self.clone();
        while renderer != ancestor {
            renderers.push(renderer.clone());
            if let Some(r) = renderer.try_parent() {
                renderer = r.parent();
            } else {
                break;
            }
        }
        renderers.push(ancestor);

        let mut transform = Matrix4::identity();
        let mut iter = renderers.iter().rev().peekable();
        while let (Some(renderer), Some(next)) = (iter.next(), iter.peek()) {
            renderer.apply_paint_transform(next, &mut transform);
        }
        transform
    }

    pub(crate) fn redepth_child(&self, child: &RenderObject) {
        if child.depth() <= self.depth() {
            child.incr_depth();
            child.redepth_children();
        }
    }

    pub(crate) fn hit_test(&self, result: &mut HitTestResult, position: Offset) -> bool {
        match self {
            RenderObject::RenderBox(o) => o.hit_test(result, position),
            RenderObject::RenderSliver(o) => o.hit_test(result, position),
            RenderObject::RenderView(o) => o.hit_test(result, position),
        }
    }

    pub fn update<T: 'static>(&self, update: impl FnOnce(&mut T)) {
        match self {
            RenderObject::RenderBox(o) => o.update(update),
            _ => unreachable!(),
        }
    }

    pub(crate) fn size(&self) -> Size {
        match self {
            RenderObject::RenderBox(o) => o.size(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn box_constraints(&self) -> BoxConstraints {
        self.constraints().box_constraints()
    }

    pub fn set_attribute(&self, key: &str, value: &str) {
        match self {
            RenderObject::RenderBox(o) => o.set_attribute(key, value),
            _ => unreachable!(),
        }
    }

    pub(crate) fn set_name(&self, name: String) {
        match self {
            RenderObject::RenderBox(o) => o.set_name(name),
            _ => unreachable!(),
        }
    }

    delegate::delegate! {
        to match self {
            RenderObject::RenderBox(box_) => box_,
            RenderObject::RenderSliver(sliver) => sliver,
            RenderObject::RenderView(view) => view,
        } {
            pub fn parent(&self) -> RenderObject;

            pub fn parent_data(&self) -> ParentData;

            pub fn try_parent_data(&self) -> Option<ParentData>;

            pub fn with_parent_data<T: 'static, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R>;

            pub fn try_parent(&self) -> Option<RenderObject>;

            pub fn first_child(&self) -> RenderObject;

            pub fn try_first_child(&self) -> Option<RenderObject>;

            pub fn last_child(&self) -> RenderObject;

            pub fn try_last_child(&self) -> Option<RenderObject>;

            pub fn next_sibling(&self) -> RenderObject;

            pub fn prev_sibling(&self) -> RenderObject;

            pub fn set_parent(&self, element: Option<RenderObject>);

            pub fn try_next_sibling(&self) -> Option<RenderObject>;

            pub fn try_prev_sibling(&self) -> Option<RenderObject>;

            pub fn set_next_sibling(&self, element: Option<RenderObject>);

            pub fn set_prev_sibling(&self, element: Option<RenderObject>);

            pub fn set_first_child(&self, element: Option<RenderObject>);

            pub fn set_last_child(&self, element: Option<RenderObject>);

            pub fn set_last_child_if_none(&self, element: Option<RenderObject>);

            pub(crate) fn attach(&self, owner: PipelineOwner);

            pub fn detach(&self);

            /// Mark the given node as being a child of this node.
            ///
            /// Subclasses should call this function when they acquire a new child.
            pub fn adopt_child(&self, child: &RenderObject);

            /// Disconnect the given node from this node.
            ///
            /// Subclasses should call this function when they lose a child.
            pub fn drop_child(&self, child: &RenderObject);

            /// Insert child into this render object's child list after the given child.
            ///
            /// If `after` is null, then this inserts the child at the start of the list,
            /// and the child becomes the new [firstChild].
            pub fn insert(&self, child: RenderObject, after: Option<RenderObject>);

            pub fn add(&self, child: RenderObject);

            pub fn remove(&self, child: &RenderObject);

            pub fn remove_all(&self);

            pub fn move_(&self, child: RenderObject, after: Option<RenderObject>);

            pub fn depth(&self) -> usize;

            pub fn child_count(&self) -> usize;

            pub fn redepth_children(&self);

            pub fn relayout_boundary(&self) -> RenderObject;

            pub fn try_relayout_boundary(&self) -> Option<RenderObject>;

            pub fn owner(&self) -> PipelineOwner;

            pub fn try_owner(&self) -> Option<PipelineOwner>;

            pub fn needs_layout(&self) -> bool;

            pub fn needs_paint(&self) -> bool;

            pub fn try_constraints(&self) -> Option<Constraints>;

            pub fn constraints(&self) -> Constraints;

            pub fn doing_this_layout_with_callback(&self) -> bool;

            pub(crate) fn try_layer(&self) -> Option<Layer>;

            pub(crate) fn layer(&self) -> Layer;

            pub fn set_relayout_boundary(&self, relayout_boundary: Option<RenderObject>);

            pub fn clean_relayout_boundary(&self);

            pub fn propagate_relayout_bondary(&self);

            pub fn mark_needs_layout(&self);

            pub fn clear_needs_layout(&self);

            pub fn mark_parent_needs_layout(&self);

            pub(crate) fn set_owner(&self, owner: Option<PipelineOwner>);

            pub fn clear_needs_paint(&self);

            pub fn mark_needs_paint(&self);

            pub fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints));

            pub(crate) fn set_layer(&self, layer: Option<Layer>);

            pub fn incr_depth(&self);

            pub fn clear_child_count(&self);

            pub fn incr_child_count(&self);

            pub fn decr_child_count(&self);

            pub fn set_constraints(&self, c: Constraints);

            pub fn paint_with_context(&self, context: &mut PaintContext, offset: Offset);
            pub fn visit_children(&self, visitor: impl FnMut(RenderObject));

            pub fn is_repaint_bondary(&self) -> bool;
            pub fn handle_event(&self, event: PointerEvent, entry: HitTestEntry);
            pub fn layout_without_resize(&self);
            pub fn layout(&self, constraints: Constraints, parent_use_size: bool);
            pub fn paint_bounds(&self) -> Rect;

            pub fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4);

            pub fn to_string_short(&self) -> String;
            pub fn to_string_deep(&self) -> String;
            pub fn id(&self) -> usize;
            pub fn set_id(&self, id: usize);
        }
    }
}

#[derive(Clone)]
pub enum WeakRenderObject {
    RenderBox(WeakRenderBox),
    RenderSliver(WeakRenderSliver),
    RenderView(WeakRenderView),
}

impl WeakRenderObject {
    pub fn upgrade(&self) -> RenderObject {
        match self {
            WeakRenderObject::RenderBox(o) => RenderObject::RenderBox(o.upgrade()),
            WeakRenderObject::RenderSliver(o) => RenderObject::RenderSliver(o.upgrade()),
            WeakRenderObject::RenderView(o) => RenderObject::RenderView(o.upgrade()),
        }
    }

    delegate::delegate!(to match self {
        WeakRenderObject::RenderBox(o) => o,
        WeakRenderObject::RenderSliver(o) => o,
        WeakRenderObject::RenderView(o) => o,
    } {
        pub fn is_alive(&self) -> bool;
    });
}
