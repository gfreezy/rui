use std::any::Any;

use std::panic::Location;



use crate::context::{ContextState, UpdateCtx};
use crate::id::ChildCounter;
use crate::key::Caller;
use crate::object::{AnyRenderObject, Properties, RenderObject};
use crate::tree::{Child, Children, State, StateNode};

pub struct Ui<'a> {
    pub(crate) tree: &'a mut Children,
    context_state: &'a mut ContextState,
    child_counter: &'a mut ChildCounter,
    state_index: usize,
    render_index: usize,
}

impl<'a> Ui<'a> {
    pub(crate) fn new(
        tree: &'a mut Children,
        context_state: &'a mut ContextState,
        child_counter: &'a mut ChildCounter,
    ) -> Self {
        Ui {
            tree,
            context_state,
            child_counter,
            state_index: 0,
            render_index: 0,
        }
    }

    #[track_caller]
    pub fn state_node<T: 'static>(&mut self, init: impl FnOnce() -> T) -> State<T> {
        let caller = Location::caller().into();
        let idx = self.find_state_node(caller);
        let index = match idx {
            None => {
                let init_value: *mut dyn Any = self.tree.bump.alloc(init());
                self.insert_state_node(caller, init_value)
            }
            Some(index) => index,
        };
        for node in &mut self.tree.states[self.state_index..index] {
            node.dead = true;
        }
        self.state_index = index + 1;

        let state = &self.tree.states[index].state;
        let raw_box: *mut dyn Any = unsafe { &mut **state };

        State::new(raw_box)
    }

    pub fn render_object<P, R, N>(&mut self, caller: Caller, props: P, content: N) -> R::Action
    where
        P: Properties<Object = R>,
        R: RenderObject<P> + Any,
        N: FnOnce(&mut Ui),
    {
        let mut action = R::Action::default();
        let index = if let Some(index) = self.find_render_object(caller) {
            let node = &mut self.tree.renders[index];
            if let Some(object) = node.object.as_any().downcast_mut::<R>() {
                let mut ctx = UpdateCtx {
                    context_state: self.context_state,
                    child_state: &mut node.state,
                };
                action = object.update(&mut ctx, props);
            } else {
                // TODO: Think of something smart
                panic!("Wrong node type. Expected {}", std::any::type_name::<R>())
            }
            index
        } else {
            let object = R::create(props);
            let index = self.insert_render_object(caller, object);
            let node = &mut self.tree.renders[index];
            node.request_layout();
            index
        };
        for node in &mut self.tree.renders[self.render_index..index] {
            node.dead = true;
        }
        self.render_index = index + 1;

        let node = &mut self.tree.renders[index];
        let mut child_ui = Ui::new(&mut node.children, self.context_state, self.child_counter);
        content(&mut child_ui);

        let mut request_layout = false;
        let child_states = &mut child_ui.tree.states;
        let child_renders = &mut child_ui.tree.renders;
        if child_states.len() > child_ui.state_index {
            child_states.truncate(child_ui.state_index);
            request_layout = true;
        }
        if child_states.iter().any(|s| s.dead) {
            child_states.retain(|s| !s.dead);
            request_layout = true;
        }
        if child_renders.len() > child_ui.render_index {
            child_renders.truncate(child_ui.render_index);
            request_layout = true;
        }
        if child_renders.iter().any(|s| s.dead) {
            child_renders.retain(|c| !c.dead);
            request_layout = true;
        }

        if request_layout {
            node.request_layout();
        }

        // let node_name = node.name();
        for child in &mut node.children {
            // debug!("merge from {} to {}", child.name(), node_name);
            node.state.merge_up(&mut child.state);
        }

        action
    }
}

impl Ui<'_> {
    fn find_state_node(&mut self, caller: Caller) -> Option<usize> {
        let mut ix = self.state_index;
        for node in &mut self.tree.states[ix..] {
            if node.key == caller {
                return Some(ix);
            }
            ix += 1;
        }
        None
    }

    fn insert_state_node(&mut self, caller: Caller, state: *mut dyn Any) -> usize {
        let key = caller;
        let dead = false;
        self.tree
            .states
            .insert(self.state_index, StateNode { key, state, dead });
        self.state_index
    }

    fn find_render_object(&mut self, caller: Caller) -> Option<usize> {
        let mut ix = self.render_index;
        for node in &mut self.tree.renders[ix..] {
            if node.key == caller {
                return Some(ix);
            }
            ix += 1;
        }
        None
    }

    fn insert_render_object(&mut self, caller: Caller, object: impl AnyRenderObject) -> usize {
        self.tree.renders.insert(
            self.render_index,
            Child::new(caller, object, self.child_counter.generate_id()),
        );
        self.render_index
    }

    pub fn track_state(&mut self, state_name: String) {
        self.tree.track_state(state_name);
    }
}
