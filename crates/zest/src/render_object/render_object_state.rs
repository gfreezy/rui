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

pub(crate) struct RenderObjectState {
    this: Option<WeakRenderObject>,
    first_child: Option<RenderObject>,
    last_child: Option<WeakRenderObject>,
    next_sibling: Option<RenderObject>,
    prev_sibling: Option<WeakRenderObject>,
    child_count: usize,
    depth: usize,
    parent: Option<WeakRenderObject>,
    owner: Option<WeakOwner>,
    parent_data: Option<ParentData>,
    needs_layout: bool,
    needs_paint: bool,
    relayout_boundary: Option<WeakRenderObject>,
    doing_this_layout_with_callback: bool,
    constraints: Option<Constraints>,
    layer: Option<Layer>,
}

impl Default for RenderObjectState {
    fn default() -> Self {
        Self {
            this: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            child_count: 0,
            depth: 0,
            parent: None,
            owner: None,
            parent_data: None,
            needs_layout: true,
            needs_paint: true,
            relayout_boundary: None,
            doing_this_layout_with_callback: false,
            constraints: None,
            layer: None,
        }
    }
}

impl RenderObjectState {
    pub(crate) fn parent(&self) -> RenderObject {
        self.try_parent().unwrap()
    }

    pub(crate) fn try_parent(&self) -> Option<RenderObject> {
        Some(self.parent.as_ref()?.upgrade())
    }

    pub(crate) fn parent_data(&self) -> ParentData {
        self.try_parent_data().unwrap()
    }

    pub(crate) fn try_parent_data(&self) -> Option<ParentData> {
        self.parent_data.clone()
    }

    pub(crate) fn first_child(&self) -> RenderObject {
        self.try_first_child().unwrap()
    }

    pub(crate) fn try_first_child(&self) -> Option<RenderObject> {
        self.first_child.clone()
    }

    pub(crate) fn last_child(&self) -> RenderObject {
        self.try_last_child().unwrap()
    }

    pub(crate) fn try_last_child(&self) -> Option<RenderObject> {
        Some(self.last_child.as_ref()?.upgrade())
    }

    pub(crate) fn next_sibling(&self) -> RenderObject {
        self.try_next_sibling().unwrap()
    }

    pub(crate) fn prev_sibling(&self) -> RenderObject {
        self.try_prev_sibling().unwrap()
    }

    pub(crate) fn try_next_sibling(&self) -> Option<RenderObject> {
        self.next_sibling.clone()
    }

    pub(crate) fn try_prev_sibling(&self) -> Option<RenderObject> {
        Some(self.prev_sibling.as_ref()?.upgrade())
    }
    pub(crate) fn render_object(&self) -> RenderObject {
        self.this.as_ref().map(|this| this.upgrade()).unwrap()
    }

    // setters
    pub(crate) fn set_render_object(&mut self, obj: RenderObject) {
        self.this.replace(obj.downgrade());
    }

    pub(crate) fn set_parent(&mut self, element: Option<RenderObject>) {
        self.parent = element.map(|e| e.downgrade());
    }

    pub(crate) fn set_next_sibling(&mut self, element: Option<RenderObject>) {
        self.next_sibling = element;
    }

    pub(crate) fn set_prev_sibling(&mut self, element: Option<RenderObject>) {
        self.prev_sibling = element.map(|v| v.downgrade());
    }

    pub(crate) fn set_first_child(&mut self, element: Option<RenderObject>) {
        self.first_child = element;
    }

    pub(crate) fn set_last_child(&mut self, element: Option<RenderObject>) {
        self.last_child = element.map(|v| v.downgrade());
    }

    pub(crate) fn set_last_child_if_none(&mut self, element: Option<RenderObject>) {
        if self.last_child.is_none() {
            self.last_child = element.map(|v| v.downgrade());
        }
    }

    // attach/detach
    pub(crate) fn attach(&mut self, owner: PipelineOwner) {
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

    pub(crate) fn detach(&mut self) {
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
    pub(crate) fn adopt_child(&mut self, child: &RenderObject) {
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
    pub(crate) fn drop_child(&mut self, child: &RenderObject) {
        assert_eq!(&child.parent(), &self.render_object());
        child.clean_relayout_boundary();
        child.set_parent(None);
        // detach the child from the owner
        self.mark_needs_layout();
    }

    /// Adjust the [depth] of the given [child] to be greater than this node's own
    /// [depth].
    ///
    /// Only call this method from overrides of [redepthChildren].

    pub(crate) fn redepth_child(&self, child: &RenderObject) {
        if child.depth() <= self.depth {
            child.incr_depth();
            child.redepth_children();
        }
    }

    /// Insert child into this render object's child list after the given child.
    ///
    /// If `after` is null, then this inserts the child at the start of the list,
    /// and the child becomes the new [firstChild].
    pub(crate) fn insert(&mut self, child: RenderObject, after: Option<RenderObject>) {
        assert_ne!(&child, &self.render_object());
        assert_ne!(after.as_ref(), Some(&self.render_object()));
        assert_ne!(Some(&child), after.as_ref());
        assert_ne!(Some(&child), self.try_first_child().as_ref());
        assert_ne!(Some(&child), self.try_last_child().as_ref());
        self.adopt_child(&child);
        self.insert_into_child_list(child, after);
    }

    pub(crate) fn add(&mut self, child: RenderObject) {
        self.insert(child, self.try_last_child());
    }

    pub(crate) fn remove(&mut self, child: &RenderObject) {
        self.remove_from_child_list(child.clone());
        self.drop_child(child);
    }

    pub(crate) fn remove_all(&mut self) {
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

    pub(crate) fn move_(&mut self, child: RenderObject, after: Option<RenderObject>) {
        assert_ne!(&child, &self.render_object());
        assert_ne!(Some(&self.render_object()), after.as_ref());
        assert_ne!(Some(&child), after.as_ref());
        assert_eq!(&child.parent(), &self.render_object());
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
                assert_eq!(try_ultimate_prev_sibling(after.clone()), self.first_child());
                assert_eq!(try_ultimate_next_sibling(after.clone()), self.last_child());
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

    pub(crate) fn redepth_children(&self) {
        let mut child = self.try_first_child();
        while let Some(c) = child {
            c.redepth_child(&c);
            child = c.try_next_sibling();
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

    pub(crate) fn relayout_boundary(&self) -> RenderObject {
        self.try_relayout_boundary().unwrap()
    }

    pub(crate) fn try_relayout_boundary(&self) -> Option<RenderObject> {
        Some(self.relayout_boundary.as_ref()?.upgrade())
    }

    pub(crate) fn set_relayout_boundary(&mut self, relayout_boundary: Option<RenderObject>) {
        self.relayout_boundary = relayout_boundary.map(|r| r.downgrade());
    }

    pub(crate) fn clean_relayout_boundary(&mut self) {
        if self.try_relayout_boundary().as_ref() != Some(&self.render_object()) {
            self.set_relayout_boundary(None);
            self.visit_children(|e| e.clean_relayout_boundary());
        }
    }

    pub(crate) fn propagate_relayout_bondary(&mut self) {
        if self.try_relayout_boundary().as_ref() == Some(&self.render_object()) {
            return;
        }

        let parent_relayout_boundary = self.parent().relayout_boundary();
        if Some(&parent_relayout_boundary) != self.try_relayout_boundary().as_ref() {
            self.set_relayout_boundary(Some(parent_relayout_boundary));
            self.visit_children(|e| e.propagate_relayout_bondary());
        }
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
                if relayout_boundary != self.render_object() {
                    self.mark_parent_needs_layout();
                } else {
                    self.needs_layout = true;
                    if let Some(owner) = self.try_owner() {
                        owner.add_node_need_layout(self.render_object());
                        owner.request_visual_update();
                    }
                }
            }
        }
    }

    pub(crate) fn clear_needs_layout(&mut self) {
        self.needs_layout = false;
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

    pub(crate) fn needs_layout(&self) -> bool {
        self.needs_layout
    }

    pub(crate) fn needs_paint(&self) -> bool {
        self.needs_paint
    }

    pub(crate) fn clear_needs_paint(&mut self) {
        self.needs_paint = false;
    }

    pub(crate) fn mark_needs_paint(&mut self) {
        if self.needs_paint {
            return;
        }
        self.needs_paint = true;
        let is_repaint_boundary = true;
        if is_repaint_boundary {
            if let Some(owner) = self.try_owner() {
                owner.add_node_need_paint(self.render_object());
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

    pub(crate) fn try_constraints(&self) -> Option<Constraints> {
        self.constraints.clone()
    }

    pub(crate) fn constraints(&self) -> Constraints {
        self.try_constraints().unwrap()
    }

    pub(crate) fn invoke_layout_callback(&mut self, callback: impl FnOnce(&Constraints)) {
        assert!(!self.doing_this_layout_with_callback);
        self.doing_this_layout_with_callback = true;
        self.owner()
            .enable_mutations_to_dirty_subtrees(|| callback(&self.constraints()));
        self.doing_this_layout_with_callback = false;
    }

    pub(crate) fn doing_this_layout_with_callback(&self) -> bool {
        self.doing_this_layout_with_callback
    }

    pub(crate) fn try_layer(&self) -> Option<Layer> {
        self.layer.clone()
    }

    pub(crate) fn set_layer(&mut self, layer: Option<Layer>) {
        self.layer = layer;
    }

    pub(crate) fn depth(&self) -> usize {
        self.depth
    }

    pub(crate) fn child_count(&self) -> usize {
        self.child_count
    }

    pub(crate) fn incr_depth(&mut self) {
        self.depth += 1;
    }

    pub(crate) fn clear_child_count(&mut self) {
        self.child_count = 0;
    }

    pub(crate) fn incr_child_count(&mut self) {
        self.child_count += 1;
    }

    pub(crate) fn decr_child_count(&mut self) {
        self.child_count -= 1;
    }

    pub(crate) fn set_constraints(&mut self, c: Constraints) {
        self.constraints = Some(c);
    }
}
