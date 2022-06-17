use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    ops::{Add, Mul, Sub},
    rc::{Rc, Weak},
};

use super::{
    render_box::RenderBox, render_object_state::RenderObjectState, render_sliver::RenderSliver,
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

#[derive(Clone)]
pub(crate) struct RenderObject {
    inner: Rc<InnerRenderObject>,
}

enum InnerRenderObject {
    RenderBox(RefCell<RenderBox>),
    RenderSliver(RefCell<RenderSliver>),
}

impl InnerRenderObject {
    fn new_box(box_: RenderBox) -> Self {
        InnerRenderObject::RenderBox(RefCell::new(box_))
    }

    fn new_sliver(sliver: RenderSliver) -> Self {
        InnerRenderObject::RenderSliver(RefCell::new(sliver))
    }
}

#[derive(Clone)]
pub(crate) struct Owner {
    inner: Rc<RefCell<InnerOwner>>,
}

#[derive(Clone)]
pub(crate) struct WeakOwner {
    inner: Weak<RefCell<InnerOwner>>,
}

impl WeakOwner {
    pub fn upgrade(&self) -> Owner {
        self.inner.upgrade().map(|inner| Owner { inner }).unwrap()
    }
}

struct InnerOwner {
    nodes_need_layout: Vec<WeakRenderObject>,
    nodes_need_paint: Vec<WeakRenderObject>,
    need_visual_update: bool,
}

impl Owner {
    pub fn add_node_need_layout(&self, node: RenderObject) {
        self.inner
            .borrow_mut()
            .nodes_need_layout
            .push(node.downgrade());
    }

    pub fn add_node_need_paint(&self, node: RenderObject) {
        self.inner
            .borrow_mut()
            .nodes_need_paint
            .push(node.downgrade());
    }

    pub fn downgrade(&self) -> WeakOwner {
        WeakOwner {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn request_visual_update(&self) {
        todo!()
    }

    pub(crate) fn enable_mutations_to_dirty_subtrees(&self, callback: impl FnOnce()) {
        todo!()
    }

    pub(crate) fn root_node(&self) -> RenderObject {
        todo!()
    }
}

impl PartialEq<RenderObject> for RenderObject {
    fn eq(&self, other: &RenderObject) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Debug for RenderObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element").finish()
    }
}

impl RenderObject {
    pub fn new_root(render_box: RenderBox) -> Self {
        RenderObject {
            inner: Rc::new(InnerRenderObject::new_box(render_box)),
        }
    }

    pub fn downgrade(&self) -> WeakRenderObject {
        WeakRenderObject {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn box_ref(&self) -> Ref<RenderBox> {
        match &*self.inner {
            InnerRenderObject::RenderBox(boxed) => boxed.borrow(),
            _ => panic!("RenderObject is not a RenderBox"),
        }
    }

    pub(crate) fn box_ref_mut(&self) -> RefMut<RenderBox> {
        match &*self.inner {
            InnerRenderObject::RenderBox(boxed) => boxed.borrow_mut(),
            _ => panic!("RenderObject is not a RenderBox"),
        }
    }

    pub(crate) fn sliver_ref(&self) -> Ref<RenderSliver> {
        match &*self.inner {
            InnerRenderObject::RenderSliver(boxed) => boxed.borrow(),
            _ => panic!("RenderObject is not a RenderSliver"),
        }
    }

    pub(crate) fn sliver_ref_mut(&self) -> RefMut<RenderSliver> {
        match &*self.inner {
            InnerRenderObject::RenderSliver(boxed) => boxed.borrow_mut(),
            _ => panic!("RenderObject is not a RenderSliver"),
        }
    }

    fn state<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R {
        match &*self.inner {
            InnerRenderObject::RenderBox(boxed) => process(&mut boxed.borrow_mut().state),
            InnerRenderObject::RenderSliver(boxed) => process(&mut boxed.borrow_mut().state),
        }
    }

    pub fn parent(&self) -> RenderObject {
        self.state(|s| s.parent())
    }

    pub fn try_parent(&self) -> Option<RenderObject> {
        self.state(|s| s.try_parent())
    }

    pub fn parent_data(&self) -> ParentData {
        self.state(|s| s.parent_data())
    }

    pub fn try_parent_data(&self) -> Option<ParentData> {
        self.state(|s| s.try_parent_data())
    }

    pub fn first_child(&self) -> RenderObject {
        self.state(|s| s.first_child())
    }

    pub fn try_first_child(&self) -> Option<RenderObject> {
        self.state(|s| s.try_first_child())
    }

    pub fn last_child(&self) -> RenderObject {
        self.state(|s| s.last_child())
    }

    pub fn try_last_child(&self) -> Option<RenderObject> {
        self.state(|s| s.try_last_child())
    }

    pub fn next_sibling(&self) -> RenderObject {
        self.state(|s| s.next_sibling())
    }

    pub fn prev_sibling(&self) -> RenderObject {
        self.state(|s| s.prev_sibling())
    }

    pub fn set_parent(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_parent(element))
    }

    pub fn try_next_sibling(&self) -> Option<RenderObject> {
        self.state(|s| s.try_next_sibling())
    }

    pub fn try_prev_sibling(&self) -> Option<RenderObject> {
        self.state(|s| s.try_prev_sibling())
    }

    pub fn set_next_sibling(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_next_sibling(element))
    }

    pub fn set_prev_sibling(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_prev_sibling(element))
    }

    pub fn set_first_child(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_first_child(element))
    }

    pub fn set_last_child(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_last_child(element))
    }

    pub fn set_last_child_if_none(&self, element: Option<RenderObject>) {
        self.state(|s| {
            if s.last_child.is_none() {
                s.last_child = element.map(|v| v.downgrade());
            }
        })
    }

    pub fn attach(&self, owner: Owner) {
        self.state(|s| s.attach(self, owner))
    }

    pub fn detach(&self) {
        self.state(|s| s.detach(self))
    }

    /// Mark the given node as being a child of this node.
    ///
    /// Subclasses should call this function when they acquire a new child.
    pub fn adopt_child(&self, child: &RenderObject) {
        self.state(|s| s.adopt_child(self, child))
    }

    /// Disconnect the given node from this node.
    ///
    /// Subclasses should call this function when they lose a child.
    pub fn drop_child(&self, child: &RenderObject) {
        self.state(|s| s.drop_child(self, child))
    }

    /// Adjust the [depth] of the given [child] to be greater than this node's own
    /// [depth].
    ///
    /// Only call this method from overrides of [redepthChildren].

    pub fn redepth_child(&self, child: &RenderObject) {
        self.state(|s| s.redepth_child(self))
    }

    /// Insert child into this render object's child list after the given child.
    ///
    /// If `after` is null, then this inserts the child at the start of the list,
    /// and the child becomes the new [firstChild].
    pub fn insert(&self, child: RenderObject, after: Option<RenderObject>) {
        self.state(|s| s.insert(self, child, after))
    }

    pub fn add(&self, child: RenderObject) {
        self.state(|s| s.add(self, child))
    }

    pub fn remove(&self, child: &RenderObject) {
        self.state(|s| s.remove(self, child))
    }

    pub fn remove_all(&self) {
        self.state(|s| s.remove_all(self))
    }

    pub fn move_(&self, child: RenderObject, after: Option<RenderObject>) {
        self.state(|s| s.move_(self, child, after))
    }

    pub(crate) fn depth(&self) -> usize {
        self.state(|s| s.depth)
    }

    pub(crate) fn incr_depth(&self) {
        self.state(|s| {
            s.depth += 1;
        })
    }

    pub(crate) fn child_count(&self) -> usize {
        self.state(|s| s.child_count)
    }

    pub(crate) fn clear_child_count(&self) {
        self.state(|s| s.child_count = 0)
    }

    pub(crate) fn incr_child_count(&self) {
        self.state(|s| {
            s.child_count += 1;
        })
    }

    pub(crate) fn decr_child_count(&self) {
        self.state(|s| {
            s.child_count -= 1;
        })
    }

    pub(crate) fn mark_needs_layout(&self) {
        self.state(|s| s.mark_needs_layout(self))
    }

    pub(crate) fn redepth_children(&self) {
        self.state(|s| s.redepth_children())
    }

    pub fn needs_layout(&self) -> bool {
        self.state(|s| s.needs_layout)
    }

    pub fn set_needs_layout(&self, needs_layout: bool) {
        self.state(|s| s.needs_layout = needs_layout)
    }

    pub fn needs_paint(&self) -> bool {
        self.state(|s| s.needs_paint)
    }

    pub fn set_needs_paint(&self, needs_paint: bool) {
        self.state(|s| s.needs_paint = needs_paint)
    }

    pub fn relayout_boundary(&self) -> RenderObject {
        self.state(|s| s.relayout_boundary())
    }

    pub fn try_relayout_boundary(&self) -> Option<RenderObject> {
        self.state(|s| s.try_relayout_boundary())
    }

    pub fn set_relayout_boundary(&self, relayout_boundary: Option<RenderObject>) {
        self.state(|s| s.set_relayout_boundary(relayout_boundary))
    }

    pub(crate) fn mark_parent_needs_layout(&self) {
        self.state(|s| s.mark_parent_needs_layout())
    }

    pub(crate) fn owner(&self) -> Owner {
        self.state(|s| s.owner())
    }

    pub(crate) fn try_owner(&self) -> Option<Owner> {
        self.state(|s| s.try_owner())
    }

    pub fn mark_needs_paint(&self) {
        self.state(|s| s.mark_needs_paint(self))
    }

    pub(crate) fn clean_relayout_boundary(&self) {
        self.state(|s| s.clean_relayout_boundary(self))
    }

    pub(crate) fn propagate_relayout_bondary(&self) {
        self.state(|s| s.propagate_relayout_bondary(self))
    }

    fn doing_this_layout_with_callback(&self) -> bool {
        self.state(|s| s.doing_this_layout_with_callback)
    }

    pub(crate) fn schedule_initial_layout(&self) {
        self.set_relayout_boundary(Some(self.clone()));
        self.owner().add_node_need_layout(self.clone());
    }

    fn layout_without_resize(&self) {
        assert_eq!(&self.relayout_boundary(), self);
        assert!(!self.doing_this_layout_with_callback());
        self.perform_layout();
        self.set_needs_layout(false);
        self.mark_needs_paint();
    }

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
    fn layout(&self, constraints: Constraints, parent_use_size: bool) {}

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
        false
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
        false
    }

    fn set_constraints(&self, constraints: Constraints) {
        self.state(|s| s.constraints = constraints)
    }

    /// {@template flutter.rendering.RenderObject.performResize}
    /// Updates the render objects size using only the constraints.
    ///
    /// Do not call this function directly: call [layout] instead. This function
    /// is called by [layout] when there is actually work to be done by this
    /// render object during layout. The layout constraints provided by your
    /// parent are available via the [constraints] getter.
    ///
    /// This function is called only if [sizedByParent] is true.
    /// {@endtemplate}
    ///
    /// Subclasses that set [sizedByParent] to true should override this method to
    /// compute their size. Subclasses of [RenderBox] should consider overriding
    /// [RenderBox.computeDryLayout] instead.
    pub(crate) fn perform_resize(&self) {
        todo!()
    }

    /// Do the work of computing the layout for this render object.
    ///
    /// Do not call this function directly: call [layout] instead. This function
    /// is called by [layout] when there is actually work to be done by this
    /// render object during layout. The layout constraints provided by your
    /// parent are available via the [constraints] getter.
    ///
    /// If [sizedByParent] is true, then this function should not actually change
    /// the dimensions of this render object. Instead, that work should be done by
    /// [performResize]. If [sizedByParent] is false, then this function should
    /// both change the dimensions of this render object and instruct its children
    /// to layout.
    ///
    /// In implementing this function, you must call [layout] on each of your
    /// children, passing true for parentUsesSize if your layout information is
    /// dependent on your child's layout information. Passing true for
    /// parentUsesSize ensures that this render object will undergo layout if the
    /// child undergoes layout. Otherwise, the child can change its layout
    /// information without informing this render object.
    pub(crate) fn perform_layout(&self) {
        todo!()
    }

    pub(crate) fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints)) {
        self.state(|s| s.invoke_layout_callback(callback))
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

    /// Override this method to handle pointer events that hit this render object.
    pub fn handle_event(&self, event: PointerEvent, entry: HitTestEntry) {}

    fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
        self.set_needs_paint(false);
        self.paint(context, offset);
        assert!(!self.needs_layout());
        assert!(!self.needs_paint());
    }

    /// Paint this render object into the given context at the given offset.
    ///
    /// Subclasses should override this method to provide a visual appearance
    /// for themselves. The render object's local coordinate system is
    /// axis-aligned with the coordinate system of the context's canvas and the
    /// render object's local origin (i.e, x=0 and y=0) is placed at the given
    /// offset in the context's canvas.
    ///
    /// Do not call this function directly. If you wish to paint yourself, call
    /// [markNeedsPaint] instead to schedule a call to this function. If you wish
    /// to paint one of your children, call [PaintingContext.paintChild] on the
    /// given `context`.
    ///
    /// When painting one of your children (via a paint child function on the
    /// given context), the current canvas held by the context might change
    /// because draw operations before and after painting children might need to
    /// be recorded on separate compositing layers.
    fn paint(&self, context: &mut PaintContext, offset: Offset) {}

    /// Applies the transform that would be applied when painting the given child
    /// to the given matrix.
    ///
    /// Used by coordinate conversion functions to translate coordinates local to
    /// one render object into coordinates local to another render object.
    pub fn apply_paint_transform(&self, child: RenderObject, transform: &mut Matrix4) {
        assert_eq!(&child.parent(), self);
    }
}

#[derive(Clone)]
pub(crate) struct WeakRenderObject {
    inner: Weak<InnerRenderObject>,
}

impl WeakRenderObject {
    pub fn upgrade(&self) -> RenderObject {
        self.inner
            .upgrade()
            .map(|inner| RenderObject { inner })
            .unwrap()
    }

    pub fn is_alive(&self) -> bool {
        self.inner.upgrade().is_some()
    }
}

impl PaintContext {
    pub(crate) fn paint_child(&self, c: &RenderObject, offset: Offset) {
        todo!()
    }
}
