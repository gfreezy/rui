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
    layer::Layer,
    pipeline_owner::PipelineOwner,
    render_box::{BoxConstraints, RenderBox, RenderBoxWidget, Size, WeakRenderBox},
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

#[enum_dispatch::enum_dispatch]
pub(crate) trait AbstractNode {
    fn parent(&self) -> RenderObject;

    fn try_parent(&self) -> Option<RenderObject>;

    fn first_child(&self) -> RenderObject;

    fn try_first_child(&self) -> Option<RenderObject>;

    fn last_child(&self) -> RenderObject;

    fn try_last_child(&self) -> Option<RenderObject>;

    fn next_sibling(&self) -> RenderObject;

    fn prev_sibling(&self) -> RenderObject;

    fn set_parent(&self, element: Option<RenderObject>);

    fn try_next_sibling(&self) -> Option<RenderObject>;

    fn try_prev_sibling(&self) -> Option<RenderObject>;

    fn set_next_sibling(&self, element: Option<RenderObject>);

    fn set_prev_sibling(&self, element: Option<RenderObject>);

    fn set_first_child(&self, element: Option<RenderObject>);

    fn set_last_child(&self, element: Option<RenderObject>);

    fn set_last_child_if_none(&self, element: Option<RenderObject>);

    fn attach(&self, owner: PipelineOwner);

    fn detach(&self);

    /// Mark the given node as being a child of this node.
    ///
    /// Subclasses should call this function when they acquire a new child.
    fn adopt_child(&self, child: &RenderObject);

    /// Disconnect the given node from this node.
    ///
    /// Subclasses should call this function when they lose a child.
    fn drop_child(&self, child: &RenderObject);

    /// Adjust the [depth] of the given [child] to be greater than this node's own
    /// [depth].
    ///
    /// Only call this method from overrides of [redepthChildren].

    fn redepth_child(&self, child: &RenderObject);

    /// Insert child into this render object's child list after the given child.
    ///
    /// If `after` is null, then this inserts the child at the start of the list,
    /// and the child becomes the new [firstChild].
    fn insert(&self, child: RenderObject, after: Option<RenderObject>);

    fn add(&self, child: RenderObject);

    fn remove(&self, child: &RenderObject);

    fn remove_all(&self);

    fn move_(&self, child: RenderObject, after: Option<RenderObject>);

    fn depth(&self) -> usize;

    fn child_count(&self) -> usize;

    fn redepth_children(&self);

    fn visit_children(&self, mut visitor: impl FnMut(RenderObject)) {
        // attach children
        let mut child = self.try_first_child();
        while let Some(c) = child {
            visitor(c.clone());
            child = c.try_next_sibling();
        }
    }

    fn relayout_boundary(&self) -> RenderObject;

    fn try_relayout_boundary(&self) -> Option<RenderObject>;

    fn owner(&self) -> PipelineOwner;

    fn try_owner(&self) -> Option<PipelineOwner>;

    fn needs_layout(&self) -> bool;

    fn needs_paint(&self) -> bool;

    fn try_constraints(&self) -> Option<Constraints>;

    fn constraints(&self) -> Constraints;

    fn doing_this_layout_with_callback(&self) -> bool;

    fn try_layer(&self) -> Option<Layer>;

    fn layer(&self) -> Layer;

    fn set_relayout_boundary(&self, relayout_boundary: Option<RenderObject>);

    fn clean_relayout_boundary(&self);

    fn propagate_relayout_bondary(&self);

    fn mark_needs_layout(&self);

    fn clear_needs_layout(&self);

    fn mark_parent_needs_layout(&self);

    fn set_owner(&self, owner: Option<PipelineOwner>);

    fn clear_needs_paint(&self);

    fn mark_needs_paint(&self);

    fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints));

    fn set_layer(&self, layer: Option<Layer>);

    fn incr_depth(&self);

    fn clear_child_count(&self);

    fn incr_child_count(&self);

    fn decr_child_count(&self);

    fn set_constraints(&self, c: Constraints);
}

#[enum_dispatch::enum_dispatch]
pub(crate) trait AbstractNodeExt {
    fn is_repaint_bondary(&self) -> bool;
    fn paint_with_context(&self, context: &mut PaintContext, offset: Offset);
    fn handle_event(&self, event: PointerEvent, entry: HitTestEntry);
    fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints));
    fn layout_without_resize(&self);
    fn layout(&self, constraints: Constraints, parent_use_size: bool);
    fn paint_bounds(&self) -> Rect;
}

#[enum_dispatch::enum_dispatch(AbstractNode, AbstractNodeExt)]
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

    // pub(crate) fn global_to_local(&self, point: Offset, ancestor: Option<RenderObject>) -> Offset {
    //     let mut transform = self.get_transform_to(ancestor);
    //     let det = transform.invert();
    //     if det == 0.0 {
    //         return Offset::ZERO;
    //     }

    //     let n = Vector3::new(0.0, 0.0, 1.0);
    //     let _i = transform.perspective_transform(Vector3::new(0.0, 0.0, 0.0));
    //     let d = transform.perspective_transform(Vector3::new(0.0, 0.0, 0.0));
    //     let s = transform.perspective_transform(Vector3::new(point.dx, point.dy, 0.0));
    //     let p = s - d * (n.dot(s) / n.dot(d));
    //     Offset::new(p.x, p.y)
    // }

    // pub(crate) fn get_transform_to(&self, ancestor: Option<RenderObject>) -> Matrix4 {
    //     let ancestor = match ancestor {
    //         Some(a) => a,
    //         None => self.owner().root_node(),
    //     };
    //     let mut renderers = vec![self.clone()];
    //     let mut renderer = self.clone();
    //     while renderer != ancestor {
    //         renderers.push(renderer.clone());
    //         if let Some(r) = renderer.try_parent() {
    //             renderer = r.parent();
    //         } else {
    //             break;
    //         }
    //     }
    //     renderers.push(ancestor);

    //     let mut transform = Matrix4::identity();
    //     let mut iter = renderers.iter().rev().peekable();
    //     while let (Some(renderer), Some(next)) = (iter.next(), iter.peek()) {
    //         renderer.apply_paint_transform(next, &mut transform);
    //     }
    //     transform
    // }
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
            WeakRenderObject::RenderView(o) => true,
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
        let child_layer = match child.try_layer() {
            None => {
                let bounds = child.paint_bounds();
                let size = Size {
                    width: bounds.width(),
                    height: bounds.height(),
                };

                let child_layer = Layer::new(piet, size);
                child.set_layer(Some(child_layer.clone()));
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
        self.layer.add_child(child.layer());
    }
}
