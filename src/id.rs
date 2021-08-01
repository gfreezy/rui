//! Unique identities.

#[derive(Debug, Default)]
pub struct ChildCounter(usize);

impl ChildCounter {
    pub fn new() -> Self {
        ChildCounter(0)
    }

    pub fn generate_id(&mut self) -> ChildId {
        self.0 += 1;
        ChildId(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct ChildId(usize);

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct WindowId(usize);
