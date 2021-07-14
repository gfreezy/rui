use crate::tree::Slot;
use std::vec::IntoIter;

#[derive(Debug)]
pub struct Mutation(Vec<MutationItem>);

impl Mutation {
    pub fn new() -> Mutation {
        Mutation(Vec::new())
    }

    pub fn skip(&mut self, n: usize) {
        if n > 0 {
            if let Some(MutationItem::Skip(old_n)) = self.0.last_mut() {
                *old_n += n;
            } else {
                self.0.push(MutationItem::Skip(n));
            }
        }
    }

    pub fn delete(&mut self, n: usize) {
        if n > 0 {
            if let Some(MutationItem::Delete(old_n)) = self.0.last_mut() {
                *old_n += n;
            } else {
                self.0.push(MutationItem::Delete(n));
            }
        }
    }

    fn insert(&mut self, new: Vec<Slot>) {
        if !new.is_empty() {
            if let Some(MutationItem::Insert(old)) = self.0.last_mut() {
                old.extend(new);
            } else {
                self.0.push(MutationItem::Insert(new));
            }
        }
    }

    /// Insert a single slot.
    ///
    /// This is semantically the same as insert, but potentially more
    /// efficient, and also convenient.
    pub fn insert_one(&mut self, slot: Slot) {
        // Just punt for now :)
        self.insert(vec![slot]);
    }

    fn update(&mut self, new: Vec<Slot>) {
        if !new.is_empty() {
            if let Some(MutationItem::Update(old)) = self.0.last_mut() {
                old.extend(new);
            } else {
                self.0.push(MutationItem::Update(new));
            }
        }
    }

    /// Update a single slot.
    ///
    /// This is semantically the same as update, but potentially more
    /// efficient, and also convenient.
    pub(crate) fn update_one(&mut self, slot: Slot) {
        // Just punt for now :)
        self.update(vec![slot]);
    }
}

impl IntoIterator for Mutation {
    type Item = MutationItem;
    type IntoIter = IntoIter<MutationItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// One item in the internal representation of a tree mutation.
#[derive(Debug)]
pub enum MutationItem {
    /// No change for the next n slots.
    Skip(usize),
    /// Delete the next n slots.
    Delete(usize),
    /// Insert new items at the current location.
    Insert(Vec<Slot>),
    /// Update existing items.
    ///
    /// Update is similar to delete + insert, but is intended to
    /// preserve the identity of those tree locations.
    Update(Vec<Slot>),
}
