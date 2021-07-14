use crate::id::Id;
use crate::key::{Caller, Key};
use crate::tree::mutation::Mutation;
use crate::tree::{Item, Payload, Slot, Tree};
use std::panic::Location;
use std::slice::Iter;
use std::vec::IntoIter;

pub struct MutCursor<'a> {
    tree: &'a Tree,
    ix: usize,
    mutation: Mutation,
    // Current nesting level (mutating)
    nest: usize,
    // Nesting level in old tree
    old_nest: usize,
    current: Option<CurrentItem>,
}

enum PayloadMutation {
    Skip,
    Insert(Option<Payload>),
    Update(Payload),
}

struct CurrentItem {
    id: Id,
    key: Key,
    payload_mutation: PayloadMutation,
}

impl<'a> MutCursor<'a> {
    /// Start building a tree mutation.
    pub fn new(tree: &Tree) -> MutCursor {
        MutCursor {
            tree,
            ix: 0,
            mutation: Mutation::new(),
            nest: 0,
            old_nest: 0,
            current: None,
        }
    }

    pub fn get_current_id(&self) -> Id {
        self.current.as_ref().unwrap().id
    }

    fn get_item_at(&self, ix: usize) -> &Item {
        match self.tree.slots.get(self.ix - 1) {
            Some(Slot::Begin(item)) => item,
            _ => panic!("Expected item body but found something else"),
        }
    }

    pub fn get_current_payload(&self) -> Option<&Payload> {
        let current = match &self.current {
            None => panic!("MutCursor::get_current_payload called before calling MutCursor::end_item_and_begin_body"),
            Some(c) => c,
        };
        match &current.payload_mutation {
            PayloadMutation::Insert(payload) => payload.as_ref(),
            PayloadMutation::Update(payload) => Some(payload),
            PayloadMutation::Skip => Some(&self.get_item_at(self.ix - 1).body),
        }
    }

    pub fn set_current_payload(&mut self, new: Payload) {
        let current = match &mut self.current {
            None => {
                panic!("MutCursor::set_current_payload called before calling MutCursor::begin_item")
            }
            Some(c) => c,
        };
        match &mut current.payload_mutation {
            PayloadMutation::Insert(payload) => {
                *payload = Some(new);
            }
            PayloadMutation::Update(payload) => {
                *payload = new;
            }
            m @ PayloadMutation::Skip => {
                *m = PayloadMutation::Update(new);
            }
        }
    }

    /// The number of previous items in this node with this caller.
    fn caller_seq(&self, caller: Caller) -> usize {
        self.tree
            .previous_siblings(self.ix)
            .filter(|(_, item)| item.key.caller == caller)
            .count()
    }

    pub(crate) fn key_from_loc(&self, loc: &'static Location) -> Key {
        // todo: 需要修复。未插入的时候 location_seq 永远为 0
        let caller: Caller = loc.into();
        Key::new(caller, self.caller_seq(caller))
    }

    /// The number of slots until the end of the current node.
    fn count_trim(&self) -> usize {
        let mut nest = 0usize;
        let mut ix = self.ix;
        loop {
            match self.tree.slots[ix] {
                Slot::Begin(_) => nest += 1,
                Slot::End => {
                    if nest == 0 {
                        return ix - self.ix;
                    }
                    nest -= 1;
                }
            }
            ix += 1;
        }
    }

    #[track_caller]
    pub fn begin_item(&mut self) -> bool {
        self.begin_item_at(Location::caller())
    }

    pub fn begin_item_at(&mut self, location: &'static Location) -> bool {
        assert!(
            self.current.is_none(),
            "MutCursor::begin_item called before calling MutCursor::end_item"
        );
        let key = self.key_from_loc(location);
        // nest != old_nest 表示正在插入，可以不需要查找相同的 item
        if self.nest == self.old_nest {
            if let Some((ix, old_item)) = self.tree.find_by_key(self.ix, key) {
                self.mutation.delete(ix - self.ix);
                self.ix = ix + 1;
                self.nest += 1;
                self.old_nest += 1;
                let id = old_item.id;
                self.current = Some(CurrentItem {
                    id,
                    key,
                    payload_mutation: PayloadMutation::Skip,
                });
                return false;
            }
        }

        // 正在插入
        self.nest += 1;
        let id = Id::new();
        self.current = Some(CurrentItem {
            id,
            key,
            payload_mutation: PayloadMutation::Insert(None),
        });
        return true;
    }

    pub fn end_item_and_begin_body(&mut self) {
        assert!(
            self.current.is_some(),
            "MutCursor::end_item called before calling MutCursor::begin_item"
        );

        let current = self.current.take().unwrap();
        match current.payload_mutation {
            PayloadMutation::Skip => {
                self.mutation.skip(1);
            }
            PayloadMutation::Insert(Some(body)) => {
                let item = Item {
                    key: current.key,
                    id: current.id,
                    body,
                };
                self.mutation.insert_one(Slot::Begin(item));
            }
            PayloadMutation::Update(body) => {
                let item = Item {
                    key: current.key,
                    id: current.id,
                    body,
                };
                self.mutation.update_one(Slot::Begin(item));
            }
            _ => panic!("MutCursor::end_item called on a new item without setting the payload"),
        };
    }

    pub fn end_body(&mut self) {
        if self.current.is_some() {
            panic!("MutCursor::end_body called before calling MutCursor::end_item_and_begin_body");
        }

        // updating
        if self.nest == self.old_nest {
            let n_trim = self.count_trim();
            self.ix += n_trim + 1;
            self.mutation.delete(n_trim);
            self.mutation.skip(1);
        } else {
            // inserting
            self.nest -= 1;
            self.mutation.insert_one(Slot::End);
        }
    }

    /// Reap the mutation.
    pub fn into_mutation(mut self) -> Mutation {
        let n_trim = self.tree.slots.len() - self.ix;
        self.mutation.delete(n_trim);
        self.mutation
    }
}
