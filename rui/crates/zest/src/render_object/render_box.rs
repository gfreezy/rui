use crate::constraints::{BoxConstraints, Constraints};
use crate::diagnostics::DiagnosticsNode;
use crate::geometry::{Matrix4, Offset, Rect, Size};
use crate::hit_test::{BoxHitTestEntry, HitTestEntry, HitTestResult};
use crate::paint_context::PaintContext;
use crate::pointer_event::PointerEvent;
use crate::render_object::layer::Layer;
use crate::render_object::parent_data::ParentData;
use crate::render_object::pipeline_owner::{PipelineOwner, WeakOwner};
use crate::render_object::render_object::{RenderObject, WeakRenderObject};
use decorum::R64;
use std::any::type_name;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::{Rc, Weak};

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
    pub(crate) inner: Rc<RefCell<InnerRenderBox>>,
}

impl Debug for RenderBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderBox")
            .field("name", &self.name())
            .finish()
    }
}

impl PartialEq for RenderBox {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

pub trait RenderBoxWidget: Any {
    fn set_attribute(&mut self, _ctx: &RenderObject, _key: &str, _value: &str) {}
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn paint(&mut self, ctx: &RenderObject, paint_context: &mut PaintContext, offset: Offset) {
        let mut child = ctx.try_first_child();
        while let Some(c) = child {
            let offset_in_parent = c.render_box().offset();
            paint_context.paint_child(&c, offset_in_parent + offset);
            child = c.try_next_sibling();
        }
    }

    fn handle_event(&mut self, _ctx: &RenderObject, _event: PointerEvent, _entry: BoxHitTestEntry) {
    }

    fn is_repaint_boundary(&self) -> bool {
        false
    }

    fn sized_by_parent(&self) -> bool {
        false
    }

    fn compute_min_instrinsic_width(&self, ctx: &RenderObject, height: f64) -> f64 {
        if ctx.child_count() == 1 {
            return ctx
                .first_child()
                .render_box()
                .get_min_instrinsic_width(height);
        }
        0.0
    }

    fn compute_max_instrinsic_width(&self, ctx: &RenderObject, height: f64) -> f64 {
        if ctx.child_count() == 1 {
            return ctx
                .first_child()
                .render_box()
                .get_max_instrinsic_width(height);
        }
        0.0
    }

    fn compute_min_instrinsic_height(&self, ctx: &RenderObject, width: f64) -> f64 {
        if ctx.child_count() == 1 {
            return ctx
                .first_child()
                .render_box()
                .get_min_instrinsic_height(width);
        }
        0.0
    }

    fn compute_max_instrinsic_height(&self, ctx: &RenderObject, width: f64) -> f64 {
        if ctx.child_count() == 1 {
            return ctx
                .first_child()
                .render_box()
                .get_max_instrinsic_height(width);
        }
        0.0
    }

    fn compute_dry_layout(&mut self, ctx: &RenderObject, constraints: BoxConstraints) -> Size {
        if ctx.child_count() == 1 {
            return ctx
                .first_child()
                .render_box()
                .get_dry_layout(constraints.into());
        }
        Size::ZERO
    }

    fn perform_layout(&mut self, ctx: &RenderObject) {
        if ctx.child_count() == 1 {
            let child = ctx.first_child();
            child.layout(ctx.constraints(), true);
            ctx.render_box().set_size(child.size());
        }
    }

    fn hit_test(
        &mut self,
        ctx: &RenderObject,
        result: &mut HitTestResult,
        position: Offset,
    ) -> bool {
        if ctx.size().contains(position) {
            if self.hit_test_children(ctx, result, position) || self.hit_test_self(ctx, position) {
                result.add(HitTestEntry::new_box_hit_test_entry(ctx, position));
                return true;
            }
        }
        return false;
    }

    fn hit_test_self(&mut self, _ctx: &RenderObject, _position: Offset) -> bool {
        false
    }

    fn hit_test_children(
        &mut self,
        ctx: &RenderObject,
        result: &mut HitTestResult,
        position: Offset,
    ) -> bool {
        let mut child = ctx.try_last_child();
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

#[mixin::insert(RenderObjectState)]
pub(crate) struct InnerRenderBox {
    object: Option<Box<dyn RenderBoxWidget + 'static>>,
    size: Option<Size>,
    offset: Offset,
    cached_instrinsic_dimensions: HashMap<InstrinsicDimensionsCacheEntry, f64>,
    cached_dry_layout_sizes: HashMap<BoxConstraints, Size>,
}

impl Default for InnerRenderBox {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_string(),
            first_child: Default::default(),
            last_child: Default::default(),
            next_sibling: Default::default(),
            prev_sibling: Default::default(),
            child_count: Default::default(),
            depth: Default::default(),
            parent: Default::default(),
            owner: Default::default(),
            parent_data: Default::default(),
            needs_layout: true,
            needs_paint: true,
            relayout_boundary: Default::default(),
            doing_this_layout_with_callback: Default::default(),
            constraints: Default::default(),
            layer: Default::default(),
            object: Default::default(),
            size: Default::default(),
            offset: Default::default(),
            cached_instrinsic_dimensions: Default::default(),
            cached_dry_layout_sizes: Default::default(),
        }
    }
}

impl RenderBox {
    pub(crate) fn render_object(&self) -> RenderObject {
        RenderObject::RenderBox(self.clone())
    }

    pub(crate) fn new_render_object(
        name: String,
        widget: Box<dyn RenderBoxWidget>,
    ) -> RenderObject {
        let inner = RefCell::new(InnerRenderBox {
            object: Some(widget),
            name,
            ..Default::default()
        });
        let render_box = RenderBox {
            inner: Rc::new(inner),
        };

        render_box.render_object()
    }

    pub fn diagnostics(&self) -> DiagnosticsNode {
        let mut node = DiagnosticsNode::new(self.name());
        node.add_number_property("id", self.id() as f64);
        node.add_string_property("name", self.name());
        node.add_number_property("child_count", self.child_count() as f64);
        node.add_string_property("needs_layout", self.needs_layout().to_string());
        node.add_string_property("needs_paint", self.needs_paint().to_string());
        node.add_string_property("needs_paint", self.needs_paint().to_string());
        node
    }

    pub fn downgrade(&self) -> WeakRenderBox {
        WeakRenderBox {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn set_name(&self, name: String) {
        self.inner.borrow_mut().name = name;
    }

    pub(crate) fn name(&self) -> String {
        self.inner.borrow().name.clone()
    }

    pub(crate) fn update<T: 'static>(&self, update: impl FnOnce(&mut T)) {
        let ret = self.with_widget(|w, _| w.as_any_mut().downcast_mut::<T>().map(update));
        if ret.is_none() {
            tracing::error!("update mismatched Type: {:?}", type_name::<T>());
        }
    }

    pub(crate) fn set_attribute(&self, key: &str, value: &str) {
        self.with_widget(|w, this| w.set_attribute(this, key, value))
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
    pub(crate) fn paint(&self, context: &mut PaintContext, offset: Offset) {
        tracing::debug!(
            "painting in {}, layout offset: {:?}, paint offset: {:?}",
            self.name(),
            self.offset(),
            offset
        );
        self.with_widget(|w, this| w.paint(this, context, offset))
    }

    pub(crate) fn sized_by_parent(&self) -> bool {
        self.with_widget(|w, _this| w.sized_by_parent())
    }

    fn compute_dry_layout(&self, constraints: BoxConstraints) -> Size {
        self.with_widget(|w, this| w.compute_dry_layout(this, constraints))
    }

    pub(crate) fn perform_layout(&self) {
        self.with_widget(|w, this| w.perform_layout(this))
    }

    pub(crate) fn hit_test_self(&self, position: Offset) -> bool {
        self.with_widget(|w, this| w.hit_test_self(this, position))
    }

    pub(crate) fn hit_test_children(&self, result: &mut HitTestResult, position: Offset) -> bool {
        self.with_widget(|w, this| w.hit_test_children(this, result, position))
    }

    pub(crate) fn get_min_instrinsic_width(&self, height: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MinWidth, height, |_width| {
            self.with_widget(|w, this| w.compute_min_instrinsic_width(this, height))
        })
    }

    pub(crate) fn get_max_instrinsic_width(&self, height: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MaxWidth, height, |_width| {
            self.with_widget(|w, this| w.compute_max_instrinsic_width(this, height))
        })
    }

    pub(crate) fn get_min_instrinsic_height(&self, width: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MinHeight, width, |width| {
            self.with_widget(|w, this| w.compute_min_instrinsic_height(this, width))
        })
    }

    pub(crate) fn get_max_instrinsic_height(&self, width: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MaxHeight, width, |width| {
            self.with_widget(|w, this| w.compute_max_instrinsic_height(this, width))
        })
    }
    pub(crate) fn box_constraints(&self) -> BoxConstraints {
        self.constraints().box_constraints()
    }

    pub(crate) fn get_dry_layout(&self, constraints: BoxConstraints) -> Size {
        let should_cache = true;
        if should_cache {
            {
                let ref_mut = &mut self.inner.borrow_mut().cached_dry_layout_sizes;
                if let Some(size) = ref_mut.get(&constraints) {
                    return size.clone();
                }
            }
            let size = self.compute_dry_layout(constraints.clone());
            let ref_mut = &mut self.inner.borrow_mut().cached_dry_layout_sizes;
            ref_mut.insert(constraints, size);
            return size;
        } else {
            self.compute_dry_layout(constraints)
        }
    }

    pub(crate) fn set_size(&self, size: Size) {
        self.inner.borrow_mut().size = Some(size);
    }

    pub(crate) fn size(&self) -> Size {
        self.inner.borrow().size.expect("no size available")
    }

    pub(crate) fn offset(&self) -> Offset {
        self.inner.borrow().offset
    }

    pub(crate) fn set_offset(&self, offset: Offset) {
        self.inner.borrow_mut().offset = offset;
    }

    pub(crate) fn mark_needs_layout(&self) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.cached_instrinsic_dimensions.clear();
            inner.cached_dry_layout_sizes.clear();
        }
        if self.try_parent().is_some() {
            self.mark_parent_needs_layout();
        } else {
            self._mark_needs_layout();
        }
    }

    pub(crate) fn mark_parent_needs_layout(&self) {
        self._mark_parent_needs_layout()
    }

    // private methods
    fn with_widget<T>(&self, f: impl FnOnce(&mut dyn RenderBoxWidget, &RenderObject) -> T) -> T {
        let mut widget = self.inner.borrow_mut().object.take().unwrap();
        let ret = f(&mut *widget, &self.render_object());
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

    fn has_size(&self) -> bool {
        self.inner.borrow().size.is_some()
    }

    pub(crate) fn hit_test(&self, result: &mut HitTestResult, position: Offset) -> bool {
        let ret = self.with_widget(|w, _| w.hit_test(&self.render_object(), result, position));
        if ret {
            tracing::debug!("hit in {}, position: {:?}", self.name(), position);
        }
        ret
    }

    pub(crate) fn is_repaint_bondary(&self) -> bool {
        self.with_widget(|w, _| w.is_repaint_boundary())
    }

    pub(crate) fn paint_bounds(&self) -> Rect {
        Rect::from_size(self.size())
    }

    pub(crate) fn handle_event(&self, event: PointerEvent, entry: HitTestEntry) {
        tracing::debug!("handle event in {}, event: {:?}", self.name(), event);

        self.with_widget(|w, _| {
            w.handle_event(&self.render_object(), event, entry.to_box_hit_test_entry())
        })
    }

    pub(crate) fn layout_without_resize(&self) {
        assert_eq!(&self.relayout_boundary(), &self.render_object());
        assert!(!self.doing_this_layout_with_callback());
        self.perform_layout();
        self.set_needs_layout(false);
        self.mark_needs_paint();
    }

    pub(crate) fn mark_needs_paint(&self) {
        self._mark_needs_paint()
    }

    pub(crate) fn layout(&self, constraints: Constraints, parent_use_size: bool) {
        let is_relayout_boundary = !parent_use_size
            || self.sized_by_parent()
            || constraints.is_tight()
            || self.try_parent().is_none();
        let relayout_boundary = if is_relayout_boundary {
            self.render_object()
        } else {
            self.parent().relayout_boundary()
        };

        if !self.needs_layout() && Some(&constraints) == self.try_constraints().as_ref() {
            if Some(relayout_boundary.clone()) != self.try_relayout_boundary() {
                self.set_relayout_boundary(Some(relayout_boundary));
                self.visit_children(|e| e.propagate_relayout_bondary());
            }
            return;
        }

        self.set_constraints(constraints.into());
        if self.try_relayout_boundary().is_some() && self.relayout_boundary() != relayout_boundary {
            self.visit_children(|e| e.clean_relayout_boundary());
        }
        self.set_relayout_boundary(Some(relayout_boundary));
        assert!(!self.doing_this_layout_with_callback());

        tracing::debug!(
            "layout in {}: is_relayout_boundary: {}, constraints: {:?}",
            self.name(),
            is_relayout_boundary,
            self.box_constraints(),
        );

        self.perform_layout();
        self.set_needs_layout(false);
        self.mark_needs_paint();
    }

    pub(crate) fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4) {
        assert_eq!(child.parent(), self.render_object());

        let offset = child.render_box().offset();
        transform.translate(offset.dx, offset.dy);
    }

    pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
        self.set_needs_paint(false);
        self.paint(context, offset);
        assert!(!self.needs_layout());
        assert!(!self.needs_paint());
    }

    pub(crate) fn attach(&self, owner: PipelineOwner) {
        self._attach(owner)
    }

    pub(crate) fn detach(&self) {
        self._detach()
    }

    /// Mark the given node as being a child of this node.
    ///
    /// Subclasses should call this function when they acquire a new child.
    pub(crate) fn adopt_child(&self, child: &RenderObject) {
        self._adopt_child(child)
    }

    /// Disconnect the given node from this node.
    ///
    /// Subclasses should call this function when they lose a child.
    pub(crate) fn drop_child(&self, child: &RenderObject) {
        self._drop_child(child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let inner = RefCell::new(InnerRenderBox {
            object: None,
            ..Default::default()
        });

        let _r = Rc::new(inner);
        // let render_box = RenderBox2 { inner: r };
        print!("hello");
    }
}
