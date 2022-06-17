use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
};

use decorum::{N64, R64};

use crate::widget::render_object::RenderObject;

use super::{
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
    const ZERO: Self = Size {
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BoxConstraints {}

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

pub(crate) struct RenderBox {
    pub(crate) state: RenderObjectState,
    size: Option<Size>,
    offset: Offset,
    constraints: Option<BoxConstraints>,
    cached_instrinsic_dimensions: RefCell<HashMap<InstrinsicDimensionsCacheEntry, f64>>,
    cached_dry_layout_sizes: RefCell<HashMap<BoxConstraints, Size>>,
}

impl RenderBox {
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
            let mut ref_mut = self.cached_instrinsic_dimensions.borrow_mut();
            let ret = ref_mut.entry(key).or_insert_with(|| computer(argument));
            *ret
        } else {
            computer(argument)
        }
    }

    fn get_min_instrinsic_width(&self, height: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MinWidth, height, |width| {
            self.compute_min_instrinsic_width(height)
        })
    }

    fn compute_min_instrinsic_width(&self, height: f64) -> f64 {
        0.0
    }

    fn get_max_instrinsic_width(&self, height: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MaxWidth, height, |width| {
            self.compute_max_instrinsic_width(height)
        })
    }

    fn compute_max_instrinsic_width(&self, height: f64) -> f64 {
        0.0
    }

    fn get_min_instrinsic_height(&self, width: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MinHeight, width, |width| {
            self.compute_min_instrinsic_height(width)
        })
    }

    fn compute_min_instrinsic_height(&self, width: f64) -> f64 {
        0.0
    }

    fn get_max_instrinsic_height(&self, width: f64) -> f64 {
        self.compute_intrinsic_dimensions(InstrinsicDimension::MaxHeight, width, |width| {
            self.compute_max_instrinsic_height(width)
        })
    }

    fn compute_max_instrinsic_height(&self, width: f64) -> f64 {
        0.0
    }

    fn constraints(&self, ctx: &RenderObject) -> BoxConstraints {
        todo!()
    }

    fn get_dry_layout(&self, constraints: BoxConstraints) -> Size {
        let should_cache = true;
        if should_cache {
            let mut ref_mut = self.cached_dry_layout_sizes.borrow_mut();
            ref_mut
                .entry(constraints.clone())
                .or_insert_with(|| self.compute_dry_layout(constraints))
                .clone()
        } else {
            self.compute_dry_layout(constraints)
        }
    }

    fn compute_dry_layout(&self, constraints: BoxConstraints) -> Size {
        Size::ZERO
    }

    pub fn has_size(&self) -> bool {
        self.size.is_some()
    }

    pub fn size(&self) -> Size {
        self.size.expect("no size available")
    }

    pub(crate) fn mark_needs_layout(&mut self, ctx: &RenderObject) {
        self.cached_instrinsic_dimensions.borrow_mut().clear();
        self.cached_dry_layout_sizes.borrow_mut().clear();
        if self.state.try_parent().is_some() {
            self.state.mark_parent_needs_layout();
        } else {
            self.state.mark_needs_layout(ctx);
        }
    }

    pub(crate) fn perform_resize(&mut self, ctx: &RenderObject) {
        self.size = Some(self.compute_dry_layout(self.constraints(ctx)));
    }

    pub(crate) fn perform_layout(&mut self, ctx: &RenderObject) {}

    pub(crate) fn hit_test(
        &self,
        ctx: &RenderObject,
        result: &mut BoxHitTestResult,
        position: Offset,
    ) -> bool {
        if self.size().contains(position) {
            if self.hit_test_children(ctx, result, position) || self.hit_test_self(ctx, position) {
                result.add(BoxHitTestEntry::new(ctx.downgrade(), position));
            }
        }
        return false;
    }

    pub(crate) fn hit_test_self(&self, ctx: &RenderObject, position: Offset) -> bool {
        false
    }

    pub(crate) fn hit_test_children(
        &self,
        ctx: &RenderObject,
        result: &mut BoxHitTestResult,
        position: Offset,
    ) -> bool {
        false
    }

    pub(crate) fn apply_paint_transform(
        &self,
        ctx: &RenderObject,
        child: &RenderObject,
        transform: &Matrix4,
    ) {
        assert_eq!(&child.parent(), ctx);

        let offset = child.box_ref().offset;
        transform.translate(offset.dx, offset.dy);
    }

    pub(crate) fn global_to_local(
        &self,
        ctx: &RenderObject,
        point: Offset,
        ancestor: Option<RenderObject>,
    ) -> Offset {
        let mut transform = self.state.get_transform_to(ctx, ancestor);
        let det = transform.invert();
        if det == 0.0 {
            return Offset::ZERO;
        }

        let n = Vector3::new(0.0, 0.0, 1.0);
        let i = transform.perspective_transform(Vector3::new(0.0, 0.0, 0.0));
        let d = transform.perspective_transform(Vector3::new(0.0, 0.0, 0.0));
        let s = transform.perspective_transform(Vector3::new(point.dx, point.dy, 0.0));
        let p = s - d * (n.dot(s) / n.dot(d));
        Offset::new(p.x, p.y)
    }

    pub(crate) fn local_to_global(
        &self,
        ctx: &RenderObject,
        point: Offset,
        ancestor: Option<RenderObject>,
    ) -> Offset {
        todo!()
    }

    pub(crate) fn paint_bounds(&self, ctx: &RenderObject) -> Rect {
        todo!()
        // Offset::ZERO & self.size()
    }

    pub(crate) fn handle_event(
        &self,
        ctx: &RenderObject,
        event: PointerEvent,
        entry: BoxHitTestEntry,
    ) {
        todo!()
    }

    pub(crate) fn default_hit_test_children(
        &self,
        ctx: &RenderObject,
        result: &mut BoxHitTestResult,
        position: Offset,
    ) -> bool {
        let mut child = self.state.try_last_child();
        while let Some(c) = child {
            let offset = c.box_ref().offset;
            let is_hit = result.add_with_paint_offset(offset, position, |result, transformed| {
                assert_eq!(transformed, position - offset);
                c.box_ref().hit_test(ctx, result, transformed)
            });
            if is_hit {
                return true;
            }
            child = c.try_prev_sibling();
        }
        false
    }

    pub(crate) fn default_paint(&self, paint_context: PaintContext, offset: Offset) {
        let mut child = self.state.try_first_child();
        while let Some(c) = child {
            let offset_in_parent = c.box_ref().offset;
            paint_context.paint_child(&c, offset_in_parent + offset);
            child = c.try_next_sibling();
        }
    }
}

pub(crate) struct BoxHitTestResult {
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

    fn add_with_paint_offset(
        &self,
        offset: Offset,
        position: Offset,
        hit_test: impl FnOnce(&mut BoxHitTestResult, Offset) -> bool,
    ) -> bool {
        todo!()
    }
}

pub(crate) struct BoxHitTestEntry {
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
