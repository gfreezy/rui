pub use crate::{box_constraints::BoxConstraints, sliver_constraints::SliverConstraints};

#[derive(Clone, Debug, PartialEq)]
pub enum Constraints {
    BoxConstraints(BoxConstraints),
    SliverConstraints(SliverConstraints),
}

impl Constraints {
    pub fn to_box(&self) -> BoxConstraints {
        self.into()
    }

    pub fn to_sliver(&self) -> SliverConstraints {
        self.into()
    }

    pub(crate) fn is_tight(&self) -> bool {
        match self {
            Constraints::BoxConstraints(bc) => bc.is_tight(),
            Constraints::SliverConstraints(sc) => sc.is_tight(),
        }
    }
}

impl From<&Constraints> for BoxConstraints {
    fn from(c: &Constraints) -> Self {
        match c {
            Constraints::BoxConstraints(bc) => *bc,
            Constraints::SliverConstraints(_) => panic!("not box constrains"),
        }
    }
}

impl From<Constraints> for BoxConstraints {
    fn from(c: Constraints) -> Self {
        match c {
            Constraints::BoxConstraints(bc) => bc,
            Constraints::SliverConstraints(_) => panic!("not box constrains"),
        }
    }
}

impl From<&Constraints> for SliverConstraints {
    fn from(c: &Constraints) -> Self {
        match c {
            Constraints::BoxConstraints(_) => panic!("not sliver constrains"),
            Constraints::SliverConstraints(sc) => sc.clone(),
        }
    }
}

impl From<Constraints> for SliverConstraints {
    fn from(c: Constraints) -> Self {
        match c {
            Constraints::BoxConstraints(_) => panic!("not sliver constrains"),
            Constraints::SliverConstraints(sc) => sc,
        }
    }
}

impl From<&BoxConstraints> for Constraints {
    fn from(bc: &BoxConstraints) -> Self {
        Constraints::BoxConstraints(*bc)
    }
}

impl From<BoxConstraints> for Constraints {
    fn from(bc: BoxConstraints) -> Self {
        Constraints::BoxConstraints(bc)
    }
}

impl From<&SliverConstraints> for Constraints {
    fn from(sc: &SliverConstraints) -> Self {
        Constraints::SliverConstraints(sc.clone())
    }
}

impl From<SliverConstraints> for Constraints {
    fn from(sc: SliverConstraints) -> Self {
        Constraints::SliverConstraints(sc)
    }
}
