use decorum::R64;

use crate::constraints::Constraints;
use crate::geometry::Size;

#[derive(Clone, Debug)]
pub struct BoxConstraints {
    pub min_width: f64,
    pub max_width: f64,
    pub min_height: f64,
    pub max_height: f64,
}

impl PartialEq for BoxConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.min_width == other.min_width
            && self.max_width == other.max_width
            && self.min_height == other.min_height
            && self.max_height == other.max_height
    }
}

impl Eq for BoxConstraints {}

impl std::hash::Hash for BoxConstraints {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        R64::from(self.min_width).hash(state);
        R64::from(self.max_width).hash(state);
        R64::from(self.min_height).hash(state);
        R64::from(self.max_height).hash(state);
    }
}

impl From<BoxConstraints> for Constraints {
    fn from(bc: BoxConstraints) -> Self {
        Constraints::BoxConstraints(bc)
    }
}

impl BoxConstraints {
    pub fn new(min_width: f64, max_width: f64, min_height: f64, max_height: f64) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    /// An unbounded box constraints object.
    ///
    /// Can be satisfied by any nonnegative size.
    pub const UNBOUNDED: BoxConstraints = BoxConstraints {
        min_width: 0.,
        min_height: 0.,
        max_width: f64::INFINITY,
        max_height: f64::INFINITY,
    };

    pub fn has_tight_width(&self) -> bool {
        self.min_width >= self.max_width
    }

    pub fn has_tight_height(&self) -> bool {
        self.min_height >= self.max_height
    }

    pub(crate) fn is_tight(&self) -> bool {
        self.has_tight_width() && self.has_tight_height()
    }

    pub(crate) fn tight(size: Size) -> Self {
        BoxConstraints {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    /// Create a "tight" box constraints object for one or more dimensions.
    ///
    /// [rounded away from zero]: struct.Size.html#method.expand
    pub fn tight_for(width: Option<f64>, height: Option<f64>) -> BoxConstraints {
        match (width, height) {
            (None, None) => BoxConstraints::UNBOUNDED,
            (None, Some(h)) => BoxConstraints {
                min_height: h,
                max_height: h,
                ..BoxConstraints::UNBOUNDED
            },
            (Some(w), None) => BoxConstraints {
                min_width: w,
                max_width: w,
                ..BoxConstraints::UNBOUNDED
            },
            (Some(w), Some(h)) => BoxConstraints {
                min_width: w,
                max_width: w,
                min_height: h,
                max_height: h,
            },
        }
    }

    pub(crate) fn constrain(&self, cross_size: Size) -> Size {
        Size::new(
            self.constrain_width(cross_size.width),
            self.constrain_height(cross_size.height),
        )
    }

    fn constrain_width(&self, width: f64) -> f64 {
        if width < self.min_width {
            self.min_width
        } else if width > self.max_width {
            self.max_width
        } else {
            width
        }
    }

    fn constrain_height(&self, height: f64) -> f64 {
        if height < self.min_height {
            self.min_height
        } else if height > self.max_height {
            self.max_height
        } else {
            height
        }
    }
}
