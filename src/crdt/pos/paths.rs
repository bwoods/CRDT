use itertools::Itertools;
use tinyvec::TinyVec;

use super::{Position, Small, INLINE};

impl Position {
    pub(crate) fn first() -> Position {
        Position {
            small: Default::default(),
        }
    }

    pub(crate) fn last() -> Position {
        Position {
            small: Small {
                path: [
                    0xfffffffe_u32.to_le(), // `0xfe` because 0xff is used as a tag
                    0,                      // and the tag overlaps it in the union
                    0,
                ],
                ..Default::default()
            },
        }
    }

    /// Generates `count` paths between the given `left` and `right` boundaries.
    pub(crate) fn generate(
        count: u32,
        left: &[u32],
        right: &[u32],
    ) -> impl Iterator<Item = TinyVec<[u32; INLINE]>> {
        let mut prefix = TinyVec::<[u32; INLINE]>::new();
        let mut rng = fastrand::Rng::new();

        for i in 0.. {
            let lhs = *(left.get(i).unwrap_or(&u32::MIN));
            let rhs = *(right.get(i).unwrap_or(&u32::MAX));
            debug_assert!(lhs < rhs);

            let interval = rhs - lhs;
            if interval < count {
                prefix.push(lhs);
                continue;
            }

            return (lhs..rhs)
                .step_by((interval / count) as usize)
                .tuple_windows()
                .map(move |(p, q)| {
                    let mut path = TinyVec::with_capacity(prefix.len() + 1);
                    path.extend_from_slice(&prefix);

                    debug_assert_eq!(path.capacity(), prefix.len() + 1);
                    path.push(rng.u32((p + 1)..q));
                    path
                });
        }

        unreachable!()
    }
}
