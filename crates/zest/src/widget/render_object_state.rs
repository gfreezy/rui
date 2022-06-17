use crate::widget::render_object::{try_ultimate_next_sibling, try_ultimate_prev_sibling};

use super::render_object::{
    Constraints, Matrix4, Offset, Owner, PaintContext, ParentData, PointerEvent, Rect,
    RenderObject, WeakOwner, WeakRenderObject,
};

pub(crate) struct RenderObjectState {
    pub(crate) first_child: Option<RenderObject>,
    pub(crate) last_child: Option<WeakRenderObject>,
    pub(crate) next_sibling: Option<RenderObject>,
    pub(crate) prev_sibling: Option<WeakRenderObject>,
    pub(crate) child_count: usize,
    pub(crate) depth: usize,
    pub(crate) parent: Option<WeakRenderObject>,
    pub(crate) owner: Option<WeakOwner>,
    pub(crate) parent_data: Option<ParentData>,
    pub(crate) needs_layout: bool,
    pub(crate) needs_paint: bool,
    pub(crate) relayout_boundary: Option<WeakRenderObject>,
    pub(crate) doing_this_layout_with_callback: bool,
    pub(crate) constraints: Constraints,
}

impl RenderObjectState {
    pub fn parent(&self) -> RenderObject {
        self.try_parent().unwrap()
    }

    pub fn try_parent(&self) -> Option<RenderObject> {
        Some(self.parent.as_ref()?.upgrade())
    }

    pub fn parent_data(&self) -> ParentData {
        self.try_parent_data().unwrap()
    }

    pub fn try_parent_data(&self) -> Option<ParentData> {
        self.parent_data.clone()
    }

    pub fn first_child(&self) -> RenderObject {
        self.try_first_child().unwrap()
    }

    pub fn try_first_child(&self) -> Option<RenderObject> {
        self.first_child.clone()
    }

    pub fn last_child(&self) -> RenderObject {
        self.try_last_child().unwrap()
    }

    pub fn try_last_child(&self) -> Option<RenderObject> {
        Some(self.last_child.as_ref()?.upgrade())
    }

    pub fn next_sibling(&self) -> RenderObject {
        self.try_next_sibling().unwrap()
    }

    pub fn prev_sibling(&self) -> RenderObject {
        self.try_prev_sibling().unwrap()
    }

    pub fn set_parent(&mut self, element: Option<RenderObject>) {
        self.parent = element.map(|e| e.downgrade());
    }

    pub fn try_next_sibling(&self) -> Option<RenderObject> {
        self.next_sibling.clone()
    }

    pub fn try_prev_sibling(&self) -> Option<RenderObject> {
        Some(self.prev_sibling.as_ref()?.upgrade())
    }

    pub fn set_next_sibling(&mut self, element: Option<RenderObject>) {
        self.next_sibling = element;
    }

    pub fn set_prev_sibling(&mut self, element: Option<RenderObject>) {
        self.prev_sibling = element.map(|v| v.downgrade());
    }

    pub fn set_first_child(&mut self, element: Option<RenderObject>) {
        self.first_child = element;
    }

    pub fn set_last_child(&mut self, element: Option<RenderObject>) {
        self.last_child = element.map(|v| v.downgrade());
    }

    pub fn set_last_child_if_none(&mut self, element: Option<RenderObject>) {
        if self.last_child.is_none() {
            self.last_child = element.map(|v| v.downgrade());
        }
    }

    pub fn attach(&mut self, ctx: &RenderObject, owner: Owner) {
        self.set_owner(Some(owner.clone()));

        if self.needs_layout && self.try_relayout_boundary().is_some() {
            self.needs_layout = false;
            self.mark_needs_layout(ctx);
        }
        // _needsCompositingBitsUpdate
        if self.needs_paint {
            self.needs_paint = false;
            self.mark_needs_paint(ctx);
        }

        // attach children
        let mut child = self.try_first_child();
        while let Some(c) = child {
            c.attach(owner.clone());
            child = c.try_next_sibling();
        }
    }

    pub fn detach(&mut self, ctx: &RenderObject) {
        assert!(self.try_prev_sibling().is_none());
        assert!(self.try_next_sibling().is_none());
        self.set_owner(None);

        // attach children
        let mut child = self.try_first_child();
        while let Some(c) = child {
            c.depth();
            child = c.try_next_sibling();
        }
    }

    /// Mark the given node as being a child of this node.
    ///
    /// Subclasses should call this function when they acquire a new child.
    pub fn adopt_child(&mut self, ctx: &RenderObject, child: &RenderObject) {
        assert!(child.try_parent().is_none());
        child.set_parent(Some(child.clone()));
        self.mark_needs_layout(ctx);
        // self.mark_needs_composition_bits_update();

        // attach the child to the owner
        self.redepth_child(child);
    }

    /// Disconnect the given node from this node.
    ///
    /// Subclasses should call this function when they lose a child.
    pub fn drop_child(&mut self, ctx: &RenderObject, child: &RenderObject) {
        assert_eq!(&child.parent(), ctx);
        child.clean_relayout_boundary();
        child.set_parent(None);
        // detach the child from the owner
        self.mark_needs_layout(ctx);
    }

    /// Adjust the [depth] of the given [child] to be greater than this node's own
    /// [depth].
    ///
    /// Only call this method from overrides of [redepthChildren].

    pub fn redepth_child(&self, child: &RenderObject) {
        if child.depth() <= self.depth {
            child.incr_depth();
            child.redepth_children();
        }
    }

    /// Insert child into this render object's child list after the given child.
    ///
    /// If `after` is null, then this inserts the child at the start of the list,
    /// and the child becomes the new [firstChild].
    pub fn insert(&mut self, ctx: &RenderObject, child: RenderObject, after: Option<RenderObject>) {
        assert_ne!(&child, ctx);
        assert_ne!(after.as_ref(), Some(ctx));
        assert_ne!(Some(&child), after.as_ref());
        assert_ne!(Some(&child), self.try_first_child().as_ref());
        assert_ne!(Some(&child), self.try_last_child().as_ref());
        self.adopt_child(ctx, &child);
        self.insert_into_child_list(ctx, child, after);
    }

    pub fn add(&mut self, ctx: &RenderObject, child: RenderObject) {
        self.insert(ctx, child, self.try_last_child());
    }

    pub fn remove(&mut self, ctx: &RenderObject, child: &RenderObject) {
        self.remove_from_child_list(ctx, child.clone());
        self.drop_child(ctx, child);
    }

    pub fn remove_all(&mut self, ctx: &RenderObject) {
        let mut child = self.try_first_child();
        while let Some(c) = child {
            c.set_prev_sibling(None);
            c.set_next_sibling(None);
            self.drop_child(ctx, &c);
            child = c.try_next_sibling();
        }
        self.set_first_child(None);
        self.set_last_child(None);
        self.child_count = 0;
    }

    pub fn move_(&mut self, ctx: &RenderObject, child: RenderObject, after: Option<RenderObject>) {
        assert_ne!(&child, ctx);
        assert_ne!(Some(ctx), after.as_ref());
        assert_ne!(Some(&child), after.as_ref());
        assert_eq!(&child.parent(), ctx);
        if self.try_prev_sibling() == after {
            return;
        }
        self.remove_from_child_list(ctx, child.clone());
        self.insert_into_child_list(ctx, child, after);
        self.mark_needs_layout(ctx);
    }

    fn insert_into_child_list(
        &mut self,
        ctx: &RenderObject,
        child: RenderObject,
        after: Option<RenderObject>,
    ) {
        assert!(self.try_next_sibling().is_none());
        assert!(self.try_prev_sibling().is_none());
        self.child_count += 1;
        assert!(self.child_count > 0);
        match after {
            None => {
                let first_child = self.try_first_child();
                self.set_next_sibling(first_child.clone());
                if first_child.is_some() {
                    self.first_child().set_prev_sibling(Some(child.clone()));
                }
                self.set_first_child(Some(child.clone()));
                self.set_last_child_if_none(Some(child));
            }
            Some(after) => {
                assert!(self.try_first_child().is_some());
                assert!(self.try_last_child().is_some());
                assert_eq!(try_ultimate_prev_sibling(after.clone()), ctx.first_child());
                assert_eq!(try_ultimate_next_sibling(after.clone()), ctx.last_child());
                match after.try_next_sibling() {
                    None => {
                        assert_eq!(after, self.last_child());
                        child.set_prev_sibling(Some(after.clone()));
                        after.set_next_sibling(Some(child.clone()));
                        self.set_last_child(Some(child));
                    }
                    Some(next_sibling) => {
                        child.set_next_sibling(Some(next_sibling));
                        child.set_prev_sibling(Some(after.clone()));
                        child.prev_sibling().set_next_sibling(Some(child.clone()));
                        child.next_sibling().set_prev_sibling(Some(child.clone()));
                        assert_eq!(after.next_sibling(), child);
                    }
                }
            }
        }
    }

    fn remove_from_child_list(&mut self, _ctx: &RenderObject, child: RenderObject) {
        assert_eq!(try_ultimate_prev_sibling(child.clone()), self.first_child());
        assert_eq!(try_ultimate_next_sibling(child.clone()), self.last_child());
        assert!(self.child_count > 0);

        match child.try_prev_sibling() {
            None => {
                assert_eq!(self.first_child(), child);
                self.set_first_child(child.try_next_sibling());
            }
            Some(prev_sibling) => {
                prev_sibling.set_next_sibling(child.try_next_sibling());
            }
        }

        match child.try_next_sibling() {
            None => {
                assert_eq!(self.last_child(), child);
                self.set_last_child(child.try_prev_sibling());
            }
            Some(next_sibling) => {
                next_sibling.set_prev_sibling(child.try_prev_sibling());
            }
        }
        child.set_prev_sibling(None);
        child.set_next_sibling(None);
        child.decr_child_count();
    }

    pub(crate) fn mark_needs_layout(&mut self, ctx: &RenderObject) {
        if self.needs_layout {
            return;
        }
        match self.try_relayout_boundary() {
            None => {
                self.needs_layout = true;
                if self.try_parent().is_some() {
                    // _relayoutBoundary is cleaned by an ancestor in RenderObject.layout.
                    // Conservatively mark everything dirty until it reaches the closest
                    // known relayout boundary.
                    self.mark_parent_needs_layout();
                }
                return;
            }
            Some(relayout_boundary) => {
                if &relayout_boundary != ctx {
                    self.mark_parent_needs_layout();
                } else {
                    self.needs_layout = true;
                    if let Some(owner) = self.try_owner() {
                        owner.add_node_need_layout(ctx.clone());
                        owner.request_visual_update();
                    }
                }
            }
        }
    }

    pub(crate) fn mark_needs_layout_for_sized_by_parent_change(&mut self, ctx: &RenderObject) {
        self.mark_needs_layout(ctx);
        self.mark_parent_needs_layout();
    }

    pub(crate) fn redepth_children(&self) {
        let mut child = self.try_first_child();
        while let Some(c) = child {
            c.redepth_child(&c);
            child = c.try_next_sibling();
        }
    }

    pub fn relayout_boundary(&self) -> RenderObject {
        self.try_relayout_boundary().unwrap()
    }

    pub fn try_relayout_boundary(&self) -> Option<RenderObject> {
        Some(self.relayout_boundary.as_ref()?.upgrade())
    }

    pub fn set_relayout_boundary(&mut self, relayout_boundary: Option<RenderObject>) {
        self.relayout_boundary = relayout_boundary.map(|r| r.downgrade());
    }

    pub(crate) fn mark_parent_needs_layout(&mut self) {
        self.needs_layout = true;
        assert!(self.try_parent().is_some());
        let parent = self.parent();
        parent.mark_needs_layout();
        assert_eq!(parent, self.parent())
    }

    pub(crate) fn owner(&self) -> Owner {
        self.try_owner().unwrap()
    }

    pub(crate) fn try_owner(&self) -> Option<Owner> {
        Some(self.owner.as_ref()?.upgrade())
    }

    pub(crate) fn set_owner(&mut self, owner: Option<Owner>) {
        self.owner = owner.map(|o| o.downgrade());
    }

    pub(crate) fn mark_needs_paint(&mut self, ctx: &RenderObject) {
        if self.needs_paint {
            return;
        }
        self.needs_paint = true;
        let is_repaint_boundary = true;
        if is_repaint_boundary {
            if let Some(owner) = self.try_owner() {
                owner.add_node_need_paint(ctx.clone());
                owner.request_visual_update();
            }
        } else if self.try_parent().is_some() {
            self.parent().mark_needs_paint();
        } else {
            if let Some(owner) = self.try_owner() {
                owner.request_visual_update();
            }
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(RenderObject)) {
        // attach children
        let mut child = self.try_first_child();
        while let Some(c) = child {
            visitor(c.clone());
            child = c.try_next_sibling();
        }
    }

    pub(crate) fn clean_relayout_boundary(&mut self, ctx: &RenderObject) {
        if self.try_relayout_boundary().as_ref() != Some(ctx) {
            self.set_relayout_boundary(None);
            self.visit_children(|e| e.clean_relayout_boundary());
        }
    }

    pub(crate) fn propagate_relayout_bondary(&mut self, ctx: &RenderObject) {
        if self.try_relayout_boundary().as_ref() == Some(ctx) {
            return;
        }

        let parent_relayout_boundary = self.parent().relayout_boundary();
        if Some(&parent_relayout_boundary) != self.try_relayout_boundary().as_ref() {
            self.set_relayout_boundary(Some(parent_relayout_boundary));
            self.visit_children(|e| e.propagate_relayout_bondary());
        }
    }

    fn layout_without_resize(&mut self, ctx: &RenderObject) {
        assert_eq!(&self.relayout_boundary(), ctx);
        assert!(!self.doing_this_layout_with_callback);
        self.perform_layout();
        self.needs_layout = false;
        self.mark_needs_paint(ctx);
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
    fn layout(&mut self, ctx: &RenderObject, constraints: Constraints, parent_use_size: bool) {
        let is_relayout_boundary = !parent_use_size
            || self.sized_by_parent()
            || constraints.is_tight()
            || self.try_parent().is_none();
        let relayout_boundary = if is_relayout_boundary {
            ctx.clone()
        } else {
            self.parent().relayout_boundary()
        };
        if !self.needs_layout
            && constraints == self.constraints()
            && Some(relayout_boundary.clone()) != self.try_relayout_boundary()
        {
            self.set_relayout_boundary(Some(relayout_boundary));
            self.visit_children(|e| e.propagate_relayout_bondary());
            return;
        }

        self.constraints = constraints;
        if self.try_relayout_boundary().is_some() && self.relayout_boundary() != relayout_boundary {
            self.visit_children(|e| e.clean_relayout_boundary());
        }
        self.set_relayout_boundary(Some(relayout_boundary));
        assert!(!self.doing_this_layout_with_callback);

        if self.sized_by_parent() {
            self.perform_resize();
        }

        self.perform_layout();
        self.needs_layout = false;
        self.mark_needs_paint(ctx);
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

    fn constraints(&self) -> Constraints {
        self.constraints.clone()
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

    pub(crate) fn invoke_layout_callback(&mut self, callback: impl FnOnce(&Constraints)) {
        assert!(!self.doing_this_layout_with_callback);
        self.doing_this_layout_with_callback = true;
        self.owner()
            .enable_mutations_to_dirty_subtrees(|| callback(&self.constraints()));
        self.doing_this_layout_with_callback = false;
    }

    fn paint_with_context(&mut self, context: &mut PaintContext, offset: Offset) {
        self.needs_paint = false;
        self.paint(context, offset);
        assert!(!self.needs_layout);
        assert!(!self.needs_paint);
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

    pub(crate) fn get_transform_to(
        &self,
        ctx: &RenderObject,
        ancestor: Option<RenderObject>,
    ) -> Matrix4 {
        let ancestor = match ancestor {
            Some(a) => a,
            None => self.owner().root_node(),
        };
        let mut renderers = vec![ctx.clone()];
        let mut renderer = ctx.clone();
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
            renderer.apply_paint_transform((*next).clone(), &mut transform);
        }
        transform
    }

    /// Returns a rect in this object's coordinate system that describes
    /// the approximate bounding box of the clip rect that would be
    /// applied to the given child during the paint phase, if any.
    ///
    /// Returns null if the child would not be clipped.
    ///
    /// This is used in the semantics phase to avoid including children
    /// that are not physically visible.
    fn describe_approximate_paint_clip(self, child: RenderObject) -> Option<Rect> {
        None
    }
}
