use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

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
        let lhs = (self.as_slice(), self.site_id(), self.clock());
        let rhs = (other.as_slice(), other.site_id(), other.clock());

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
        self.as_slice().hash(state)
    }
}

impl Deref for Position {
    type Target = [u32];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
