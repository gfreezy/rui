use std::any::Any;

use std::cell::RefCell;
use std::ops::Index;
use std::panic::Location;
use std::rc::{Rc, Weak};

use crate::context::{ContextState, UpdateCtx};
use crate::ext_event::ExtEventSink;
use crate::id::ElementId;
use crate::key::{Key, LocalKey};
use crate::object::{AnyParentData, AnyRenderObject, Properties, RenderObject};
use crate::perf::measure_time;
use crate::tree::{Children, Element, InnerElement, State, StateNode};

pub struct Ui<'a, 'b, 'c, 'c2> {
    pub tree: &'a mut Children,
    parent: Option<Weak<RefCell<InnerElement>>>,
    context_state: &'b mut ContextState<'c, 'c2>,
    state_index: usize,
    render_index: usize,
    parent_data: Option<Box<dyn AnyParentData>>,
}

pub enum RenderAction {
    Insert(usize),
    Update(usize),
    Auto,
}

impl<'a, 'b, 'c, 'c2> Ui<'a, 'b, 'c, 'c2> {
    pub(crate) fn new(
        tree: &'a mut Children,
        context_state: &'b mut ContextState<'c, 'c2>,
        parent: Option<Weak<RefCell<InnerElement>>>,
    ) -> Self {
        Ui {
            parent,
            tree,
            context_state,
            state_index: 0,
            render_index: 0,
            parent_data: None,
        }
    }

    fn alloc<T>(&mut self, val: T) -> &mut T {
        self.context_state.bump.alloc(val)
    }

    pub fn set_parent_data(&mut self, parent_data: Option<Box<dyn AnyParentData>>) {
        self.parent_data = parent_data;
    }

    #[track_caller]
    pub fn state_node<T: 'static>(&mut self, init: impl FnOnce() -> T) -> State<T> {
        let key = Location::caller().into();
        let idx = self.find_state_node(key);
        let index = match idx {
            None => {
                let init_value: *mut dyn Any = self.alloc(init());
                self.insert_state_node(key, init_value)
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

    pub fn render_object_pro<Props, R, N>(
        &mut self,
        key: impl Into<(Key, LocalKey)>,
        props: Props,
        render_action: RenderAction,
        parent_data: Option<Box<dyn AnyParentData>>,
        content: Option<N>,
    ) -> R::Action
    where
        Props: Properties<Object = R>,
        R: RenderObject<Props> + Any,
        N: FnOnce(&mut Ui),
    {
        match render_action {
            RenderAction::Insert(at) => {
                self.render_index = at;
            }
            RenderAction::Update(at) => {
                self.render_index = at;
            }
            RenderAction::Auto => {}
        }

        if parent_data.is_some() {
            self.set_parent_data(parent_data);
        }
        let mut action = R::Action::default();
        let (key, local_key) = key.into();
        let index = match (render_action, self.find_render_object(key, &local_key)) {
            (RenderAction::Insert(_), _) | (RenderAction::Auto, None) => {
                let object = R::create(props);
                let index = self.insert_render_object(key, local_key.clone(), object);
                let node = &mut self.tree.renders[index];
                node.request_layout();
                tracing::trace!(
                    "Insert render object, key: {:?}, local_key: {}, index: {}",
                    key,
                    &local_key,
                    index
                );
                index
            }
            (RenderAction::Update(_), Some(index)) | (RenderAction::Auto, Some(index)) => {
                let mut guard = self.tree.renders[index].inner.borrow_mut();
                let inner_node = &mut *guard;
                let object = inner_node
                    .object
                    .as_any()
                    .downcast_mut::<R>()
                    .expect(&format!(
                        "Wrong node type. Expected {}",
                        std::any::type_name::<R>()
                    ));

                let mut ctx = UpdateCtx {
                    context_state: self.context_state,
                    child_state: &mut inner_node.state,
                    parent: self.parent.clone(),
                };
                action = object.update(&mut ctx, props, &mut inner_node.children);
                tracing::trace!(
                    "Update render object, key: {:?}, local_key: {}, index: {}",
                    key,
                    &local_key,
                    index
                );
                index
            }
            (RenderAction::Update(index), None) => {
                panic!(
                    "Update render object, but not found, key: {:?}, local_key: {}, index: {}",
                    key, local_key, index
                );
            }
        };

        for node in &mut self.tree.renders[self.render_index..index] {
            node.mark_dead();
        }
        self.render_index = index + 1;

        let node = &self.tree.renders[index].inner;
        let parent = Rc::downgrade(node);
        let mut inner_node = node.borrow_mut();
        inner_node.local_key = local_key;

        let changed = inner_node.set_parent_data(self.parent_data.take());
        if changed {
            inner_node.request_layout();
        }

        if let Some(content) = content {
            let mut child_ui = Ui::new(&mut inner_node.children, self.context_state, Some(parent));
            content(&mut child_ui);

            if child_ui.cleanup_tree() {
                inner_node.request_layout();
            }
            inner_node.merge_child_states_up();
        }

        action
    }

    pub fn render_object<Props, R, N>(
        &mut self,
        key: impl Into<(Key, LocalKey)>,
        props: Props,
        content: N,
    ) -> R::Action
    where
        Props: Properties<Object = R>,
        R: RenderObject<Props> + Any,
        N: FnOnce(&mut Ui),
    {
        self.render_object_pro(key, props, RenderAction::Auto, None, Some(content))
    }
}

impl Ui<'_, '_, '_, '_> {
    fn parent(&self) -> Option<Weak<RefCell<InnerElement>>> {
        self.parent.clone()
    }

    fn find_state_node(&mut self, key: Key) -> Option<usize> {
        let mut ix = self.state_index;
        for node in &mut self.tree.states[ix..] {
            if node.key == key {
                return Some(ix);
            }
            ix += 1;
        }
        None
    }

    fn insert_state_node(&mut self, key: Key, state: *mut dyn Any) -> usize {
        let key = key;
        let dead = false;
        self.tree
            .states
            .insert(self.state_index, StateNode { key, state, dead });
        self.state_index
    }

    fn find_render_object(&mut self, key: Key, local_key: &LocalKey) -> Option<usize> {
        let mut ix = self.render_index;
        for node in &mut self.tree.renders[ix..] {
            if node.key() == key && &node.local_key() == local_key {
                return Some(ix);
            }
            ix += 1;
        }
        None
    }

    fn insert_render_object(
        &mut self,
        key: Key,
        local_key: LocalKey,
        object: impl AnyRenderObject,
    ) -> usize {
        self.tree.renders.insert(
            self.render_index,
            Element::new(key, local_key, object, self.parent()),
        );
        self.render_index
    }

    pub(crate) fn cleanup_tree(&mut self) -> bool {
        let mut request_layout = false;
        let states = &mut self.tree.states;
        let renders = &mut self.tree.renders;
        if states.len() > self.state_index {
            states.truncate(self.state_index);
            request_layout = true;
        }
        if states.iter().any(|s| s.dead) {
            states.retain(|s| !s.dead);
            request_layout = true;
        }
        if renders.len() > self.render_index {
            renders.truncate(self.render_index);
            request_layout = true;
        }
        if renders.iter().any(|s| s.dead()) {
            renders.retain(|c| !c.dead());
            request_layout = true;
        }
        request_layout
    }

    pub fn ext_handle(&self) -> &ExtEventSink {
        &self.context_state.ext_handle
    }
}
