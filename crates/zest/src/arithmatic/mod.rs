pub struct Tolerance {
    pub distance: f64,
    pub time: f64,
    pub velocity: f64,
}

const EPSILON_DEFAULT: f64 = 1e-3;

impl Tolerance {
    pub const DEFAULT: Tolerance = Tolerance {
        distance: EPSILON_DEFAULT,
        time: EPSILON_DEFAULT,
        velocity: EPSILON_DEFAULT,
    };
}

/// Whether two doubles are within a given distance of each other.
///
/// The `epsilon` argument must be positive and not null.
/// The `a` and `b` arguments may be null. A null value is only considered
/// near-equal to another null value.
pub fn near_equal(a: f64, b: f64, epsilon: f64) -> bool {
    assert!(epsilon >= 0.0);
    (a > (b - epsilon)) && (a < (b + epsilon)) || a == b
}

pub fn default_near_equal(a: f64, b: f64) -> bool {
    near_equal(a, b, EPSILON_DEFAULT)
}
