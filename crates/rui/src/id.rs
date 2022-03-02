//! Unique identities.

use druid_shell::Counter;

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct ChildId(u64);

impl ChildId {
    /// Allocate a new, unique window id.
    pub fn next() -> ChildId {
        static CHILD_COUNTER: Counter = Counter::new();
        ChildId(CHILD_COUNTER.next())
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct WindowId(u64);

impl WindowId {
    /// Allocate a new, unique window id.
    pub fn next() -> WindowId {
        static WINDOW_COUNTER: Counter = Counter::new();
        WindowId(WINDOW_COUNTER.next())
    }
}
