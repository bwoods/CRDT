use itertools::Itertools;

use super::{Position, Small, INLINE};

type Builder = tinyvec::TinyVec<[u32; INLINE]>;

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
                path: [
                    0xfffffffe_u32.to_be(), // `0xfe` because 0xff is used as a tag
                    0,                      // and the tag overlaps it in the union
                    0,
                ],
                ..Default::default()
            },
        }
    }

    /// Generate a path between the given `left` and `right` boundaries.
    pub(crate) fn between(left: &[u32], right: &[u32]) -> Option<Builder> {
        Self::generate(1, left, right).next()
    }

    #[inline(never)]
    /// Generate `count` paths between the given `left` and `right` boundaries.
    pub(crate) fn generate(
        count: u32,
        left: &[u32],
        right: &[u32],
    ) -> impl Iterator<Item = Builder> {
        let mut prefix = Builder::new();
        let mut rng = fastrand::Rng::new();

        for i in 0.. {
            let lhs = *(left.get(i).unwrap_or(&u32::MIN));
            let rhs = *(right.get(i).unwrap_or(&u32::MAX));
            debug_assert!(lhs < rhs);

            let interval = rhs - lhs;
            if interval <= count {
                prefix.push(lhs);
                continue;
            }

            return ((lhs + 1)..rhs)
                .step_by((interval / count) as usize)
                .tuple_windows()
                .map(move |(p, q)| {
                    let mut path = prefix.clone();
                    path.push(rng.u32(p..q));
                    path
                });
        }

        unreachable!()
    }
}
