use crate::render_object::render_object::{try_ultimate_next_sibling, try_ultimate_prev_sibling};

use super::{
    abstract_node::AbstractNode,
    layer::Layer,
    pipeline_owner::{PipelineOwner, WeakOwner},
    render_object::{
        Constraints, Matrix4, Offset, PaintContext, ParentData, Rect, RenderObject,
        WeakRenderObject,
    },
};

#[derive(Default)]
pub(crate) struct RenderObjectState {
    pub(crate) this: Option<WeakRenderObject>,
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
    pub(crate) constraints: Option<Constraints>,
    pub(crate) layer: Option<Layer>,
}

impl RenderObjectState {
    pub fn new(self_render_object: &RenderObject) -> Self {
        RenderObjectState {
            this: Some(self_render_object.downgrade()),
            needs_layout: true,
            needs_paint: true,
            ..Default::default()
        }
    }

    pub fn this(&self) -> RenderObject {
        self.this.as_ref().map(|this| this.upgrade()).unwrap()
    }

    pub fn set_this(&mut self, obj: RenderObject) {
        self.this.replace(obj.downgrade());
    }

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

    pub fn attach(&mut self, owner: PipelineOwner) {
        tracing::debug!("attach owner");
        self.set_owner(Some(owner.clone()));

        if self.needs_layout && self.try_relayout_boundary().is_some() {
            self.needs_layout = false;
            self.mark_needs_layout();
        }
        // _needsCompositingBitsUpdate
        if self.needs_paint && self.layer.is_some() {
            self.needs_paint = false;
            self.mark_needs_paint();
        }

        // attach children
        let mut child = self.try_first_child();
        while let Some(c) = child {
            tracing::debug!("attach child");
            c.attach(owner.clone());
            child = c.try_next_sibling();
        }
    }

    pub fn detach(&mut self) {
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
    pub fn adopt_child(&mut self, child: &RenderObject) {
        assert!(child.try_parent().is_none());
        child.set_parent(Some(child.clone()));
        self.mark_needs_layout();
        // self.mark_needs_composition_bits_update();

        // attach the child to the owner
        self.redepth_child(child);
    }

    /// Disconnect the given node from this node.
    ///
    /// Subclasses should call this function when they lose a child.
    pub fn drop_child(&mut self, child: &RenderObject) {
        assert_eq!(&child.parent(), &self.this());
        child.clean_relayout_boundary();
        child.set_parent(None);
        // detach the child from the owner
        self.mark_needs_layout();
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
    pub fn insert(&mut self, child: RenderObject, after: Option<RenderObject>) {
        assert_ne!(&child, &self.this());
        assert_ne!(after.as_ref(), Some(&self.this()));
        assert_ne!(Some(&child), after.as_ref());
        assert_ne!(Some(&child), self.try_first_child().as_ref());
        assert_ne!(Some(&child), self.try_last_child().as_ref());
        self.adopt_child(&child);
        self.insert_into_child_list(child, after);
    }

    pub fn add(&mut self, child: RenderObject) {
        self.insert(child, self.try_last_child());
    }

    pub fn remove(&mut self, child: &RenderObject) {
        self.remove_from_child_list(child.clone());
        self.drop_child(child);
    }

    pub fn remove_all(&mut self) {
        let mut child = self.try_first_child();
        while let Some(c) = child {
            c.set_prev_sibling(None);
            c.set_next_sibling(None);
            self.drop_child(&c);
            child = c.try_next_sibling();
        }
        self.set_first_child(None);
        self.set_last_child(None);
        self.child_count = 0;
    }

    pub fn move_(&mut self, child: RenderObject, after: Option<RenderObject>) {
        assert_ne!(&child, &self.this());
        assert_ne!(Some(&self.this()), after.as_ref());
        assert_ne!(Some(&child), after.as_ref());
        assert_eq!(&child.parent(), &self.this());
        if self.try_prev_sibling() == after {
            return;
        }
        self.remove_from_child_list(child.clone());
        self.insert_into_child_list(child, after);
        self.mark_needs_layout();
    }

    fn insert_into_child_list(&mut self, child: RenderObject, after: Option<RenderObject>) {
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
                assert_eq!(
                    try_ultimate_prev_sibling(after.clone()),
                    self.this().first_child()
                );
                assert_eq!(
                    try_ultimate_next_sibling(after.clone()),
                    self.this().last_child()
                );
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

    fn remove_from_child_list(&mut self, child: RenderObject) {
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

    pub(crate) fn mark_needs_layout(&mut self) {
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
                if relayout_boundary != self.this() {
                    self.mark_parent_needs_layout();
                } else {
                    self.needs_layout = true;
                    if let Some(owner) = self.try_owner() {
                        owner.add_node_need_layout(self.this());
                        owner.request_visual_update();
                    }
                }
            }
        }
    }

    pub(crate) fn mark_needs_layout_for_sized_by_parent_change(&mut self) {
        self.mark_needs_layout();
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

    pub(crate) fn owner(&self) -> PipelineOwner {
        self.try_owner().unwrap()
    }

    pub(crate) fn try_owner(&self) -> Option<PipelineOwner> {
        Some(self.owner.as_ref()?.upgrade())
    }

    pub(crate) fn set_owner(&mut self, owner: Option<PipelineOwner>) {
        self.owner = owner.map(|o| o.downgrade());
    }

    pub(crate) fn mark_needs_paint(&mut self) {
        if self.needs_paint {
            return;
        }
        self.needs_paint = true;
        let is_repaint_boundary = true;
        if is_repaint_boundary {
            if let Some(owner) = self.try_owner() {
                owner.add_node_need_paint(self.this());
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

    pub(crate) fn clean_relayout_boundary(&mut self) {
        if self.try_relayout_boundary().as_ref() != Some(&self.this()) {
            self.set_relayout_boundary(None);
            self.visit_children(|e| e.clean_relayout_boundary());
        }
    }

    pub(crate) fn propagate_relayout_bondary(&mut self) {
        if self.try_relayout_boundary().as_ref() == Some(&self.this()) {
            return;
        }

        let parent_relayout_boundary = self.parent().relayout_boundary();
        if Some(&parent_relayout_boundary) != self.try_relayout_boundary().as_ref() {
            self.set_relayout_boundary(Some(parent_relayout_boundary));
            self.visit_children(|e| e.propagate_relayout_bondary());
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
        false
    }

    fn constraints(&self) -> Constraints {
        self.constraints.clone().unwrap()
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
    fn paint(&self, _context: &mut PaintContext, _offset: Offset) {}

    pub(crate) fn get_transform_to(&self, ancestor: Option<RenderObject>) -> Matrix4 {
        let ancestor = match ancestor {
            Some(a) => a,
            None => self.owner().root_node(),
        };
        let mut renderers = vec![self.this()];
        let mut renderer = self.this();
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

    /// Returns a rect in this object's coordinate system that describes
    /// the approximate bounding box of the clip rect that would be
    /// applied to the given child during the paint phase, if any.
    ///
    /// Returns null if the child would not be clipped.
    ///
    /// This is used in the semantics phase to avoid including children
    /// that are not physically visible.
    fn describe_approximate_paint_clip(self, _child: RenderObject) -> Option<Rect> {
        None
    }
}
