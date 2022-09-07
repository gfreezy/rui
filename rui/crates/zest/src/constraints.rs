mod box_constraints;
mod sliver_constraints;

pub use box_constraints::BoxConstraints;
pub use sliver_constraints::{SliverConstraints, SliverGeometry};

#[derive(Debug, Clone, PartialEq)]
pub enum Constraints {
    BoxConstraints(BoxConstraints),
    SliverConstraints(SliverConstraints),
}

impl Constraints {
    pub fn is_tight(&self) -> bool {
        self.box_constraints().is_tight()
    }

    pub fn box_constraints(&self) -> BoxConstraints {
        match self {
            Constraints::BoxConstraints(constraints) => constraints.clone(),
            _ => unreachable!(),
        }
    }

    pub fn sliver_constraints(&self) -> SliverConstraints {
        match self {
            Constraints::SliverConstraints(constraints) => constraints.clone(),
            _ => unreachable!(),
        }
    }
}
