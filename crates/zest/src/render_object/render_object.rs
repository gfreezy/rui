use std::{
    any::Any,
    cell::RefCell,
    fmt::Debug,
    ops::{Add, Mul, Neg, Sub},
    rc::{Rc, Weak},
};

use cgmath::{SquareMatrix, Transform};
use druid_shell::{
    kurbo::{Circle, Point},
    piet::{Color, Piet, PietTextLayout, RenderContext},
    MouseEvent,
};

use super::{
    layer::Layer,
    pipeline_owner::PipelineOwner,
    render_box::{
        BoxConstraints, BoxHitTestEntry, HitTestResult, RenderBox, RenderBoxWidget, Size,
        WeakRenderBox,
    },
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

#[derive(Debug, Clone, PartialEq)]
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
pub struct Matrix4(cgmath::Matrix4<f64>);

impl Matrix4 {
    pub fn identity() -> Matrix4 {
        Matrix4(cgmath::Matrix4::identity())
    }

    pub(crate) fn translate(&self, dx: f64, dy: f64) {
        self.0.transform_point(cgmath::Point3::new(dx, dy, 0.));
    }

    pub(crate) fn invert(&self) -> f64 {
        todo!()
    }

    pub(crate) fn perspective_transform(&mut self, _point: Vector3) -> Vector3 {
        todo!()
    }

    pub(crate) fn from_translation(dx: f64, dy: f64) -> Matrix4 {
        Matrix4(cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            dx, dy, 0.,
        )))
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

    pub(crate) fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }
}
pub type PointerEvent = MouseEvent;

#[derive(Clone)]
pub enum HitTestEntry {
    BoxHitTestEntry(BoxHitTestEntry),
    SliverHitTestEntry(SliverHitTestEntry),
}

#[derive(Clone)]
pub struct SliverHitTestEntry {
    render_object: WeakRenderObject,
}

impl SliverHitTestEntry {
    pub fn target(&self) -> RenderObject {
        self.render_object.upgrade()
    }
}

impl HitTestEntry {
    pub fn to_box_hit_test_entry(self) -> BoxHitTestEntry {
        match self {
            HitTestEntry::BoxHitTestEntry(entry) => entry,
            HitTestEntry::SliverHitTestEntry(_entry) => todo!(),
        }
    }

    pub(crate) fn new_box_hit_test_entry(
        render_object: WeakRenderObject,
        position: Offset,
    ) -> Self {
        HitTestEntry::BoxHitTestEntry(BoxHitTestEntry::new(render_object, position))
    }

    delegate::delegate! {
        to match self {
            HitTestEntry::BoxHitTestEntry(e) => e,
            HitTestEntry::SliverHitTestEntry(e) => e,
        } {
            pub fn target(&self) -> RenderObject;
        }
    }
}

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

impl Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Offset {
        Offset {
            dx: -self.dx,
            dy: -self.dy,
        }
    }
}

impl From<Offset> for Point {
    fn from(offset: Offset) -> Self {
        Point::new(offset.dx, offset.dy)
    }
}

impl From<Point> for Offset {
    fn from(p: Point) -> Self {
        Offset::new(p.x, p.y)
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

impl ParentData {
    pub fn new<T: Any + 'static>(data: T) -> Self {
        ParentData {
            inner: Rc::new(RefCell::new(data)),
        }
    }

    pub fn with<T: 'static, R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&*self.inner.borrow_mut().downcast_ref().unwrap())
    }

    pub fn with_mut<T: 'static, R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut *self.inner.borrow_mut().downcast_mut::<T>().unwrap())
    }
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

impl RenderObject {
    pub(crate) fn redepth_child(&self, child: &RenderObject) {
        if child.depth() <= self.depth() {
            child.incr_depth();
            child.redepth_children();
        }
    }

    delegate::delegate! {
        to match self {
            RenderObject::RenderBox(box_) => box_,
            RenderObject::RenderSliver(sliver) => sliver,
            RenderObject::RenderView(view) => view,
        } {
            pub(crate) fn parent(&self) -> RenderObject;

            pub(crate) fn parent_data(&self) -> ParentData;

            pub(crate) fn try_parent_data(&self) -> Option<ParentData>;

            pub(crate) fn with_parent_data<T: 'static, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R>;

            pub(crate) fn try_parent(&self) -> Option<RenderObject>;

            pub(crate) fn first_child(&self) -> RenderObject;

            pub(crate) fn try_first_child(&self) -> Option<RenderObject>;

            pub(crate) fn last_child(&self) -> RenderObject;

            pub(crate) fn try_last_child(&self) -> Option<RenderObject>;

            pub(crate) fn next_sibling(&self) -> RenderObject;

            pub(crate) fn prev_sibling(&self) -> RenderObject;

            pub(crate) fn set_parent(&self, element: Option<RenderObject>);

            pub(crate) fn try_next_sibling(&self) -> Option<RenderObject>;

            pub(crate) fn try_prev_sibling(&self) -> Option<RenderObject>;

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

            pub(crate) fn depth(&self) -> usize;

            pub(crate) fn child_count(&self) -> usize;

            pub(crate) fn redepth_children(&self);

            pub(crate) fn relayout_boundary(&self) -> RenderObject;

            pub(crate) fn try_relayout_boundary(&self) -> Option<RenderObject>;

            pub(crate) fn owner(&self) -> PipelineOwner;

            pub(crate) fn try_owner(&self) -> Option<PipelineOwner>;

            pub(crate) fn needs_layout(&self) -> bool;

            pub(crate) fn needs_paint(&self) -> bool;

            pub(crate) fn try_constraints(&self) -> Option<Constraints>;

            pub(crate) fn constraints(&self) -> Constraints;

            pub(crate) fn doing_this_layout_with_callback(&self) -> bool;

            pub(crate) fn try_layer(&self) -> Option<Layer>;

            pub(crate) fn layer(&self) -> Layer;

            pub(crate) fn set_relayout_boundary(&self, relayout_boundary: Option<RenderObject>);

            pub(crate) fn clean_relayout_boundary(&self);

            pub(crate) fn propagate_relayout_bondary(&self);

            pub(crate) fn mark_needs_layout(&self);

            pub(crate) fn clear_needs_layout(&self);

            pub(crate) fn mark_parent_needs_layout(&self);

            pub(crate) fn set_owner(&self, owner: Option<PipelineOwner>);

            pub(crate) fn clear_needs_paint(&self);

            pub(crate) fn mark_needs_paint(&self);

            pub(crate) fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints));

            pub(crate) fn set_layer(&self, layer: Option<Layer>);

            pub(crate) fn incr_depth(&self);

            pub(crate) fn clear_child_count(&self);

            pub(crate) fn incr_child_count(&self);

            pub(crate) fn decr_child_count(&self);

            pub(crate) fn set_constraints(&self, c: Constraints);

            pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset);
            pub(crate) fn visit_children(&self, visitor: impl FnMut(RenderObject));

            pub(crate) fn is_repaint_bondary(&self) -> bool;
            pub(crate) fn handle_event(&self, event: PointerEvent, entry: HitTestEntry);
            pub(crate) fn layout_without_resize(&self);
            pub(crate) fn layout(&self, constraints: Constraints, parent_use_size: bool);
            pub(crate) fn get_dry_layout(&self, constraints: Constraints) -> Size;
            pub(crate) fn paint_bounds(&self) -> Rect;

            pub(crate) fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4);

            pub(crate) fn to_string_short(&self) -> String;
            pub(crate) fn to_string_deep(&self) -> String;

        }
    }

    pub(crate) fn hit_test(&self, result: &mut HitTestResult, position: Offset) -> bool {
        tracing::debug!("hit_test in {:?}", self);
        match self {
            RenderObject::RenderBox(o) => o.hit_test(result, position),
            RenderObject::RenderSliver(o) => o.hit_test(result, position),
            RenderObject::RenderView(o) => o.hit_test(result, position),
        }
    }
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

    pub(crate) fn repaint_composited_child(child: &RenderObject, offset: Offset, piet: &mut Piet) {
        assert!(child.needs_paint());
        assert!(child.is_repaint_bondary());
        let child_bounds = child.paint_bounds();
        let child_layer = match child.try_layer() {
            Some(layer) if layer.size() == child_bounds.size() => {
                layer.clear_children();
                layer.clear();
                layer.set_offset(offset);
                layer
            }
            _ => {
                let bounds = &child_bounds;
                let size = Size {
                    width: bounds.width(),
                    height: bounds.height(),
                };

                let child_layer = Layer::new(piet, size, offset);
                child.set_layer(Some(child_layer.clone()));
                child_layer
            }
        };

        let mut paint_context = PaintContext::new(child_layer, child_bounds);
        child.paint_with_context(&mut paint_context, Offset::ZERO);
    }

    fn composite_child(&mut self, child: &RenderObject, offset: Offset) {
        assert!(child.is_repaint_bondary());
        if child.needs_paint() {
            self.layer.with_piet(|p| {
                Self::repaint_composited_child(child, offset, p);
            });
        }
        let child_layer = child.layer();
        child_layer.set_offset(offset);
        self.layer.add_child(child_layer);
    }
}
