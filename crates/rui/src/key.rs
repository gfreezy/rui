//! Unique keys and key paths.

use std::hash::Hash;
use std::panic::Location;

pub type LocalKey = String;

/// A unique call location.
///
/// These come from `#[track_caller]` annotations. It is a newtype
/// so we can use it as a key in various contexts; the traits we
/// want are not implemented on the inner type.
#[derive(Clone, Copy, Debug)]
pub struct Key(&'static Location<'static>);

impl Key {
    /// The pointer to the location metadata
    ///
    /// Unique locations are expected to have unique pointers. This
    /// is perhaps not formally guaranteed by the language spec, but
    /// it's hard to imagine how it can be implemented otherwise.
    fn as_ptr(&self) -> *const Location<'static> {
        self.0
    }

    #[track_caller]
    pub fn current() -> Self {
        Location::caller().into()
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Key) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl Eq for Key {}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state)
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ptr().partial_cmp(&other.as_ptr())
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ptr().cmp(&other.as_ptr())
    }
}

impl From<&'static Location<'static>> for Key {
    fn from(inner: &'static Location<'static>) -> Self {
        Key(inner)
    }
}

impl From<Key> for (Key, LocalKey) {
    fn from(key: Key) -> Self {
        (key, String::new())
    }
}
