use decorum::R64;
use std::cell::RefCell;

use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    rc::{Rc, Weak},
};

use crate::{arithmatic::Tolerance, render_object::render_object::RenderObject};

use super::{
    layer::Layer,
    pipeline_owner::{PipelineOwner, WeakOwner},
    render_object::{
        Constraints, HitTestEntry, Matrix4, Offset, PaintContext, ParentData, PointerEvent, Rect,
        WeakRenderObject,
    },
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
    pub const ZERO: Self = Size {
        width: 0.0,
        height: 0.0,
    };

    pub fn contains(&self, position: Offset) -> bool {
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
    fn name(&self) -> String;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn paint(&mut self, ctx: &RenderObject, paint_context: &mut PaintContext, offset: Offset) {
        let mut child = ctx.try_first_child();
        while let Some(c) = child {
            let offset_in_parent = c.render_box().offset();
            paint_context.paint_child(&c, offset_in_parent + offset);
            child = c.try_next_sibling();
        }
    }

    fn handle_event(&mut self, ctx: &RenderObject, event: PointerEvent, entry: BoxHitTestEntry) {}

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
            }
        }
        return false;
    }

    fn hit_test_self(&mut self, ctx: &RenderObject, position: Offset) -> bool {
        ctx.render_box().size().contains(position)
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

impl RenderBox {
    fn to_render_object(&self) -> RenderObject {
        RenderObject::RenderBox(self.clone())
    }

    pub(crate) fn new_render_object(widget: Box<dyn RenderBoxWidget>) -> RenderObject {
        let inner = RefCell::new(InnerRenderBox {
            object: Some(widget),
            ..Default::default()
        });
        let render_box = RenderBox {
            inner: Rc::new(inner),
        };

        let render_object = render_box.to_render_object();
        render_box.set_render_object(&render_object);
        render_object
    }

    pub fn downgrade(&self) -> WeakRenderBox {
        WeakRenderBox {
            inner: Rc::downgrade(&self.inner),
        }
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
            first_child: Default::default(),
            last_child: Default::default(),
            next_sibling: Default::default(),
            prev_sibling: Default::default(),
            child_count: Default::default(),
            depth: Default::default(),
            self_render_object: Default::default(),
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
    pub(crate) fn name(&self) -> String {
        self.with_widget(|w, _| w.name())
    }

    pub(crate) fn update<T: 'static>(&self, update: impl FnOnce(&mut T)) {
        self.with_widget(|w, _| update(w.as_any_mut().downcast_mut::<T>().unwrap()))
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
        tracing::debug!("painting in {}, offset: {:?}", self.name(), offset);
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
        let mut inner = self.inner.borrow_mut();
        inner.cached_instrinsic_dimensions.clear();
        inner.cached_dry_layout_sizes.clear();
        if inner.try_parent().is_some() {
            inner.mark_parent_needs_layout();
        } else {
            inner.mark_needs_layout();
        }
    }

    // private methods
    fn with_widget<T>(&self, f: impl FnOnce(&mut dyn RenderBoxWidget, &RenderObject) -> T) -> T {
        let mut widget = self.inner.borrow_mut().object.take().unwrap();
        let ret = f(&mut *widget, &self.to_render_object());
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
}

impl RenderBox {
    pub(crate) fn hit_test(&self, result: &mut HitTestResult, position: Offset) -> bool {
        tracing::debug!("hit test in {}, position: {:?}", self.name(), position);
        self.with_widget(|w, _| w.hit_test(&self.render_object(), result, position))
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
        assert_eq!(&self.relayout_boundary(), &self.to_render_object());
        assert!(!self.doing_this_layout_with_callback());
        self.perform_layout();
        self.clear_needs_layout();
        self.mark_needs_paint();
    }

    pub(crate) fn layout(&self, constraints: Constraints, parent_use_size: bool) {
        let is_relayout_boundary = !parent_use_size
            || self.sized_by_parent()
            || constraints.is_tight()
            || self.try_parent().is_none();
        let relayout_boundary = if is_relayout_boundary {
            self.to_render_object()
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

        self.perform_layout();
        self.clear_needs_layout();
        self.mark_needs_paint();

        tracing::debug!(
            "layout in {}: size: {:?}, is_relayout_boundary: {}, constraints: {:?}",
            self.name(),
            self.size(),
            is_relayout_boundary,
            self.box_constraints(),
        );
    }

    pub(crate) fn apply_paint_transform(&self, child: &RenderObject, transform: &Matrix4) {
        assert_eq!(child.parent(), self.to_render_object());

        let offset = child.render_box().offset();
        transform.translate(offset.dx, offset.dy);
    }
}

pub struct HitTestResult {
    entries: Vec<HitTestEntry>,
    local_transforms: Vec<Matrix4>,
    transforms: Vec<Matrix4>,
}

impl HitTestResult {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            local_transforms: vec![],
            transforms: vec![Matrix4::identity()],
        }
    }

    pub fn add(&mut self, entry: HitTestEntry) {
        self.entries.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn add_with_paint_offset(
        &mut self,
        offset: Offset,
        position: Offset,
        hit_test: impl FnOnce(&mut HitTestResult, Offset) -> bool,
    ) -> bool {
        let transformed = position - offset;
        if offset != Offset::ZERO {
            self.push_offset(-offset);
        }
        let hit = hit_test(self, transformed);
        if offset != Offset::ZERO {
            self.pop_transform();
        }
        hit
    }

    pub fn entries(&self) -> impl Iterator<Item = &HitTestEntry> {
        self.entries.iter()
    }

    fn push_offset(&mut self, offset: Offset) {
        assert_ne!(offset, Offset::ZERO);
        self.local_transforms
            .push(Matrix4::from_translation(offset.dx, offset.dx));
    }

    fn pop_transform(&mut self) {
        if self.local_transforms.pop().is_none() {
            self.transforms.pop();
        }
    }
}

#[derive(Clone)]
pub struct BoxHitTestEntry {
    render_object: WeakRenderObject,
    position: Offset,
}

impl BoxHitTestEntry {
    pub(crate) fn new(render_object: &RenderObject, position: Offset) -> Self {
        Self {
            render_object: render_object.downgrade(),
            position,
        }
    }

    pub fn target(&self) -> RenderObject {
        self.render_object.upgrade()
    }
}

impl From<BoxHitTestEntry> for HitTestEntry {
    fn from(entry: BoxHitTestEntry) -> Self {
        HitTestEntry::BoxHitTestEntry(entry)
    }
}

#[derive(Clone, Debug)]
pub struct BoxConstraints {
    pub min_width: f64,
    pub max_width: f64,
    pub min_height: f64,
    pub max_height: f64,
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

impl BoxConstraints {
    /// An unbounded box constraints object.
    ///
    /// Can be satisfied by any nonnegative size.
    pub const UNBOUNDED: BoxConstraints = BoxConstraints {
        min_width: 0.,
        min_height: 0.,
        max_width: f64::INFINITY,
        max_height: f64::INFINITY,
    };

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

    /// Create a "tight" box constraints object for one or more dimensions.
    ///
    /// [rounded away from zero]: struct.Size.html#method.expand
    pub fn tight_for(width: Option<f64>, height: Option<f64>) -> BoxConstraints {
        match (width, height) {
            (None, None) => BoxConstraints::UNBOUNDED,
            (None, Some(h)) => BoxConstraints {
                min_height: h,
                max_height: h,
                ..BoxConstraints::UNBOUNDED
            },
            (Some(w), None) => BoxConstraints {
                min_width: w,
                max_width: w,
                ..BoxConstraints::UNBOUNDED
            },
            (Some(w), Some(h)) => BoxConstraints {
                min_width: w,
                max_width: w,
                min_height: h,
                max_height: h,
            },
        }
    }

    pub(crate) fn constrain(&self, cross_size: Size) -> Size {
        Size::new(
            self.constrain_width(cross_size.width),
            self.constrain_height(cross_size.height),
        )
    }

    fn constrain_width(&self, width: f64) -> f64 {
        if width < self.min_width {
            self.min_width
        } else if width > self.max_width {
            self.max_width
        } else {
            width
        }
    }

    fn constrain_height(&self, height: f64) -> f64 {
        if height < self.min_height {
            self.min_height
        } else if height > self.max_height {
            self.max_height
        } else {
            height
        }
    }
}

impl RenderBox {
    pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
        self.clear_needs_paint();
        self.paint(context, offset);
        assert!(!self.needs_layout());
        assert!(!self.needs_paint());
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
            pub(crate) fn visit_children(&self, visitor: impl FnMut(RenderObject));

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

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;

    // #[mixin::insert(RenderObjectState)]
    // pub(crate) struct InnerRenderBox {
    //     object: Option<Box<dyn RenderBoxWidget + 'static>>,
    //     size: Option<Size>,
    //     offset: Offset,
    //     cached_instrinsic_dimensions: HashMap<InstrinsicDimensionsCacheEntry, f64>,
    //     cached_dry_layout_sizes: HashMap<BoxConstraints, Size>,
    // }

    // impl Default for InnerRenderBox {
    //     fn default() -> Self {
    //         Self {
    //             first_child: Default::default(),
    //             last_child: Default::default(),
    //             next_sibling: Default::default(),
    //             prev_sibling: Default::default(),
    //             child_count: Default::default(),
    //             depth: Default::default(),
    //             self_render_object: Default::default(),
    //             parent: Default::default(),
    //             owner: Default::default(),
    //             parent_data: Default::default(),
    //             needs_layout: true,
    //             needs_paint: true,
    //             relayout_boundary: Default::default(),
    //             doing_this_layout_with_callback: Default::default(),
    //             constraints: Default::default(),
    //             layer: Default::default(),
    //             object: Default::default(),
    //             size: Default::default(),
    //             offset: Default::default(),
    //             cached_instrinsic_dimensions: Default::default(),
    //             cached_dry_layout_sizes: Default::default(),
    //         }
    //     }
    // }

    pub struct RenderBox2 {
        pub(crate) inner: Rc<RefCell<InnerRenderBox>>,
    }

    #[test]
    fn test_name() {
        let inner = RefCell::new(InnerRenderBox {
            object: None,
            ..Default::default()
        });

        let r = Rc::new(inner);
        // let render_box = RenderBox2 { inner: r };
        print!("hello");
    }
}
