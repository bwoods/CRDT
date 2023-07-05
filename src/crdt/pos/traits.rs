use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use super::Position;

impl PartialOrd for Position {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // `clock` must be included for `BtreeMap::remove()` to work properly
        let lhs = (self.path(), self.site_id(), self.clock());
        let rhs = (other.path(), other.site_id(), other.clock());

        lhs.cmp(&rhs)
    }
}

impl PartialEq for Position {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Position {}

impl Hash for Position {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path().hash(state)
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("site", &self.site_id())
            .field("clock", &self.clock())
            .field("path", &self.path())
            .finish()
    }
}
