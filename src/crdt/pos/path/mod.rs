use super::{Position, Small, INLINE};

pub mod algorithm;
mod allocator;

pub(crate) type Builder = tinyvec::TinyVec<[u32; INLINE]>;

pub(crate) const FIRST: [u32; 1] = [0];
pub(crate) const LAST: [u32; 3] = [Position::end_bound(0), u32::MAX, u32::MAX];

impl Position {
    #[inline]
    pub(crate) fn first() -> Position {
        Position {
            small: Default::default(),
        }
    }

    #[inline]
    pub(crate) fn last() -> Position {
        Position {
            small: Small {
                path: LAST,
                ..Default::default()
            },
        }
    }

    pub(crate) const fn end_bound(level: usize) -> u32 {
        match level {
            0 => {
                // • `0xfe` because 0xff is used as a tag and the tag overlaps level zero in the union
                0xfffffffe_u32.to_be()
            }
            _ => u32::MAX,
        }
    }
}
