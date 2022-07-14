use super::{
    pipeline_owner::PipelineOwner, render_object::RenderObject,
    render_object_state::RenderObjectState,
};

pub(crate) trait AbstractNode {
    fn state<R>(&self, process: impl FnOnce(&mut RenderObjectState) -> R) -> R;

    fn parent(&self) -> RenderObject {
        self.state(|s| s.parent())
    }

    fn try_parent(&self) -> Option<RenderObject> {
        self.state(|s| s.try_parent())
    }

    fn first_child(&self) -> RenderObject {
        self.state(|s| s.first_child())
    }

    fn try_first_child(&self) -> Option<RenderObject> {
        self.state(|s| s.try_first_child())
    }

    fn last_child(&self) -> RenderObject {
        self.state(|s| s.last_child())
    }

    fn try_last_child(&self) -> Option<RenderObject> {
        self.state(|s| s.try_last_child())
    }

    fn next_sibling(&self) -> RenderObject {
        self.state(|s| s.next_sibling())
    }

    fn prev_sibling(&self) -> RenderObject {
        self.state(|s| s.prev_sibling())
    }

    fn set_parent(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_parent(element))
    }

    fn try_next_sibling(&self) -> Option<RenderObject> {
        self.state(|s| s.try_next_sibling())
    }

    fn try_prev_sibling(&self) -> Option<RenderObject> {
        self.state(|s| s.try_prev_sibling())
    }

    fn set_next_sibling(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_next_sibling(element))
    }

    fn set_prev_sibling(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_prev_sibling(element))
    }

    fn set_first_child(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_first_child(element))
    }

    fn set_last_child(&self, element: Option<RenderObject>) {
        self.state(|s| s.set_last_child(element))
    }

    fn set_last_child_if_none(&self, element: Option<RenderObject>) {
        self.state(|s| {
            if s.last_child.is_none() {
                s.last_child = element.map(|v| v.downgrade());
            }
        })
    }

    fn attach(&self, owner: PipelineOwner) {
        self.state(|s| s.attach(owner))
    }

    fn detach(&self) {
        self.state(|s| s.detach())
    }

    /// Mark the given node as being a child of this node.
    ///
    /// Subclasses should call this function when they acquire a new child.
    fn adopt_child(&self, child: &RenderObject) {
        self.state(|s| s.adopt_child(child))
    }

    /// Disconnect the given node from this node.
    ///
    /// Subclasses should call this function when they lose a child.
    fn drop_child(&self, child: &RenderObject) {
        self.state(|s| s.drop_child(child))
    }

    /// Adjust the [depth] of the given [child] to be greater than this node's own
    /// [depth].
    ///
    /// Only call this method from overrides of [redepthChildren].

    fn redepth_child(&self, child: &RenderObject) {
        self.state(|s| s.redepth_child(child))
    }

    /// Insert child into this render object's child list after the given child.
    ///
    /// If `after` is null, then this inserts the child at the start of the list,
    /// and the child becomes the new [firstChild].
    fn insert(&self, child: RenderObject, after: Option<RenderObject>) {
        self.state(|s| s.insert(child, after))
    }

    fn add(&self, child: RenderObject) {
        self.state(|s| s.add(child))
    }

    fn remove(&self, child: &RenderObject) {
        self.state(|s| s.remove(child))
    }

    fn remove_all(&self) {
        self.state(|s| s.remove_all())
    }

    fn move_(&self, child: RenderObject, after: Option<RenderObject>) {
        self.state(|s| s.move_(child, after))
    }

    fn depth(&self) -> usize {
        self.state(|s| s.depth)
    }

    fn incr_depth(&self) {
        self.state(|s| {
            s.depth += 1;
        })
    }

    fn child_count(&self) -> usize {
        self.state(|s| s.child_count)
    }

    fn clear_child_count(&self) {
        self.state(|s| s.child_count = 0)
    }

    fn incr_child_count(&self) {
        self.state(|s| {
            s.child_count += 1;
        })
    }

    fn decr_child_count(&self) {
        self.state(|s| {
            s.child_count -= 1;
        })
    }

    fn redepth_children(&self) {
        self.state(|s| s.redepth_children())
    }

    fn visit_children(&self, mut visitor: impl FnMut(RenderObject)) {
        // attach children
        let mut child = self.try_first_child();
        while let Some(c) = child {
            visitor(c.clone());
            child = c.try_next_sibling();
        }
    }
}
