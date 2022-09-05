//! Rendering backend for the Zest.

use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};

use sycamore_core::generic_node::{GenericNode, SycamoreElement};
use sycamore_core::render::insert;
use sycamore_core::view::View;
use sycamore_reactive::*;

use zest::render_object::render_object::{RenderObject, WeakRenderObject};
use zest::rendering::render_flex::RenderFlex;
use zest::rendering::render_pointer_listener::RenderPointerListener;
use zest::rendering::render_text::RenderText;

pub use zest::run;

#[inline]
pub fn intern(s: &str) -> &str {
    s
}

/// Rendering backend for the zest.
///
/// _This API requires the following crate features to be activated: `dom`_
#[derive(Clone)]
pub struct ZestNode {
    weak_node: WeakRenderObject,
    node: Option<RenderObject>,
}

impl Debug for ZestNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ZestNode")
            .field("render_object", &self.weak_node.upgrade())
            .finish()
    }
}

impl ZestNode {
    pub fn new(node: RenderObject) -> Self {
        Self {
            weak_node: node.downgrade(),
            node: Some(node),
        }
    }

    pub fn from_tag(name: &str) -> Self {
        eprintln!("from tag: {}", name);
        let node = match name {
            "flex" => {
                RenderObject::new_render_box("sycamore flex".to_string(), RenderFlex::default())
            }
            "listener" => RenderObject::new_render_box(
                "sycamore listener".to_string(),
                RenderPointerListener::default(),
            ),
            "text" => {
                RenderObject::new_render_box("sycamore text".to_string(), RenderText::default())
            }
            _ => todo!(),
        };
        ZestNode::new(node)
    }

    pub fn from_text(text: &str) -> Self {
        eprintln!("from text: {}", text);
        let node = RenderObject::new_render_box(
            format!("text: {{text}}"),
            RenderText::new(text.to_string(), 16.0, None),
        );
        ZestNode::new(node)
    }

    /// Get the underlying [`web_sys::Node`].
    pub fn node(&self) -> RenderObject {
        self.weak_node.upgrade()
    }
}

impl PartialEq for ZestNode {
    fn eq(&self, other: &Self) -> bool {
        self.weak_node.upgrade() == other.weak_node.upgrade()
    }
}

impl Eq for ZestNode {}

impl Hash for ZestNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node().id().hash(state);
    }
}

impl GenericNode for ZestNode {
    type EventType = zest::pointer_event::PointerEvent;
    type PropertyType = ();

    fn element<T: SycamoreElement>() -> Self {
        eprintln!("element: {}", T::TAG_NAME);
        Self::element_from_tag(T::TAG_NAME)
    }

    fn element_from_tag(tag: &str) -> Self {
        eprintln!("element_from_tag: {}", tag);
        Self::from_tag(tag)
    }

    fn text_node(text: &str) -> Self {
        todo!()
    }

    fn text_node_int(int: i32) -> Self {
        todo!()
    }

    fn marker_with_text(text: &str) -> Self {
        todo!()
    }

    fn set_attribute(&self, name: &str, value: &str) {
        eprintln!("set_attribute: {} {}", name, value);
        self.node().set_attribute(name, value);
    }

    fn remove_attribute(&self, name: &str) {}

    fn set_class_name(&self, value: &str) {}

    fn add_class(&self, class: &str) {}

    fn remove_class(&self, class: &str) {}

    fn set_property(&self, name: &str, value: &Self::PropertyType) {
        eprintln!("set_property: {} {:?}", name, value);
    }

    fn remove_property(&self, name: &str) {}

    fn append_child(&self, child: &Self) {
        eprintln!("append child");
        self.node().add(child.node());
    }

    fn first_child(&self) -> Option<Self> {
        self.node().try_first_child().map(ZestNode::new)
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        eprintln!(
            "insert child before, self: {:?} new_node: {:?}, reference_node: {:?}",
            self, new_node, reference_node
        );
        self.node().insert(
            new_node.node(),
            reference_node.and_then(|n| dbg!(n.node().try_prev_sibling())),
        );
    }

    fn remove_child(&self, child: &Self) {
        eprintln!("remove child");
        self.node().remove(&child.node());
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        eprintln!("replace child");
        self.node()
            .insert(new.node(), old.node().try_prev_sibling());
        self.remove_child(&old);
    }

    fn insert_sibling_before(&self, child: &Self) {
        eprintln!("insert_sibling_before");
        self.parent_node()
            .map(|p| p.insert_child_before(child, Some(self)));
        assert!(child.node().try_owner().is_some());
    }

    fn parent_node(&self) -> Option<Self> {
        self.node().try_parent().map(ZestNode::new)
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node().try_next_sibling().map(ZestNode::new)
    }

    fn remove_self(&self) {
        eprintln!("remove_self");
        self.parent_node().map(|parent| parent.remove_child(self));
    }

    fn event<'a, F: FnMut(Self::EventType) + 'a>(&self, cx: Scope<'a>, name: &str, handler: F) {
        let boxed: Box<dyn FnMut(Self::EventType)> = Box::new(handler);
        // SAFETY: extend lifetime because the closure is dropped when the cx is disposed,
        // preventing the handler from ever being accessed after its lifetime.
        let mut handler: Box<dyn FnMut(Self::EventType) + 'static> =
            unsafe { std::mem::transmute(boxed) };
        self.node()
            .update::<RenderPointerListener>(move |render_listener| {
                render_listener.on_pointer_up = Box::new(move |_, e| handler(e))
            });
    }

    fn update_inner_text(&self, text: &str) {
        eprintln!("update_inner_text: {:?}", self);
        let mut next_child = self.node().try_first_child();
        while let Some(child) = next_child {
            self.node().remove(&child);
            next_child = child.try_next_sibling();
        }
    }

    fn dangerously_set_inner_html(&self, html: &str) {
        unimplemented!()
    }

    fn clone_node(&self) -> Self {
        self.clone()
    }
}

/// Render a [`View`] under a `parent` node.
/// For rendering under the `<body>` tag, use [`render`] instead.
///
/// _This API requires the following crate features to be activated: `dom`_
pub fn render_to(view: impl FnOnce(Scope<'_>) -> View<ZestNode>, parent: &RenderObject) {
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
    parent: &'a RenderObject,
) -> ScopeDisposer<'a> {
    create_scope(|cx| {
        insert(
            cx,
            &ZestNode::new(parent.clone()),
            view(cx),
            None,
            None,
            false,
        );
    })
}
