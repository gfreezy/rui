// use std::{any::Any, mem, rc::Rc};

// use debug_cell::RefCell;

// #[derive(Clone)]
// struct Cx {
//     inner: Rc<RefCell<InnerCx>>,
// }

// struct InnerCx {
//     states: Vec<Box<dyn Any>>,
//     dirty: bool,
// }

// #[derive(Clone, Copy)]
// struct State<'cx, T> {
//     idx: usize,
//     cx: &'cx Cx,
//     phantom: std::marker::PhantomData<T>,
// }

// impl<'cx, T> State<'cx, T> {
//     fn new(cx: &Cx, default: impl FnOnce() -> T) -> Self {
//         let mut inner = cx.inner.borrow_mut();
//         let idx = inner.states.len();
//         inner.states.push(Box::new(default()));
//         Self {
//             idx,
//             cx,
//             phantom: std::marker::PhantomData,
//         }
//     }

//     fn get(&self) -> &T {
//         self.cx.inner.borrow_mut().states[self.idx]
//             .downcast_ref()
//             .unwrap()
//     }

//     fn set(&self, v: T) -> T {
//         let old = mem::replace(
//             &mut self.cx.inner.borrow_mut().states[self.idx],
//             Box::new(v),
//         );
//         *old.downcast().unwrap()
//     }

//     fn update(&self, update_fn: impl FnOnce(&mut T)) {
//         let v = &mut self.cx.inner.borrow_mut().states[self.idx];
//         update_fn(v.downcast_mut().unwrap());
//     }
// }

// impl Cx {
//     fn new() -> Self {
//         Cx {
//             inner: Rc::new(RefCell::new(InnerCx {
//                 states: vec![],
//                 dirty: false,
//             })),
//         }
//     }

//     fn insert_state_val(&self, v: Box<dyn Any>) -> usize {
//         let mut inner = self.inner.borrow_mut();
//         inner.states.push(v);
//         inner.states.len() - 1
//     }

//     fn create_state<T>(&self, default: impl FnOnce() -> T) -> State<T> {
//         State::new(self, default)
//     }

//     fn mark_dirty(&mut self) {
//         self.inner.borrow_mut().dirty = true
//     }
// }

// trait Component {
//     type Props;

//     fn create(cx: Cx, props: Self::Props) -> Self;
//     fn render(&self);
// }

// struct Todo {
//     text: String,
//     completed: bool,
// }

// struct TodoListComp {
//     cx: Cx,
//     props: String,
// }

// impl Component for TodoListComp {
//     type Props = String;

//     fn create(cx: Cx, props: Self::Props) -> Self {
//         Self { cx, props }
//     }

//     fn render(&self) {
//         let todos: State<Vec<Todo>> = self.cx.create_state(|| vec![]);
//         let toggle = move |idx: usize| {
//             todos.update(|todo| todo[idx].completed = !todo[idx].completed);
//         };
//         let add = move |text: String| {
//             todos.update(|todos| {
//                 todos.push(Todo {
//                     text,
//                     completed: false,
//                 });
//             });
//         };

//         for todo in todos.get() {
//             let todo_comp = TodoComp::create(
//                 self.cx.clone(),
//                 TodoCompProps {
//                     toggle: Box::new(toggle),
//                     add: Box::new(add),
//                     todo: todo.clone(),
//                 },
//             );
//             todo_comp.render();
//         }
//     }
// }

// struct TodoCompProps<'a> {
//     todo: &'a Todo,
//     toggle: Box<dyn FnOnce(usize)>,
//     add: Box<dyn FnOnce(String)>,
// }

// struct TodoComp<'a> {
//     cx: Cx,
//     props: TodoCompProps<'a>,
// }

// impl<'a> Component for TodoComp<'a> {
//     type Props = TodoCompProps<'a>;

//     fn create(cx: Cx, props: Self::Props) -> Self {
//         Self { cx, props }
//     }

//     fn render(&self) {}
// }

// struct TextComp {
//     cx: Cx,
//     props: String,
// }

// impl Component for TextComp {
//     type Props = String;

//     fn create(cx: Cx, props: Self::Props) -> Self {
//         Self { cx, props }
//     }

//     fn render(&self) {}
// }

fn main() {}
