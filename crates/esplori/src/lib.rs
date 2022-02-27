#![allow(dead_code)]

mod object;
use generational_indextree::{Arena, NodeId};
use object::{Ctx, Properties, Widget, WidgetInterface};
use std::any::Any;

type Key = usize;
pub struct UiState {
    pub(crate) id: usize,
}

impl UiState {
    pub fn new() -> Self {
        UiState { id: 0 }
    }
}

pub struct RenderObject<T: WidgetInterface + 'static> {
    pub(crate) object: T,
    pub(crate) state: UiState,
}

pub struct StateObject<T> {
    pub(crate) object: T,
}

struct Element {
    pub(crate) key: Key,
    pub(crate) element_object: Box<dyn Any>,
}

impl Element {
    pub fn from_widget<T: WidgetInterface + 'static>(key: Key, widget: T) -> Self {
        Element {
            key,
            element_object: Box::new(RenderObject {
                object: widget,
                state: UiState::new(),
            }),
        }
    }
    pub fn from_state<T: 'static>(key: Key, widget: T) -> Self {
        Element {
            key,
            element_object: Box::new(StateObject { object: widget }),
        }
    }
    pub fn key(&self) -> Key {
        self.key
    }

    pub fn state_object<T: 'static>(&self) -> Option<&StateObject<T>> {
        self.element_object.downcast_ref::<StateObject<T>>()
    }

    pub fn render_object<T: WidgetInterface + 'static>(&self) -> Option<&RenderObject<T>> {
        self.element_object.downcast_ref::<RenderObject<T>>()
    }

    pub fn object<T: 'static>(&self) -> Option<&T> {
        self.element_object.downcast_ref::<T>()
    }

    pub fn object_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.element_object.downcast_mut::<T>()
    }
}

struct Window {
    arena: Arena<Element>,
    root: NodeId,
}

/// Static state that is shared between most contexts.
pub struct GlobalState<'a> {
    arena: &'a mut Arena<Element>,
}

fn find_sibling_node(arena: &Arena<Element>, current: NodeId, key: Key) -> Option<NodeId> {
    for node_id in current.following_siblings(arena) {
        if let Some(node) = arena.get(node_id) {
            if node.get().key == key {
                return Some(node_id);
            }
        }
    }
    None
}

fn remove_siblings_til(arena: &mut Arena<Element>, from: NodeId, to: Option<NodeId>) -> usize {
    let to_delete: Vec<_> = from
        .following_siblings(arena)
        .take_while(|n| Some(*n) != to)
        .collect();
    to_delete
        .iter()
        .for_each(|node_id| node_id.remove_subtree(arena));
    to_delete.len()
}

fn node_to_object<T: 'static>(arena: &Arena<Element>, node_id: NodeId) -> Option<&T> {
    let node = arena.get(node_id)?;
    node.get().object()
}

fn node_to_object_mut<T: 'static>(arena: &mut Arena<Element>, node_id: NodeId) -> Option<&mut T> {
    let node = arena.get_mut(node_id)?;
    node.get_mut().object_mut()
}

pub struct Ui<'a, 'g> {
    current_element: NodeId,
    global_state: &'a mut GlobalState<'g>,
}

impl<'a, 'g> Ui<'a, 'g> {
    pub fn new(current_element: NodeId, global_state: &'a mut GlobalState<'g>) -> Self {
        Ui {
            current_element,
            global_state,
        }
    }

    fn insert_element<P, W, N>(&mut self, key: Key, props: P, content: N)
    where
        P: Properties,
        W: Widget<P> + 'static,
        N: FnOnce(&mut Ui),
    {
        let arena = &mut self.global_state.arena;
        let node = find_sibling_node(arena, self.current_element, key);
        let object = node.and_then(|node| node_to_object_mut::<W>(arena, node));
        let matched_node = match (node, object) {
            (Some(node), Some(object)) => {
                let mut ctx = Ctx {};
                object.update(&mut ctx, props);
                node
            }
            _ => {
                let widget = W::create(props);
                let new_node = arena.new_node(Element::from_widget(key, widget));
                self.current_element.insert_after(new_node, arena);
                new_node
            }
        };
        let _ = remove_siblings_til(arena, self.current_element, Some(matched_node));
        self.current_element = matched_node;

        let mut child_ui = Ui::new(self.current_element, self.global_state);
        content(&mut child_ui);

        let arena = &mut self.global_state.arena;
        let count = remove_siblings_til(arena, self.current_element, None);

        // if request_layout {
        //     node.request_layout();
        // }

        // // let node_name = node.name();
        // for child in &mut node.children {
        //     // debug!("merge from {} to {}", child.name(), node_name);
        //     node.state.merge_up(&mut child.state);
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {}
}
