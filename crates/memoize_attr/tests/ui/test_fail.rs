use memoize_attr::memoize;

pub struct Ui;

impl Ui {
    pub fn memoize<T, F>(&mut self, _t: T, _f: F) {}
}

#[memoize]
fn comp(_count: usize) {}

fn main() {
    comp(1);
}
