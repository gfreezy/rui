use std::{
    cell::RefCell,
    collections::{HashMap},
    hash::Hash,
    rc::{Rc, Weak},
};

use decorum::{R64};


use crate::render_object::render_object::RenderObject;

use super::{
    abstract_node::AbstractNode,
    render_object::{
        Constraints, Matrix4, Offset, PaintContext, PointerEvent, Rect, Vector3, WeakRenderObject,
    },
    render_object_state::RenderObjectState,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub const ZERO: Self = Size {
        width: 0.0,
        height: 0.0,
    };

    fn contains(&self, position: Offset) -> bool {
        position.dx >= 0.0
            && position.dx < self.width
            && position.dy >= 0.0
            && position.dy < self.height
    }
}

impl From<druid_shell::kurbo::Size> for Size {
    fn from(size: druid_shell::kurbo::Size) -> Self {
        Size {
            width: size.width,
            height: size.height,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoxConstraints {
    min_width: f64,
    max_width: f64,
    min_height: f64,
    max_height: f64,
}

impl PartialEq for BoxConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.min_width == other.min_width
            && self.max_width == other.max_width
            && self.min_height == other.min_height
            && self.max_height == other.max_height
    }
}

impl Eq for BoxConstraints {}

impl Hash for BoxConstraints {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        R64::from(self.min_width).hash(state);
        R64::from(self.max_width).hash(state);
        R64::from(self.min_height).hash(state);
        R64::from(self.max_height).hash(state);
    }
}

impl From<BoxConstraints> for Constraints {
    fn from(bc: BoxConstraints) -> Self {
        Constraints::BoxConstraints(bc)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum InstrinsicDimension {
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
struct InstrinsicDimensionsCacheEntry {
    dimension: InstrinsicDimension,
    argument: R64,
}
#[derive(Clone)]
pub struct RenderBox {
    inner: Rc<RefCell<InnerRenderBox>>,
}

impl PartialEq for RenderBox {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

pub trait RenderBoxWidget {
    fn paint(&mut self, this: &RenderObject, paint_context: &mut PaintContext, offset: Offset) {
        let mut child = this.try_first_child();
        while let Some(c) = child {
            let offset_in_parent = c.render_box().offset();
            paint_context.paint_child(&c, offset_in_parent + offset);
            child = c.try_next_sibling();
        }
    }

    fn handle_event(&self, _event: PointerEvent, _entry: BoxHitTestEntry) {}

    fn is_repaint_boundary(&self) -> bool {
        false
    }

    fn sized_by_parent(&self) -> bool {
        false
    }

    fn compute_min_instrinsic_width(&self, _this: &RenderObject, _height: f64) -> f64 {
        0.0
    }

    fn compute_max_instrinsic_width(&self, _this: &RenderObject, _height: f64) -> f64 {
        0.0
    }

    fn compute_min_instrinsic_height(&self, _this: &RenderObject, _width: f64) -> f64 {
        0.0
    }

    fn compute_max_instrinsic_height(&self, _this: &RenderObject, _width: f64) -> f64 {
        0.0
    }

    fn compute_dry_layout(&mut self, _this: &RenderObject, _constraints: BoxConstraints) -> Size {
        Size::ZERO
    }

    fn perform_resize(&mut self, _this: &RenderObject) {}

    fn perform_layout(&mut self, _this: &RenderObject) {
        eprintln!("widget: perform layout")
    }

    fn hit_test_self(&mut self, _this: &RenderObject, _position: Offset) -> bool {
        false
    }

    fn hit_test_children(
        &mut self,
        this: &RenderObject,
        result: &mut BoxHitTestResult,
        position: Offset,
    ) -> bool {
        let mut child = this.try_last_child();
        while let Some(c) = child {
            let offset = c.render_box().offset();
            let is_hit = result.add_with_paint_offset(offset, position, |result, transformed| {
                assert_eq!(transformed, position - offset);
                c.render_box().hit_test(result, transformed)
            });
            if is_hit {
                return true;
            }
            child = c.try_prev_sibling();
        }
        false
    }
}

impl RenderBox {
    pub(crate) fn new_render_object(widget: Box<dyn RenderBoxWidget>) -> RenderObject {
        let render_box = Self {
            inner: Rc::new(RefCell::new(InnerRenderBox {
                object: Some(widget),
                ..Default::default()
            })),
        };
        let render_object = RenderObject::RenderBox(render_box);
        render_object.set_this(render_object.clone());
        render_object
    }

    pub fn downgrade(&self) -> WeakRenderBox {
        WeakRenderBox {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn state<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R {
        process(&mut self.inner.borrow_mut().state)
    }

    pub(crate) fn mark_needs_paint(&self) {
        self.state(|s| s.mark_needs_paint())
    }

    pub(crate) fn clean_relayout_boundary(&self) {
        self.state(|s| s.clean_relayout_boundary())
    }

    pub(crate) fn propagate_relayout_bondary(&self) {
        self.state(|s| s.propagate_relayout_bondary())
    }

    pub(crate) fn relayout_boundary(&self) -> RenderObject {
        self.state(|s| s.relayout_boundary())
    }

    pub(crate) fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints)) {
        self.state(|s| s.invoke_layout_callback(callback))
    }

    pub(crate) fn needs_layout(&self) -> bool {
        self.state(|s| s.needs_layout)
    }

    pub(crate) fn needs_paint(&self) -> bool {
        self.state(|s| s.needs_paint)
    }

    pub(crate) fn is_repaint_bondary(&self) -> bool {
        self.with_widget(|w, _| w.is_repaint_boundary())
    }

    pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
        self.state(|s| s.needs_paint = false);
        self.paint(context, offset);
        assert!(!self.needs_layout());
        assert!(!self.needs_paint());
    }

    pub(crate) fn try_relayout_boundary(&self) -> Option<RenderObject> {
        self.state(|s| s.try_relayout_boundary())
    }

    pub(crate) fn doing_this_layout_with_callback(&self) -> bool {
        self.state(|s| s.doing_this_layout_with_callback)
    }

    pub(crate) fn layout_without_resize(&self) {
        assert_eq!(&self.relayout_boundary(), &self.this());
        assert!(!self.doing_this_layout_with_callback());
        self.perform_layout();
        self.state(|s| s.needs_layout = false);
        self.mark_needs_paint();
    }

    pub(crate) fn layer(&self) -> Option<super::layer::Layer> {
        self.state(|s| s.layer.clone())
    }

    pub(crate) fn set_layer(&self, child_layer: super::layer::Layer) {
        self.state(|s| s.layer = Some(child_layer));
    }

    pub(crate) fn set_size(&self, size: Size) {
        self.inner.borrow_mut().size = Some(size);
    }
}

#[derive(Clone)]
pub struct WeakRenderBox {
    inner: Weak<RefCell<InnerRenderBox>>,
}

impl WeakRenderBox {
    pub fn upgrade(&self) -> RenderBox {
        self.inner
            .upgrade()
            .map(|inner| RenderBox { inner })
            .unwrap()
    }
    pub fn is_alive(&self) -> bool {
        self.inner.upgrade().is_some()
    }
}

#[derive(Default)]
struct InnerRenderBox {
    state: RenderObjectState,
    object: Option<Box<dyn RenderBoxWidget + 'static>>,
    size: Option<Size>,
    offset: Offset,
    constraints: Option<BoxConstraints>,
    cached_instrinsic_dimensions: HashMap<InstrinsicDimensionsCacheEntry, f64>,
    cached_dry_layout_sizes: HashMap<BoxConstraints, Size>,
}

impl AbstractNode for RenderBox {
    fn node<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R {
        process(&mut self.inner.borrow_mut().state)
    }
}

impl RenderBox {
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
    pub(crate) fn paint(&self, context: &mut PaintContext, offset: Offset) {
        self.with_widget(|w, this| w.paint(this, context, offset))
    }

    pub(crate) fn sized_by_parent(&self) -> bool {
        self.with_widget(|w, _this| w.sized_by_parent())
    }

    fn compute_min_instrinsic_width(&self, height: f64) -> f64 {
        self.with_widget(|w, this| w.compute_min_instrinsic_width(this, height))
    }

    fn compute_max_instrinsic_width(&self, height: f64) -> f64 {
        self.with_widget(|w, this| w.compute_max_instrinsic_width(this, height))
    }

    fn compute_min_instrinsic_height(&self, width: f64) -> f64 {
        self.with_widget(|w, this| w.compute_min_instrinsic_height(this, width))
    }

    fn compute_max_instrinsic_height(&self, width: f64) -> f64 {
        self.with_widget(|w, this| w.compute_max_instrinsic_height(this, width))
    }

    fn compute_dry_layout(&self, constraints: BoxConstraints) -> Size {
        self.with_widget(|w, this| w.compute_dry_layout(this, constraints))
    }

    pub(crate) fn perform_resize(&self) {
        self.with_widget(|w, this| w.perform_resize(this))
    }

    pub(crate) fn perform_layout(&self) {
        self.with_widget(|w, this| w.perform_layout(this))
    }

    pub(crate) fn hit_test_self(&self, position: Offset) -> bool {
        self.with_widget(|w, this| w.hit_test_self(this, position))
    }

    pub(crate) fn hit_test_children(
        &self,
        result: &mut BoxHitTestResult,
        position: Offset,
    ) -> bool {
        self.with_widget(|w, this| w.hit_test_children(this, result, position))
    }
}

impl RenderBox {
    fn with_widget<T>(&self, f: impl FnOnce(&mut dyn RenderBoxWidget, &RenderObject) -> T) -> T {
        let mut widget = self.inner.borrow_mut().object.take().unwrap();
        let ret = f(&mut *widget, &self.this());
        self.inner.borrow_mut().object.replace(widget);
        ret
    }

    fn compute_intrinsic_dimensions(
        &self,
        dimension: InstrinsicDimension,
        argument: f64,
        computer: impl FnOnce(f64) -> f64,
    ) -> f64 {
        let should_cache = true;
        if should_cache {
            let key = InstrinsicDimensionsCacheEntry {
                dimension,
                argument: argument.into(),
            };
            let ref_mut = &mut self.inner.borrow_mut().cached_instrinsic_dimensions;
            let ret = ref_mut.entry(key).or_insert_with(|| computer(argument));
            *ret
        } else {
            computer(argument)
        }
    }

    fn get_min_instrinsic_width(&self, height: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MinWidth, height, |_width| {
            self.compute_min_instrinsic_width(height)
        })
    }

    fn get_max_instrinsic_width(&self, height: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MaxWidth, height, |_width| {
            self.compute_max_instrinsic_width(height)
        })
    }

    fn get_min_instrinsic_height(&self, width: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MinHeight, width, |width| {
            self.compute_min_instrinsic_height(width)
        })
    }

    fn get_max_instrinsic_height(&self, width: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MaxHeight, width, |width| {
            self.compute_max_instrinsic_height(width)
        })
    }

    fn try_constraints(&self) -> Option<Constraints> {
        self.state(|s| s.constraints.clone())
    }

    fn constraints(&self) -> Constraints {
        self.try_constraints().unwrap()
    }

    fn box_constraints(&self) -> BoxConstraints {
        self.constraints().box_constraints()
    }

    fn get_dry_layout(&self, constraints: BoxConstraints) -> Size {
        let should_cache = true;
        if should_cache {
            let ref_mut = &mut self.inner.borrow_mut().cached_dry_layout_sizes;
            ref_mut
                .entry(constraints.clone())
                .or_insert_with(|| self.compute_dry_layout(constraints))
                .clone()
        } else {
            self.compute_dry_layout(constraints)
        }
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
    pub(crate) fn layout(&self, constraints: Constraints, parent_use_size: bool) {
        let is_relayout_boundary = !parent_use_size
            || self.sized_by_parent()
            || constraints.is_tight()
            || self.try_parent().is_none();
        let relayout_boundary = if is_relayout_boundary {
            self.this()
        } else {
            self.parent().relayout_boundary()
        };
        if !self.needs_layout()
            && Some(&constraints) == self.try_constraints().as_ref()
            && Some(relayout_boundary.clone()) != self.try_relayout_boundary()
        {
            self.state(|s| s.set_relayout_boundary(Some(relayout_boundary)));
            self.visit_children(|e| e.propagate_relayout_bondary());
            return;
        }

        self.state(|s| s.constraints = constraints.into());
        if self.try_relayout_boundary().is_some() && self.relayout_boundary() != relayout_boundary {
            self.visit_children(|e| e.clean_relayout_boundary());
        }
        self.state(|s| s.set_relayout_boundary(Some(relayout_boundary)));
        assert!(!self.state(|s| s.doing_this_layout_with_callback));

        if self.sized_by_parent() {
            self.perform_resize();
        }

        self.perform_layout();
        self.state(|s| s.needs_layout = false);
        self.mark_needs_paint();
    }

    pub fn has_size(&self) -> bool {
        self.inner.borrow().size.is_some()
    }

    pub fn size(&self) -> Size {
        self.inner.borrow().size.expect("no size available")
    }

    pub fn offset(&self) -> Offset {
        self.inner.borrow().offset
    }

    pub(crate) fn mark_needs_layout(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.cached_instrinsic_dimensions.clear();
        inner.cached_dry_layout_sizes.clear();
        if inner.state.try_parent().is_some() {
            inner.state.mark_parent_needs_layout();
        } else {
            inner.state.mark_needs_layout();
        }
    }
    pub(crate) fn hit_test(&self, result: &mut BoxHitTestResult, position: Offset) -> bool {
        if self.size().contains(position) {
            if self.hit_test_children(result, position) || self.hit_test_self(position) {
                result.add(BoxHitTestEntry::new(self.this().downgrade(), position));
            }
        }
        return false;
    }

    pub(crate) fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4) {
        assert_eq!(child.parent(), self.this());

        let offset = child.render_box().offset();
        transform.translate(offset.dx, offset.dy);
    }

    pub(crate) fn global_to_local(&self, point: Offset, ancestor: Option<RenderObject>) -> Offset {
        let mut transform = self.state(|s| s.get_transform_to(ancestor));
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

    pub(crate) fn local_to_global(&self, _point: Offset, _ancestor: Option<RenderObject>) -> Offset {
        todo!()
    }

    pub(crate) fn paint_bounds(&self) -> Rect {
        Rect::from_size(self.size())
        // Offset::ZERO & self.size()
    }

    pub(crate) fn handle_event(&self, _event: PointerEvent, _entry: BoxHitTestEntry) {
        todo!()
    }
}

pub struct BoxHitTestResult {
    entries: Vec<BoxHitTestEntry>,
}

impl BoxHitTestResult {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add(&mut self, entry: BoxHitTestEntry) {
        self.entries.push(entry);
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn add_with_paint_offset(
        &self,
        _offset: Offset,
        _position: Offset,
        _hit_test: impl FnOnce(&mut BoxHitTestResult, Offset) -> bool,
    ) -> bool {
        todo!()
    }
}

pub struct BoxHitTestEntry {
    render_object: WeakRenderObject,
    position: Offset,
}

impl BoxHitTestEntry {
    fn new(render_object: WeakRenderObject, position: Offset) -> Self {
        Self {
            render_object,
            position,
        }
    }
}
impl BoxConstraints {
    pub fn has_tight_width(&self) -> bool {
        self.min_width >= self.max_width
    }

    pub fn has_tight_height(&self) -> bool {
        self.min_height >= self.max_height
    }

    pub(crate) fn is_tight(&self) -> bool {
        self.has_tight_width() && self.has_tight_height()
    }

    pub(crate) fn tight(size: Size) -> Self {
        BoxConstraints {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }
}