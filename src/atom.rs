use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
    usize,
};

use once_cell::{self, sync::Lazy};
use std::ops::{Deref, DerefMut};

type BlockId = usize;

type VarId = usize;

#[derive(Debug)]
struct Block {
    id: BlockId,
    vars: HashSet<VarId>,
}

struct Tree {
    slots: Vec<BlockId>,
    blocks: HashMap<BlockId, Block>,
    block_counter: usize,
    var_counter: usize,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            slots: Vec::new(),
            blocks: HashMap::new(),
            var_counter: 0,
            block_counter: 0,
        }
    }

    pub fn begin(&mut self) {
        let block_id = self.new_block();
        self.blocks.insert(
            block_id,
            Block {
                id: block_id,
                vars: HashSet::new(),
            },
        );
        self.slots.push(block_id);
    }

    pub fn end(&mut self) {
        let _ = self.slots.pop();
    }

    pub fn current_block_id(&self) -> BlockId {
        *self.slots.last().unwrap()
    }

    pub fn track_var(&mut self, var_id: VarId) {
        let current_block_id = self.current_block_id();
        let current_block = self.blocks.get_mut(&current_block_id).unwrap();
        current_block.vars.insert(var_id);
    }

    pub fn new_var<T: Sized>(&mut self, default: T) -> Var<T> {
        let counter = self.var_counter;
        self.var_counter += 1;
        Var {
            id: counter,
            value: default,
        }
    }

    pub fn new_block(&mut self) -> BlockId {
        let c = self.block_counter;
        self.block_counter += 1;
        c
    }

    pub fn debug_tracks(&self) {
        dbg!(&self.blocks);
    }

    pub fn tracked_vars(&self) -> HashSet<usize> {
        let current_block_id = self.current_block_id();
        let current_block = self.blocks.get(&current_block_id).unwrap();
        current_block.vars.clone()
    }
}

static TREE: Lazy<Mutex<Tree>> = Lazy::new(|| Mutex::new(Tree::new()));

#[derive(Debug)]
pub struct Var<T: Sized> {
    id: usize,
    value: T,
}

impl<T: Sized> Var<T> {
    pub fn get(&self) -> &T {
        TREE.lock().unwrap().track_var(self.id);
        &self.value
    }

    pub fn set(&mut self, val: T) {
        self.value = val;
    }
}

impl<T> Deref for Var<T>
where
    T: Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Var<T>
where
    T: Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub fn new_var<T: Sized>(default: T) -> Var<T> {
    TREE.lock().unwrap().new_var(default)
}

fn block(blk: impl Fn()) {
    TREE.lock().unwrap().begin();
    blk();
    TREE.lock().unwrap().end();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_var() {
        block(|| {
            let mut counter = new_var(0);
            println!("counter before: {}", *counter);
            counter.set(5);
            println!("counter after: {}", *counter);
            block(|| {
                let mut counter = new_var(1);
                println!("counter before: {}", counter.get());
                counter.set(6);
                println!("counter after: {}", counter.get());
            });
            let mut counter = new_var(2);
            let a = *counter;
            *counter = 2;
        });
        TREE.lock().unwrap().debug_tracks();
    }

    fn todo() {
        flex(|| {
            button("check", || {});
            label("tomorrow");
        });
    }

    fn flex(children: impl FnMut()) {}

    fn button(text: &str, on_click: impl FnMut()) {}

    fn label(text: &str) {}
}
