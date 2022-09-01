use crate::{
    object::AnyParentData,
    physics::tolerance::{near_equal, Tolerance},
};

#[derive(Debug, Default, Clone)]
pub struct SliverListParentData {
    pub(crate) keep_alive: bool,
    pub(crate) layout_offset: Option<f64>,
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
        let layout_offset_equal = match (self.layout_offset, other.layout_offset) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(l), Some(r)) => near_equal(l, r, Tolerance::DEFAULT.distance),
        };
        self.keep_alive == other.keep_alive && self.index == other.index && layout_offset_equal
    }
}
