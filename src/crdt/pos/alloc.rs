//! Allocation Strategies

use std::collections::BTreeMap;
use std::ops::Range;

use itertools::Itertools;

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
        left: &'a [u32],
        right: &'a [u32],
    ) -> impl Iterator<Item = Builder> + 'a {
        let mut prefix = Builder::new();

        for i in 0.. {
            let lhs = *(left.get(i).unwrap_or(&u32::MIN));
            let rhs = *(right.get(i).unwrap_or(&u32::MAX));
            debug_assert!(lhs < rhs);

            let interval = rhs - lhs;
            if interval <= count {
                prefix.push(lhs);
                continue;
            }

            let range = self.algorithm.within(lhs..rhs, i);
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

pub enum Algorithm {
    /// The naive strategy: always choose the position after p in (p, q).
    Boundary,
    /// The Logoot strategy: Randomly choose a position between (p, q).
    ///
    /// Note that when inserting a batch of data (p, q) is split into
    /// evenly divided subranges and each element is inserted randomly
    /// into its own sub-range.
    Random,
    /// The 1<sup>st</sup> LSEQ strategy: Choose a position close to p in (p, q).
    BoundaryPlus(u32),
    /// The 2<sup>nd</sup> LSEQ strategy: Choose a position close to q in (p, q).
    BoundaryMinus(u32),
    /// The optimal LSEQ strategy: Randomly choose between using
    /// bounder+ and boundary- at each level. Once a decision is
    /// made for a level it is always used (at that level).
    Boundaries {
        limit: u32,
        choices: BTreeMap<u32, bool>,
    },
}

impl Algorithm {
    fn within(&mut self, range: Range<u32>, level: usize) -> Range<u32> {
        match self {
            Algorithm::BoundaryPlus(limit) => Range {
                start: range.start + 1,
                end: range.end.min(range.start + 1 + *limit),
            },
            Algorithm::BoundaryMinus(limit) => Range {
                start: (range.end.saturating_sub(*limit)).max(range.start + 1),
                end: range.end,
            },
            Algorithm::Boundaries { limit, choices } => {
                let choice = choices.entry(level as u32).or_insert_with(fastrand::bool);

                match choice {
                    true => Algorithm::BoundaryPlus(*limit).within(range, level),
                    false => Algorithm::BoundaryMinus(*limit).within(range, level),
                }
            }
            Algorithm::Random => Range {
                start: range.start + 1,
                end: range.end,
            },
            Algorithm::Boundary => Range {
                // There is an inefficiency in calling `generate` with Algorithm::Boundary
                // in that there is no point calling `rng.u32(p..q)` with a one value range
                start: range.start + 1,
                end: range.start + 2,
            },
        }
    }
}
