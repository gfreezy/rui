use std::any::Any;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Index;
use std::panic::Location;
use std::rc::{Rc, Weak};

use crate::context::{ContextState, UpdateCtx};
use crate::ext_event::ExtEventSink;
use crate::id::ElementId;
use crate::key::{Key, LocalKey};
use crate::object::{AnyParentData, AnyRenderObject, Properties, RenderObject};
use crate::perf::measure_time;
use crate::tree::{Children, Element, InnerElement, Meoizee, State, StateHandle, StateNode};

pub trait Component<Params> {
    fn call(self, ui: &mut Ui, params: Params);
}

macro_rules! component_args {
    () => {
        impl<F: FnOnce(&mut Ui)> Component<()> for F {
            fn call(self, ui: &mut Ui, params: ()) {
                self(ui);
            }
        }
    };
    ($P: tt) => {
        impl<P: Clone + PartialEq + 'static, F: FnOnce(&mut Ui, P)> Component<(P,)> for F {
            fn call(self, ui: &mut Ui, params: (P,)) {
                let (p,) = params;
                self(ui, p);
            }
        }
    };
    ($($P: tt),*) => {
        #[allow(non_snake_case)]
        impl<$($P: Clone + PartialEq + 'static),*, F: FnOnce(&mut Ui, $($P),*)> Component<($($P),*)>
            for F
        {
            fn call(self, ui: &mut Ui, params: ($($P),*)) {
                let ($($P),*) = params;
                self(ui, $($P),*);
            }
        }
    };
}
component_args!();
component_args!(P1);
component_args!(P1, P2);
component_args!(P1, P2, P3);
component_args!(P1, P2, P3, P4);
component_args!(P1, P2, P3, P4, P5);
component_args!(P1, P2, P3, P4, P5, P6);
component_args!(P1, P2, P3, P4, P5, P6, P7);
component_args!(P1, P2, P3, P4, P5, P6, P7, P8);
component_args!(P1, P2, P3, P4, P5, P6, P7, P8, P9);
component_args!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);

pub struct Ui<'a, 'b, 'c, 'c2> {
    tree: &'a mut Children,
    parent_element: Option<Weak<RefCell<InnerElement>>>,
    context_state: &'b mut ContextState<'c, 'c2>,
    state_index: usize,
    render_index: usize,
    memoizee_index: usize,
    need_update: bool,
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
        parent_element: Option<Weak<RefCell<InnerElement>>>,
        need_update: bool,
    ) -> Self {
        Ui {
            parent_element,
            tree,
            context_state,
            state_index: 0,
            render_index: 0,
            memoizee_index: 0,
            need_update,
            parent_data: None,
        }
    }

    pub(crate) fn dirty_elements(&self) -> Rc<RefCell<Vec<Weak<RefCell<InnerElement>>>>> {
        self.context_state.dirty_elements()
    }

    pub fn tree(&self) -> &Children {
        &self.tree
    }

    pub fn element_needs_update(&self, index: usize) -> bool {
        let element = self.tree.get(index);
        let need_update = element.map(|e| e.needs_update()).unwrap_or(true);
        let name = element
            .map(|e| e.name().to_string())
            .unwrap_or_else(|| "".to_string());
        tracing::trace!(
            "name: {name}, index: {}, need_update: {}, element_needs_update: {:?}",
            index,
            self.need_update,
            need_update,
        );
        need_update || self.need_update
    }

    fn alloc<T>(&mut self, val: T) -> &mut T {
        self.context_state.bump.alloc(val)
    }

    pub(crate) fn parent_element(&self) -> Option<Weak<RefCell<InnerElement>>> {
        self.parent_element.clone()
    }

    pub fn set_parent_data(&mut self, parent_data: Option<Box<dyn AnyParentData>>) {
        self.parent_data = parent_data;
    }

    #[track_caller]
    pub fn state_node<T: 'static>(&mut self, init: impl FnOnce() -> T) -> StateHandle<T> {
        let key = Location::caller().into();
        let idx = self.find_state_node(key);
        let index = match idx {
            None => {
                let init_value: *mut dyn Any = self.alloc(State::new(init()));
                self.insert_state_node(key, init_value)
            }
            Some(index) => index,
        };
        for node in &mut self.tree.states[self.state_index..index] {
            node.dead = true;
        }
        self.state_index = index + 1;

        let state = self.tree.states[index].state;

        StateHandle::new(state)
    }

    #[track_caller]
    pub fn memoize<Params: PartialEq + Clone + Debug + 'static, Comp: Component<Params>>(
        &mut self,
        component: Comp,
        params: Params,
    ) {
        let key = Location::caller().into();
        let mut params_changed = false;
        let idx = self.find_memoizee(key);
        let index = match idx {
            None => {
                params_changed = true;
                self.insert_memoizee(key, Box::new(params.clone()))
            }
            Some(index) => {
                let old_params = self.tree.memoizees[index]
                    .val
                    .downcast_mut::<Params>()
                    .unwrap();
                tracing::debug!("old_params: {:?}, new_params: {:?}", old_params, params);
                if old_params != &params {
                    params_changed = true;
                    *old_params = params.clone();
                }

                index
            }
        };
        for node in &mut self.tree.memoizees[self.memoizee_index..index] {
            node.dead = true;
        }
        self.memoizee_index = index + 1;

        self.need_update = params_changed;
        component.call(self, params);
        self.need_update = true;
    }

    pub fn render_object_advanced<Props, R, N>(
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
                if self.element_needs_update(index) {
                    let node = &mut self.tree.renders[index];
                    node.request_layout();
                    tracing::trace!(
                        "Insert render object, key: {:?}, local_key: {}, index: {}",
                        key,
                        &local_key,
                        index
                    );
                }
                index
            }
            (RenderAction::Update(_), Some(index)) | (RenderAction::Auto, Some(index)) => {
                if self.element_needs_update(index) {
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
                        parent: self.parent_element.clone(),
                    };
                    action = object.update(&mut ctx, props);
                    tracing::trace!(
                        "Update render object, key: {:?}, local_key: {}, index: {}",
                        key,
                        &local_key,
                        index
                    );
                }
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

        if self.element_needs_update(index) {
            let node = &self.tree.renders[index].inner;
            let parent_element = Rc::downgrade(node);
            let mut current_element = node.borrow_mut();
            current_element.local_key = local_key;
            current_element.clear_needs_update();

            let changed = current_element.set_parent_data(self.parent_data.take());
            if changed {
                current_element.request_layout();
            }

            if let Some(content) = content {
                let mut child_ui = Ui::new(
                    &mut current_element.children,
                    self.context_state,
                    Some(parent_element),
                    true,
                );
                content(&mut child_ui);

                if child_ui.cleanup_tree() {
                    current_element.request_layout();
                }
                current_element.merge_child_states_up();
            }
        }

        let _ = self.parent_data.take();

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
        self.render_object_advanced(key, props, RenderAction::Auto, None, Some(content))
    }
}

impl<T> Index<StateHandle<T>> for Ui<'_, '_, '_, '_> {
    type Output = T;

    fn index(&self, index: StateHandle<T>) -> &Self::Output {
        index.get(self)
    }
}

impl Ui<'_, '_, '_, '_> {
    fn parent(&self) -> Option<Weak<RefCell<InnerElement>>> {
        self.parent_element.clone()
    }

    fn find_memoizee(&mut self, key: Key) -> Option<usize> {
        let mut ix = self.memoizee_index;
        for node in &mut self.tree.memoizees[ix..] {
            if node.key == key {
                return Some(ix);
            }
            ix += 1;
        }
        None
    }

    fn insert_memoizee(&mut self, key: Key, meoizee: Box<dyn Any>) -> usize {
        let dead = false;
        self.tree.memoizees.insert(
            self.memoizee_index,
            Meoizee {
                key,
                val: meoizee,
                dead,
            },
        );
        self.memoizee_index
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
        let meoizees = &mut self.tree.memoizees;

        if renders.len() > self.render_index {
            renders.truncate(self.render_index);
            request_layout = true;
        }
        if renders.iter().any(|s| s.dead()) {
            renders.retain(|c| !c.dead());
            request_layout = true;
        }

        states.truncate(self.state_index);
        states.retain(|s| !s.dead);

        meoizees.truncate(self.memoizee_index);
        meoizees.retain(|c| !c.dead);

        request_layout
    }

    pub fn ext_handle(&self) -> &ExtEventSink {
        &self.context_state.ext_handle
    }
}
