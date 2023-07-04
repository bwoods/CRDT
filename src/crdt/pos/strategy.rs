//! Allocation Strategies

use itertools::Itertools;

use super::algorithm::Algorithm;
use super::path::Builder;

pub struct Strategy {
    rng: fastrand::Rng,
    algorithm: Algorithm,
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy {
            rng: Default::default(),
            algorithm: Algorithm::Boundaries {
                choices: Default::default(),
                limit: 1024,
            },
        }
    }
}

impl Strategy {
    pub fn with_seed(seed: u64) -> Strategy {
        Strategy {
            rng: fastrand::Rng::with_seed(seed),
            ..Default::default()
        }
    }

    #[inline(never)]
    /// Generate `count` paths between the given `left` and `right` boundaries.
    pub(crate) fn generate<'a>(
        &'a mut self,
        count: u32,
        left: &[u32],
        right: &[u32],
    ) -> impl Iterator<Item = Builder> + 'a {
        let mut prefix = Builder::new();

        for level in 0.. {
            let lhs = *(left.get(level).unwrap_or(&u32::MIN));
            let rhs = *(right.get(level).unwrap_or(&u32::MAX));
            debug_assert!(lhs < rhs);

            let interval = rhs - lhs;
            if interval <= count {
                prefix.push(lhs);
                continue;
            }

            let range = self.algorithm.reduce(lhs..rhs, level);
            return range
                .step_by((interval / count) as usize)
                .tuple_windows()
                .map(move |(p, q)| {
                    let mut path = prefix.clone();
                    let val = self.rng.u32(p..q);
                    path.push(val);
                    path
                });
        }

        unreachable!()
    }
}
