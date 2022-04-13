//! Unique identities.

use druid_shell::Counter;

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Default)]
pub struct ElementId(u64);

impl ElementId {
    pub const ZERO: ElementId = ElementId(0);
    /// Allocate a new, unique window id.
    pub fn next() -> ElementId {
        static CHILD_COUNTER: Counter = Counter::new();
        ElementId(CHILD_COUNTER.next())
    }
}

impl ToString for ElementId {
    fn to_string(&self) -> String {
        self.0.to_string()
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
