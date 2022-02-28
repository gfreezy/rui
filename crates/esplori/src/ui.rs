use std::panic::Location;
use std::thread::LocalKey;

use generational_indextree::{Arena, NodeId};

use crate::context::{GlobalState, UpdateCtx};
use crate::ext_event::ExtEventSink;

use crate::key::Caller;
use crate::object::{Properties, Render};
use crate::tree::ElementId;
use crate::tree::{Element, RenderObject, StateObject};

fn find_sibling_node(arena: &Arena<Element>, current: NodeId, key: Caller) -> Option<NodeId> {
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

pub(crate) fn node_to_object_mut<T: 'static>(
    arena: &mut Arena<Element>,
    node_id: NodeId,
) -> Option<&mut T> {
    let node = arena.get_mut(node_id)?;
    node.get_mut().object_mut()
}

pub struct Ui<'a, 'g> {
    arena: &'g mut Arena<Element>,
    global_state: &'a mut GlobalState,
    current_element: NodeId,
}

impl<'a, 'g> Ui<'a, 'g> {
    pub fn new(
        current_element: NodeId,
        global_state: &'a mut GlobalState,
        arena: &'g mut Arena<Element>,
    ) -> Self {
        Ui {
            arena,
            current_element,
            global_state,
        }
    }

    #[track_caller]
    fn render_node<P, W, N>(&mut self, key: Caller, props: P, content: N)
    where
        P: Properties,
        W: Render<P> + 'static,
        N: FnOnce(&mut Ui),
    {
        let arena = &mut self.arena;
        let node = find_sibling_node(arena, self.current_element, key);
        let object = node.and_then(|node| node_to_object_mut::<RenderObject>(arena, node));
        let current_node = match (node, object) {
            (Some(node), Some(object)) => {
                let mut ctx = UpdateCtx {
                    ui_state: &mut object.state,
                    global_state: self.global_state,
                };
                object
                    .object
                    .as_any()
                    .downcast_mut::<W>()
                    .unwrap()
                    .update(&mut ctx, props);
                node
            }
            _ => {
                let widget = W::create(props);
                let new_node =
                    arena.new_node_with(|id| Element::from_widget(key, widget, ElementId(id)));
                self.current_element.insert_after(new_node, arena);
                new_node
            }
        };
        let _ = remove_siblings_til(arena, self.current_element, Some(current_node));
        self.current_element = current_node;

        let mut child_ui = Ui::new(self.current_element, self.global_state, arena);
        content(&mut child_ui);

        let arena = &mut self.arena;
        let count = remove_siblings_til(arena, self.current_element, None);
        let current_render_object = node_to_object_mut::<RenderObject>(arena, current_node)
            .expect("failed to find render object");

        if count > 0 {
            current_render_object.request_layout();
        }

        for node in current_node.children(arena) {
            if let Some(render_node) = node_to_object_mut::<RenderObject>(arena, node) {
                current_render_object.state.merge_up(&mut render_node.state)
            }
        }
    }

    #[track_caller]
    pub fn state_node<T: 'static>(&mut self, init: impl FnOnce() -> T) -> &T {
        let key = Location::caller().into();
        let arena = &mut self.arena;
        let node = find_sibling_node(arena, self.current_element, key);
        let object = node.and_then(|node| node_to_object_mut::<StateObject<T>>(arena, node));
        let current_node = match (node, object) {
            (Some(node), Some(object)) => node,
            _ => {
                let value = init();
                let new_node = arena.new_node(Element::from_state(key, value));
                self.current_element.insert_after(new_node, arena);
                new_node
            }
        };
        let _ = remove_siblings_til(arena, self.current_element, Some(current_node));
        self.current_element = current_node;
        node_to_object(arena, current_node).unwrap()
    }

    pub fn ext_handle(&self) -> &ExtEventSink {
        &self.global_state.ext_handle
    }
}
