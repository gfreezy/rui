use memoize_attr::memoize;

pub struct Ui;

impl Ui {
    pub fn memoize<T, F>(&mut self, _t: T, _f: F) {}
}

#[memoize]
fn comp(ui: &mut Ui, _count: usize) {}

#[memoize]
fn comp2(ui: &mut Ui) {}

fn main() {
    comp(&mut Ui, 1);
    comp2(&mut Ui);
}
