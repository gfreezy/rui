pub mod cx;
pub mod mut_cursor;
pub mod mutation;

use druid_shell::kurbo::{Point, Size};

use crate::id::Id;
use crate::key::Key;
use crate::state::{AnyState, State};
use crate::tree::mutation::{Mutation, MutationItem};
use crate::view::AnyView;
use crate::{key::Caller, widgets::Widget};
use std::fmt::Debug;
use std::fmt::Write;
use std::panic::Location;

#[derive(Default, Debug)]
pub struct Tree {
    slots: Vec<Slot>,
}

#[derive(Debug)]
pub enum Slot {
    Begin(Item),
    End,
}

/// The type of an item in the tree.
#[derive(Debug)]
pub struct Item {
    key: Key,
    id: Id,
    body: Payload,
}

#[derive(Debug)]
pub enum Payload {
    Placeholder,
    View(AnyView),
    State(AnyState),
}

pub struct Node {
    start: usize,
    end: usize,
}

impl Tree {
    /// Apply the mutation, mutating the tree.
    pub fn apply_mutation(&mut self, mutation: Mutation) {
        // This implementation isn't trying to be efficient.
        let mut ix = 0;
        for mut_item in mutation {
            match mut_item {
                MutationItem::Skip(n) => ix += n,
                MutationItem::Delete(n) => {
                    self.slots.drain(ix..ix + n);
                }
                MutationItem::Insert(new) => {
                    let n = new.len();
                    self.slots.splice(ix..ix, new);
                    ix += n;
                }
                MutationItem::Update(new) => {
                    let n = new.len();
                    self.slots.splice(ix..ix + n, new);
                    ix += n;
                }
            }
        }
    }

    pub fn dump(&self) -> String {
        let mut s = String::new();
        let mut nest = 0;
        for slot in &self.slots {
            match slot {
                Slot::Begin(item) => {
                    writeln!(&mut s, "{}{:?}", "  ".repeat(nest), item);
                    nest += 1;
                }
                Slot::End => nest -= 1,
            }
        }
        s
    }

    fn previous_siblings(&self, mut ix: usize) -> impl Iterator<Item = (usize, &Item)> {
        let mut nest = 0;

        std::iter::from_fn(move || {
            while ix > 0 {
                ix -= 1;
                match &self.slots[ix] {
                    Slot::End => nest += 1,
                    Slot::Begin(item) => {
                        nest -= 1;
                        if nest == 0 {
                            return Some((ix, item));
                        } else if nest < 0 {
                            return None;
                        }
                    }
                }
            }
            None
        })
    }

    fn next_siblings(&self, mut ix: usize) -> impl Iterator<Item = (usize, &Item)> {
        let mut nest = 0;

        std::iter::from_fn(move || {
            while ix < self.slots.len() {
                match &self.slots[ix] {
                    Slot::Begin(item) => {
                        nest += 1;
                        ix += 1;
                        if nest == 1 {
                            return Some((ix - 1, item));
                        }
                    }
                    Slot::End => {
                        nest -= 1;
                        ix += 1;
                        if nest < 0 {
                            return None;
                        }
                    }
                }
            }
            None
        })
    }

    /// Find the key in the current node.
    ///
    /// Returns number of slots until the key.
    fn find_by_key(&self, ix: usize, key: Key) -> Option<(usize, &Item)> {
        self.next_siblings(ix).find(|(ix, item)| item.key == key)
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::mut_cursor::MutCursor;
    use crate::tree::{Payload, Tree};
    use expect_test::expect_file;

    fn component(mut_cursor: &mut MutCursor) {
        mut_cursor.begin_item();
        mut_cursor.set_current_payload(Payload::Placeholder);
        mut_cursor.end_item_and_begin_body();
        for _i in 0..4 {
            mut_cursor.begin_item();
            mut_cursor.set_current_payload(Payload::Placeholder);
            mut_cursor.end_item_and_begin_body();
            mut_cursor.end_body();
        }
        mut_cursor.end_body();
    }

    #[test]
    fn test_tree_siblings() {
        let mut tree = Tree::default();
        assert_eq!(tree.next_siblings(0).count(), 0);
        assert_eq!(tree.previous_siblings(0).count(), 0);
        let mut mut_cursor = MutCursor::new(&tree);
        component(&mut mut_cursor);
        let mutation = mut_cursor.into_mutation();
        tree.apply_mutation(mutation);
        dbg!(&tree);
        assert_eq!(tree.next_siblings(0).count(), 1);
        assert_eq!(tree.next_siblings(1).count(), 4);
        assert_eq!(tree.next_siblings(2).count(), 0);
        assert_eq!(tree.next_siblings(3).count(), 3);
        assert_eq!(tree.previous_siblings(0).count(), 0);
        assert_eq!(tree.previous_siblings(1).count(), 0);
        assert_eq!(tree.previous_siblings(2).count(), 0);
        assert_eq!(tree.previous_siblings(3).count(), 1);
        assert_eq!(tree.previous_siblings(5).count(), 2);
        assert_eq!(tree.previous_siblings(7).count(), 3);
    }

    #[test]
    fn test_tree() {
        let mut tree = Tree::default();
        let mut mut_cursor = MutCursor::new(&tree);
        component(&mut mut_cursor);
        let mutation = mut_cursor.into_mutation();
        tree.apply_mutation(mutation);
        expect_file!["./test_data/test_tree_before.txt"].assert_eq(&tree.dump());
        let mut mut_cursor = MutCursor::new(&tree);
        component(&mut mut_cursor);
        let mutation = mut_cursor.into_mutation();
        tree.apply_mutation(mutation);
        expect_file!["./test_data/test_tree_after.txt"].assert_eq(&tree.dump());
    }
}
