use std::any::Any;
use std::panic::Location;

use crate::context::{ContextState, UpdateCtx};
use crate::id::ChildCounter;
use crate::key::Caller;
use crate::object::{AnyRenderObject, Properties, RenderObject};
use crate::tree::{Child, ChildState, Children, State, StateNode};

pub struct Ui<'a, 'b> {
    pub(crate) tree: &'a mut Children,
    context_state: &'a mut ContextState<'b>,
    child_counter: &'a mut ChildCounter,
    state_index: usize,
    render_index: usize,
}

impl<'a, 'b> Ui<'a, 'b> {
    pub(crate) fn new(
        tree: &'a mut Children,
        context_state: &'a mut ContextState<'b>,
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
        let index = match self.find_state_node(caller) {
            None => {
                let init_value = Box::new(State::new(init()));
                self.insert_state_node(caller, init_value)
            }
            Some(index) => index,
        };
        for node in &mut self.tree.states[self.state_index..index] {
            node.dead = true;
        }
        self.state_index = index + 1;

        self.tree.states[index]
            .state
            .downcast_ref::<State<T>>()
            .unwrap()
            .clone()
    }

    pub fn render_object<P, R, N>(&mut self, caller: Caller, props: P, content: N) -> R::Action
    where
        P: Properties<Object = R>,
        R: RenderObject<P> + Any,
        N: FnOnce(&mut Ui),
    {
        let mut props = Some(props);
        let index = match self.find_render_object(caller) {
            Some(index) => index,
            None => {
                let object = R::create(props.take().unwrap());
                self.insert_render_object(caller, Box::new(object))
            }
        };
        for node in &mut self.tree.renders[self.render_index..index] {
            node.dead = true;
        }
        let node = &mut self.tree.renders[index];
        self.render_index = index + 1;

        let mut action = R::Action::default();
        if let Some(props) = props {
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
        }

        let mut object_cx = Ui::new(&mut node.children, self.context_state, self.child_counter);
        content(&mut object_cx);

        object_cx.tree.states.truncate(object_cx.state_index);
        object_cx.tree.states.retain(|s| !s.dead);
        object_cx.tree.renders.truncate(object_cx.render_index);
        object_cx.tree.renders.retain(|c| !c.dead);

        for child in &mut node.children {
            node.state.merge_up(&mut child.state);
        }

        action
    }
}

impl Ui<'_, '_> {
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

    fn insert_state_node(&mut self, caller: Caller, state: Box<dyn Any>) -> usize {
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

    fn insert_render_object(&mut self, caller: Caller, object: Box<dyn AnyRenderObject>) -> usize {
        self.tree.renders.insert(
            self.render_index,
            Child {
                key: caller,
                object,
                children: Children::new(),
                state: ChildState::new(self.child_counter.generate_id(), None),
                dead: false,
            },
        );
        self.render_index
    }

    pub fn track_state(&mut self, state_name: String) {
        self.tree.track_state(state_name);
    }
}
