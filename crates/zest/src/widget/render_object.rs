use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    ops::{Add, Mul, Sub},
    rc::{Rc, Weak},
};

use super::{
    abstract_node::AbstractNode,
    owner::Owner,
    render_box::{BoxConstraints, RenderBox, WeakRenderBox},
    render_object_state::RenderObjectState,
    render_sliver::{RenderSliver, WeakRenderSliver},
};

pub(crate) fn try_ultimate_prev_sibling(mut element: RenderObject) -> RenderObject {
    while let Some(prev) = element.try_prev_sibling() {
        element = prev;
    }
    element
}

pub(crate) fn try_ultimate_next_sibling(mut element: RenderObject) -> RenderObject {
    while let Some(next) = element.try_next_sibling() {
        element = next;
    }
    element
}

#[derive(Clone, PartialEq, Default)]
pub struct Constraints {}

impl Constraints {
    pub fn is_tight(&self) -> bool {
        todo!()
    }

    pub fn box_constraints(&self) -> BoxConstraints {
        todo!()
    }
}

pub struct Rect {}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub(crate) fn dot(&self, other: Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Mul<f64> for Vector3 {
    type Output = Vector3;

    fn mul(self, s: f64) -> Vector3 {
        Vector3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

impl Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Self::Output {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
pub struct Matrix4([[f64; 4]; 4]);

impl Matrix4 {
    pub fn identity() -> Matrix4 {
        Matrix4([
            [1., 0., 0., 0.], // to preserve formatting
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub(crate) fn translate(&self, dx: f64, dy: f64) {
        todo!()
    }

    pub(crate) fn invert(&self) -> f64 {
        todo!()
    }

    pub(crate) fn perspective_transform(&mut self, point: Vector3) -> Vector3 {
        todo!()
    }
}

pub struct PointerEvent {}

pub struct HitTestEntry {}

pub struct PaintContext {}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Offset {
    pub dx: f64,
    pub dy: f64,
}

impl Offset {
    pub const ZERO: Offset = Offset { dx: 0.0, dy: 0.0 };

    pub(crate) fn new(dx: f64, dy: f64) -> Offset {
        Offset { dx, dy }
    }
}

impl Sub<Offset> for Offset {
    type Output = Offset;

    fn sub(self, rhs: Offset) -> Self::Output {
        Offset {
            dx: self.dx - rhs.dx,
            dy: self.dy - rhs.dy,
        }
    }
}

impl Add<Offset> for Offset {
    type Output = Offset;

    fn add(self, rhs: Offset) -> Self::Output {
        Offset {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}

#[derive(Clone)]
pub struct ParentData {
    inner: Rc<RefCell<dyn Any + 'static>>,
}

#[derive(Clone)]
struct WeakParentData {
    inner: Weak<RefCell<dyn Any + 'static>>,
}

#[derive(Clone, PartialEq)]
pub enum RenderObject {
    RenderBox(RenderBox),
    RenderSliver(RenderSliver),
}

impl Debug for RenderObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element").finish()
    }
}

impl AbstractNode for RenderObject {
    fn state<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R {
        match self {
            RenderObject::RenderBox(boxed) => boxed.state(process),
            RenderObject::RenderSliver(boxed) => boxed.state(process),
        }
    }
}

impl RenderObject {
    pub fn downgrade(&self) -> WeakRenderObject {
        match self {
            RenderObject::RenderBox(boxed) => WeakRenderObject::RenderBox(boxed.downgrade()),
            RenderObject::RenderSliver(boxed) => WeakRenderObject::RenderSliver(boxed.downgrade()),
        }
    }

    pub(crate) fn render_box(&self) -> RenderBox {
        match self {
            RenderObject::RenderBox(boxed) => boxed.clone(),
            RenderObject::RenderSliver(_) => unreachable!(),
        }
    }

    pub(crate) fn render_sliver(&self) -> RenderSliver {
        match self {
            RenderObject::RenderBox(_) => unreachable!(),
            RenderObject::RenderSliver(boxed) => boxed.clone(),
        }
    }

    // fn parent_data(&self) -> ParentData {
    //     self.state(|s| s.parent_data())
    // }

    // fn try_parent_data(&self) -> Option<ParentData> {
    //     self.state(|s| s.try_parent_data())
    // }

    pub(crate) fn mark_needs_layout(&self) {
        match self {
            RenderObject::RenderBox(s) => s.mark_needs_layout(),
            RenderObject::RenderSliver(s) => s.mark_needs_layout(),
        }
    }

    pub fn needs_layout(&self) -> bool {
        match self {
            RenderObject::RenderBox(s) => s.needs_layout(),
            RenderObject::RenderSliver(s) => s.needs_layout(),
        }
    }

    pub fn needs_paint(&self) -> bool {
        match self {
            RenderObject::RenderBox(s) => s.needs_paint(),
            RenderObject::RenderSliver(s) => s.needs_paint(),
        }
    }

    pub fn relayout_boundary(&self) -> RenderObject {
        match self {
            RenderObject::RenderBox(s) => s.relayout_boundary(),
            RenderObject::RenderSliver(s) => s.relayout_boundary(),
        }
    }

    pub(crate) fn owner(&self) -> Owner {
        self.state(|s| s.owner())
    }

    pub(crate) fn try_owner(&self) -> Option<Owner> {
        self.state(|s| s.try_owner())
    }

    pub fn mark_needs_paint(&self) {
        match self {
            RenderObject::RenderBox(s) => s.mark_needs_paint(),
            RenderObject::RenderSliver(s) => s.mark_needs_paint(),
        }
    }

    pub(crate) fn clean_relayout_boundary(&self) {
        match self {
            RenderObject::RenderBox(s) => s.clean_relayout_boundary(),
            RenderObject::RenderSliver(s) => s.clean_relayout_boundary(),
        }
    }

    pub(crate) fn propagate_relayout_bondary(&self) {
        match self {
            RenderObject::RenderBox(s) => s.propagate_relayout_bondary(),
            RenderObject::RenderSliver(s) => s.propagate_relayout_bondary(),
        }
    }

    // pub(crate) fn schedule_initial_layout(&self) {
    //     self.set_relayout_boundary(Some(self.clone()));
    //     self.owner().add_node_need_layout(self.clone());
    // }

    // fn layout_without_resize(&self) {
    //     assert_eq!(&self.relayout_boundary(), self);
    //     assert!(!self.doing_this_layout_with_callback());
    //     self.perform_layout();
    //     self.set_needs_layout(false);
    //     self.mark_needs_paint();
    // }

    /// Compute the layout for this render object.
    ///
    /// This method is the main entry point for parents to ask their children to
    /// update their layout information. The parent passes a constraints object,
    /// which informs the child as to which layouts are permissible. The child is
    /// required to obey the given constraints.
    ///
    /// If the parent reads information computed during the child's layout, the
    /// parent must pass true for `parentUsesSize`. In that case, the parent will
    /// be marked as needing layout whenever the child is marked as needing layout
    /// because the parent's layout information depends on the child's layout
    /// information. If the parent uses the default value (false) for
    /// `parentUsesSize`, the child can change its layout information (subject to
    /// the given constraints) without informing the parent.
    ///
    /// Subclasses should not override [layout] directly. Instead, they should
    /// override [performResize] and/or [performLayout]. The [layout] method
    /// delegates the actual work to [performResize] and [performLayout].
    ///
    /// The parent's [performLayout] method should call the [layout] of all its
    /// children unconditionally. It is the [layout] method's responsibility (as
    /// implemented here) to return early if the child does not need to do any
    /// work to update its layout information.
    fn layout(&self, constraints: Constraints, parent_use_size: bool) {
        match self {
            RenderObject::RenderBox(s) => s.layout(constraints, parent_use_size),
            RenderObject::RenderSliver(s) => s.layout(constraints, parent_use_size),
        }
    }

    /// Whether the constraints are the only input to the sizing algorithm (in
    /// particular, child nodes have no impact).
    ///
    /// Returning false is always correct, but returning true can be more
    /// efficient when computing the size of this render object because we don't
    /// need to recompute the size if the constraints don't change.
    ///
    /// Typically, subclasses will always return the same value. If the value can
    /// change, then, when it does change, the subclass should make sure to call
    /// [markNeedsLayoutForSizedByParentChange].
    ///
    /// Subclasses that return true must not change the dimensions of this render
    /// object in [performLayout]. Instead, that work should be done by
    /// [performResize] or - for subclasses of [RenderBox] - in
    /// [RenderBox.computeDryLayout].
    fn sized_by_parent(&self) -> bool {
        match self {
            RenderObject::RenderBox(s) => s.sized_by_parent(),
            RenderObject::RenderSliver(s) => s.sized_by_parent(),
        }
    }

    /// Whether this render object repaints separately from its parent.
    ///
    /// Override this in subclasses to indicate that instances of your class ought
    /// to repaint independently. For example, render objects that repaint
    /// frequently might want to repaint themselves without requiring their parent
    /// to repaint.
    ///
    /// If this getter returns true, the [paintBounds] are applied to this object
    /// and all descendants. The framework automatically creates an [OffsetLayer]
    /// and assigns it to the [layer] field. Render objects that declare
    /// themselves as repaint boundaries must not replace the layer created by
    /// the framework.
    ///
    /// Warning: This getter must not change value over the lifetime of this object.
    ///
    /// See [RepaintBoundary] for more information about how repaint boundaries function.
    pub(crate) fn is_repaint_bondary(&self) -> bool {
        match self {
            RenderObject::RenderBox(s) => s.is_repaint_bondary(),
            RenderObject::RenderSliver(s) => s.is_repaint_bondary(),
        }
    }

    pub(crate) fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints)) {
        match self {
            RenderObject::RenderBox(s) => s.invoke_layout_callback(callback),
            RenderObject::RenderSliver(s) => s.invoke_layout_callback(callback),
        }
    }

    // /// Bootstrap the rendering pipeline by scheduling the very first paint.
    // ///
    // /// Requires that this render object is attached, is the root of the render
    // /// tree, and has a composited layer.
    // ///
    // /// See [RenderView] for an example of how this function is used.
    // fn schedule_initial_paint(&self) {
    //     self.owner().add_node_need_paint(self.clone());
    // }

    // /// Override this method to handle pointer events that hit this render object.
    // pub fn handle_event(&self, event: PointerEvent, entry: HitTestEntry) {}

    pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
        match self {
            RenderObject::RenderBox(s) => s.paint_with_context(context, offset),
            RenderObject::RenderSliver(s) => s.paint_with_context(context, offset),
        }
    }

    pub(crate) fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4) {
        match self {
            RenderObject::RenderBox(s) => s.apply_paint_transform(child, transform),
            RenderObject::RenderSliver(s) => s.apply_paint_transform(child, transform),
        }
    }
}

#[derive(Clone)]
pub enum WeakRenderObject {
    RenderBox(WeakRenderBox),
    RenderSliver(WeakRenderSliver),
}

impl WeakRenderObject {
    pub fn upgrade(&self) -> RenderObject {
        match self {
            WeakRenderObject::RenderBox(o) => RenderObject::RenderBox(o.upgrade()),
            WeakRenderObject::RenderSliver(o) => RenderObject::RenderSliver(o.upgrade()),
        }
    }

    pub fn is_alive(&self) -> bool {
        match self {
            WeakRenderObject::RenderBox(o) => o.is_alive(),
            WeakRenderObject::RenderSliver(o) => o.is_alive(),
        }
    }
}

impl PaintContext {
    pub(crate) fn paint_child(&mut self, child: &RenderObject, offset: Offset) {
        if child.is_repaint_bondary() {
            // composite_child(child, offset)
        } else {
            child.paint_with_context(self, offset)
        }
    }
}
