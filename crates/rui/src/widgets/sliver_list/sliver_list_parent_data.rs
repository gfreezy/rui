use crate::{
    object::AnyParentData,
    physics::tolerance::{near_equal, Tolerance},
};

#[derive(Debug)]
pub struct SliverListParentData {
    pub(crate) keep_alive: bool,
    pub(crate) kept_alive: bool,
    pub(crate) layout_offset: f64,
    pub(crate) index: usize,
}

impl AnyParentData for SliverListParentData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_any_box(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn eql(&self, other: &dyn AnyParentData) -> bool {
        let other = other
            .as_any()
            .downcast_ref::<SliverListParentData>()
            .unwrap();
        self.keep_alive == other.keep_alive
            && self.kept_alive == other.kept_alive
            && self.index == other.index
            && near_equal(
                self.layout_offset,
                other.layout_offset,
                Tolerance::DEFAULT.distance,
            )
    }
}
