//! Rendering backend for the DOM.

use std::cell::Cell;
use std::fmt;
use std::hash::{Hash, Hasher};

use sycamore_core::generic_node::{GenericNode, SycamoreElement};
use sycamore_core::render::insert;
use sycamore_core::view::View;
use sycamore_reactive::*;

use zest::render_object::render_object::RenderObject as Node;

/// Rendering backend for the DOM.
///
/// _This API requires the following crate features to be activated: `dom`_
#[derive(Clone)]
pub struct ZestNode {
    node: Node,
}

impl ZestNode {
    pub fn new(node: Node) -> Self {
        Self { node }
    }

    /// Get the underlying [`web_sys::Node`].
    pub fn inner_element(&self) -> Node {
        self.node.clone()
    }
}

impl PartialEq for ZestNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Eq for ZestNode {}

impl Hash for ZestNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner_element().id().hash(state);
    }
}

impl fmt::Debug for ZestNode {
    /// Prints outerHtml of [`Node`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner_element())
    }
}

impl GenericNode for ZestNode {
    type EventType = web_sys::Event;
    type PropertyType = String;

    fn element<T: SycamoreElement>() -> Self {
        Self::element_from_tag(T::TAG_NAME)
    }

    fn element_from_tag(tag: &str) -> Self {
        let node = todo!();
        ZestNode { node }
    }

    fn text_node(text: &str) -> Self {
        let node = todo!();
        ZestNode { node }
    }

    fn text_node_int(int: i32) -> Self {
        todo!()
    }

    fn marker_with_text(text: &str) -> Self {
        todo!()
    }

    fn set_attribute(&self, name: &str, value: &str) {}

    fn remove_attribute(&self, name: &str) {}

    fn set_class_name(&self, value: &str) {}

    fn add_class(&self, class: &str) {}

    fn remove_class(&self, class: &str) {}

    fn set_property(&self, name: &str, value: &Self::PropertyType) {}

    fn remove_property(&self, name: &str) {}

    fn append_child(&self, child: &Self) {
        self.node.add(&child.node);
    }

    fn first_child(&self) -> Option<Self> {
        self.node.try_first_child().map(ZestNode::new)
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        self.node.insert(
            new_node.node.clone(),
            reference_node.and_then(|n| n.node.try_prev_sibling()),
        );
    }

    fn remove_child(&self, child: &Self) {
        self.node.remove(&child.inner_element());
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        self.insert_child_before(new.node.clone(), &old.node);
        self.remove_child(&old);
    }

    fn insert_sibling_before(&self, child: &Self) {
        self.parent_node()
            .map(|p| p.insert_child_before(child, Some(self)));
    }

    fn parent_node(&self) -> Option<Self> {
        self.node.try_parent().map(ZestNode::new)
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.try_next_sibling().map(ZestNode::new)
    }

    fn remove_self(&self) {
        self.parent_node().map(|parent| parent.remove_child(self));
    }

    fn event<'a, F: FnMut(Self::EventType) + 'a>(&self, cx: Scope<'a>, name: &str, handler: F) {
        todo!()
    }

    fn update_inner_text(&self, text: &str) {
        todo!()
    }

    fn dangerously_set_inner_html(&self, html: &str) {
        unimplemented!()
    }

    fn clone_node(&self) -> Self {
        self.clone()
    }
}

/// Render a [`View`] into the DOM.
/// Alias for [`render_to`] with `parent` being the `<body>` tag.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render(view: impl FnOnce(Scope<'_>) -> View<ZestNode>) {
    render_to(view, Node);
}

/// Render a [`View`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render_to(view: impl FnOnce(Scope<'_>) -> View<ZestNode>, parent: &Node) {
    // Do not call the destructor function, effectively leaking the scope.
    let _ = render_get_scope(view, parent);
}

/// Render a [`View`] under a `parent` node, in a way that can be cleaned up.
/// This function is intended to be used for injecting an ephemeral sycamore view into a
/// non-sycamore app (for example, a file upload modal where you want to cancel the upload if the
/// modal is closed).
///
/// It is, however, preferable to have a single call to [`render`] or [`render_to`] at the top level
/// of your app long-term. For rendering a view that will never be unmounted from the dom, use
/// [`render_to`] instead. For rendering under the `<body>` tag, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
#[must_use = "please hold onto the ScopeDisposer until you want to clean things up, or use render_to() instead"]
pub fn render_get_scope<'a>(
    view: impl FnOnce(Scope<'_>) -> View<ZestNode> + 'a,
    parent: &'a Node,
) -> ScopeDisposer<'a> {
    create_scope(|cx| {
        insert(cx, &parent.clone(), view(cx), None, None, false);
    })
}
