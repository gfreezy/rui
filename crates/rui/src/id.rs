//! Unique identities.

use std::{cell::Cell, rc::Rc};

use druid_shell::Counter;

#[derive(Debug, Default)]
pub struct ChildCounter(Rc<Cell<usize>>);

impl Clone for ChildCounter {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl ChildCounter {
    pub fn new() -> Self {
        ChildCounter(Rc::new(Cell::new(0)))
    }

    pub fn generate_id(&self) -> ChildId {
        let old = self.0.get();
        self.0.replace(old + 1);
        ChildId(old + 1)
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct ChildId(usize);

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct WindowId(u64);

impl WindowId {
    /// Allocate a new, unique window id.
    pub fn next() -> WindowId {
        static WINDOW_COUNTER: Counter = Counter::new();
        WindowId(WINDOW_COUNTER.next())
    }
}
