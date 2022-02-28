//! Unique identities.

use druid_shell::Counter;
use generational_indextree::NodeId;

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct WindowId(u64);

impl WindowId {
    /// Allocate a new, unique window id.
    pub fn next() -> WindowId {
        static WINDOW_COUNTER: Counter = Counter::new();
        WindowId(WINDOW_COUNTER.next())
    }
}
