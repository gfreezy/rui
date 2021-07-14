use crate::id::Id;
use crate::tree::mut_cursor::MutCursor;
use crate::tree::{Payload, Tree};
use crate::view::AnyView;
use std::panic::Location;

pub struct Cx<'a> {
    mut_cursor: MutCursor<'a>,
}

impl<'a> Cx<'a> {
    /// Only public for experimentation.
    pub fn new(tree: &'a Tree) -> Cx<'a> {
        let mut_cursor = MutCursor::new(tree);
        Cx { mut_cursor }
    }

    /// Add a view as a leaf.
    ///
    /// This method is expected to be called mostly by the `build`
    /// methods on `View` implementors.
    pub fn leaf_view(&mut self, view: AnyView, loc: &'static Location) -> Id {
        let id = self.begin_view(view, loc);
        self.end_view();
        id
    }

    /// Begin a view element.
    ///
    /// This method is expected to be called mostly by the `build`
    /// methods on `View` implementors.
    ///
    /// The API may change to return a child cx.
    pub fn begin_view(&mut self, view: AnyView, loc: &'static Location) -> Id {
        let body = Payload::View(view);
        let is_new = self.mut_cursor.begin_item_at(loc);
        let id = self.mut_cursor.get_current_id();
        if is_new {
            self.mut_cursor.set_current_payload(body);
        } else if Some(&body) != self.mut_cursor.get_current_payload() {
            self.mut_cursor.set_current_payload(body);
        }
        self.mut_cursor.end_item_and_begin_body();
        id
    }

    pub fn end_view(&mut self) {
        self.mut_cursor.end_body();
    }
}
