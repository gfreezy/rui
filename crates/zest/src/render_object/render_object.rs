use std::{
    any::Any,
    cell::RefCell,
    fmt::Debug,
    ops::{Add, Mul, Sub},
    rc::{Rc, Weak},
};

use druid_shell::{
    kurbo::{Circle, Point},
    piet::{Color, Piet, PietTextLayout, RenderContext},
};

use super::{
    abstract_node::AbstractNode,
    layer::Layer,
    pipeline_owner::PipelineOwner,
    render_box::{BoxConstraints, RenderBox, RenderBoxWidget, Size, WeakRenderBox},
    render_object_state::RenderObjectState,
    render_sliver::{RenderSliver, WeakRenderSliver},
    render_view::{RenderView, WeakRenderView},
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

#[derive(Clone, PartialEq)]
pub enum Constraints {
    BoxConstraints(BoxConstraints),
}

impl Constraints {
    pub fn is_tight(&self) -> bool {
        self.box_constraints().is_tight()
    }

    pub fn box_constraints(&self) -> BoxConstraints {
        match self {
            Constraints::BoxConstraints(constraints) => constraints.clone(),
        }
    }
}

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

    pub(crate) fn translate(&self, _dx: f64, _dy: f64) {
        todo!()
    }

    pub(crate) fn invert(&self) -> f64 {
        todo!()
    }

    pub(crate) fn perspective_transform(&mut self, _point: Vector3) -> Vector3 {
        todo!()
    }
}
pub struct Rect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Rect {
    pub fn from_size(size: Size) -> Self {
        Rect {
            x0: 0.,
            y0: 0.,
            x1: size.width,
            y1: size.height,
        }
    }

    pub(crate) fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    pub(crate) fn height(&self) -> f64 {
        self.y1 - self.y0
    }
}
pub struct PointerEvent {}

pub struct HitTestEntry {}

pub struct PaintContext {
    paint_bounds: Rect,
    layer: Layer,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
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

impl From<Offset> for Point {
    fn from(offset: Offset) -> Self {
        Point::new(offset.dx, offset.dy)
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
    RenderView(RenderView),
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
            RenderObject::RenderView(boxed) => boxed.state(process),
        }
    }
}

impl RenderObject {
    pub fn new_render_box(widget: Box<dyn RenderBoxWidget>) -> RenderObject {
        RenderBox::new_render_object(widget)
    }

    pub(crate) fn new_render_view(child: RenderObject, size: Size) -> RenderObject {
        RenderView::new_render_object(child, size)
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
            RenderObject::RenderSliver(_) => unreachable!(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub(crate) fn render_sliver(&self) -> RenderSliver {
        match self {
            RenderObject::RenderBox(_) => unreachable!(),
            RenderObject::RenderSliver(boxed) => boxed.clone(),
            RenderObject::RenderView(_boxed) => todo!(),
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
        match self {
            RenderObject::RenderView(boxed) => boxed.set_relayout_boundary(Some(self.clone())),
            _ => unreachable!(),
        }
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
}

impl RenderObject {
    //-- begin delegate methods --//
    pub(crate) fn parent_data(&self) -> ParentData {
        match self {
            RenderObject::RenderBox(s) => s.parent_data(),
            RenderObject::RenderSliver(s) => todo!(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub(crate) fn try_parent_data(&self) -> Option<ParentData> {
        match self {
            RenderObject::RenderBox(s) => s.try_parent_data(),
            RenderObject::RenderSliver(s) => todo!(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub(crate) fn mark_needs_layout(&self) {
        match self {
            RenderObject::RenderBox(s) => s.mark_needs_layout(),
            RenderObject::RenderSliver(s) => s.mark_needs_layout(),
            RenderObject::RenderView(s) => s.mark_needs_layout(),
        }
    }

    pub fn needs_layout(&self) -> bool {
        match self {
            RenderObject::RenderBox(s) => s.needs_layout(),
            RenderObject::RenderSliver(s) => s.needs_layout(),
            RenderObject::RenderView(s) => s.needs_layout(),
        }
    }

    pub fn needs_paint(&self) -> bool {
        match self {
            RenderObject::RenderBox(s) => s.needs_paint(),
            RenderObject::RenderSliver(s) => s.needs_paint(),
            RenderObject::RenderView(s) => s.needs_paint(),
        }
    }

    pub fn relayout_boundary(&self) -> RenderObject {
        match self {
            RenderObject::RenderBox(s) => s.relayout_boundary(),
            RenderObject::RenderSliver(s) => s.relayout_boundary(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub(crate) fn owner(&self) -> PipelineOwner {
        match self {
            RenderObject::RenderBox(s) => s.owner(),
            RenderObject::RenderSliver(s) => todo!(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub(crate) fn try_owner(&self) -> Option<PipelineOwner> {
        match self {
            RenderObject::RenderBox(s) => s.try_owner(),
            RenderObject::RenderSliver(s) => todo!(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub fn mark_needs_paint(&self) {
        match self {
            RenderObject::RenderBox(s) => s.mark_needs_paint(),
            RenderObject::RenderSliver(s) => s.mark_needs_paint(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub(crate) fn clean_relayout_boundary(&self) {
        match self {
            RenderObject::RenderBox(s) => s.clean_relayout_boundary(),
            RenderObject::RenderSliver(s) => s.clean_relayout_boundary(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub(crate) fn propagate_relayout_bondary(&self) {
        match self {
            RenderObject::RenderBox(s) => s.propagate_relayout_bondary(),
            RenderObject::RenderSliver(s) => s.propagate_relayout_bondary(),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

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
    pub fn layout(&self, constraints: Constraints, parent_use_size: bool) {
        match self {
            RenderObject::RenderBox(s) => s.layout(constraints, parent_use_size),
            RenderObject::RenderSliver(s) => s.layout(constraints, parent_use_size),
            RenderObject::RenderView(_boxed) => todo!(),
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
            RenderObject::RenderView(_boxed) => todo!(),
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
            RenderObject::RenderView(s) => s.is_repaint_bondary(),
        }
    }

    pub(crate) fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints)) {
        match self {
            RenderObject::RenderBox(s) => s.invoke_layout_callback(callback),
            RenderObject::RenderSliver(s) => s.invoke_layout_callback(callback),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    /// Override this method to handle pointer events that hit this render object.
    pub fn handle_event(&self, event: PointerEvent, entry: HitTestEntry) {
        match self {
            RenderObject::RenderBox(s) => todo!(), //s.handle_event(event, entry),
            RenderObject::RenderSliver(s) => s.handle_event(event, entry),
            RenderObject::RenderView(s) => s.handle_event(event, entry),
        }
    }

    pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
        match self {
            RenderObject::RenderBox(s) => s.paint_with_context(context, offset),
            RenderObject::RenderSliver(s) => s.paint_with_context(context, offset),
            RenderObject::RenderView(s) => s.paint_with_context(context, offset),
        }
    }

    pub(crate) fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4) {
        match self {
            RenderObject::RenderBox(s) => s.apply_paint_transform(child, transform),
            RenderObject::RenderSliver(s) => s.apply_paint_transform(child, transform),
            RenderObject::RenderView(_boxed) => todo!(),
        }
    }

    pub(crate) fn layout_without_resize(&self) {
        match self {
            RenderObject::RenderBox(s) => s.layout_without_resize(),
            RenderObject::RenderSliver(_s) => todo!("s.layout_without_resize()"),
            RenderObject::RenderView(s) => s.layout_without_resize(),
        }
    }

    pub(crate) fn layer(&self) -> Option<Layer> {
        match self {
            RenderObject::RenderBox(s) => s.layer(),
            RenderObject::RenderSliver(_s) => todo!(),
            RenderObject::RenderView(s) => s.layer(),
        }
    }

    pub(crate) fn paint_bounds(&self) -> Rect {
        match self {
            RenderObject::RenderBox(s) => s.paint_bounds(),
            RenderObject::RenderSliver(_s) => todo!(),
            RenderObject::RenderView(s) => s.paint_bounds(),
        }
    }

    pub(crate) fn set_layer(&self, child_layer: Layer) {
        match self {
            RenderObject::RenderBox(s) => s.set_layer(child_layer),
            RenderObject::RenderSliver(_s) => todo!(),
            RenderObject::RenderView(s) => s.set_layer(child_layer),
        }
    }

    pub(crate) fn set_size(&self, size: Size) {
        match self {
            RenderObject::RenderBox(s) => s.set_size(size),
            RenderObject::RenderSliver(_s) => todo!(),
            RenderObject::RenderView(_s) => todo!(),
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

    pub fn is_alive(&self) -> bool {
        match self {
            WeakRenderObject::RenderBox(o) => o.is_alive(),
            WeakRenderObject::RenderSliver(o) => o.is_alive(),
            WeakRenderObject::RenderView(_) => unreachable!(),
        }
    }
}

impl PaintContext {
    pub(crate) fn new(layer: Layer, rect: Rect) -> Self {
        Self {
            layer,
            paint_bounds: rect,
        }
    }

    pub(crate) fn paint_child(&mut self, child: &RenderObject, offset: Offset) {
        if child.is_repaint_bondary() {
            self.composite_child(child, offset);
        } else {
            child.paint_with_context(self, offset)
        }
    }

    pub fn draw_text(&mut self, layout: &PietTextLayout, offset: Offset) {
        self.layer.with_piet(|p| p.draw_text(layout, offset))
    }

    pub fn fill(&mut self) {
        self.layer
            .with_piet(|p| p.fill(Circle::new((10., 10.), 10.), &Color::BLACK));
    }

    pub(crate) fn repaint_composited_child(child: &RenderObject, piet: &mut Piet) {
        assert!(child.needs_paint());
        assert!(child.is_repaint_bondary());
        let child_layer = match child.layer() {
            None => {
                let bounds = child.paint_bounds();
                let size = Size {
                    width: bounds.width(),
                    height: bounds.height(),
                };

                let child_layer = Layer::new(piet, size);
                child.set_layer(child_layer.clone());
                child_layer
            }
            Some(layer) => {
                layer.clear_children();
                layer
            }
        };

        let rect = child.paint_bounds();
        eprintln!("repaint_composited_child");
        let mut paint_context = PaintContext::new(child_layer, rect);
        child.paint_with_context(&mut paint_context, Offset::ZERO);
    }

    fn composite_child(&mut self, child: &RenderObject, _offset: Offset) {
        assert!(child.is_repaint_bondary());
        if child.needs_paint() {
            self.layer.with_piet(|p| {
                Self::repaint_composited_child(child, p);
            });
        }
        self.layer.add_child(child.layer().unwrap());
    }
}
