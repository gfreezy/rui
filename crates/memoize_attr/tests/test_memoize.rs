use derive_macro::memoize;

struct Ui;

#[memoize]
fn comp(ui: &mut Ui, count: usize) {
    let b = 2;
}
