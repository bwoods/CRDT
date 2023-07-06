use itertools::{diff_with, Diff};

use super::allocator::Allocator;
use super::Builder;

pub enum Strategy {
    /// The naive strategy: always choose the next position after p in (p, q).
    Boundary,
    /// The 1<sup>st</sup> LSEQ strategy: Choose a position close to p in (p, q).
    BoundaryPlus(u32),
    /// The 2<sup>nd</sup> LSEQ strategy: Choose a position close to q in (p, q).
    BoundaryMinus(u32),
    /// The optimal LSEQ strategy: Randomly choose between using
    /// bounder+ and boundary- at each level. Once a decision is
    /// made for a level it is always used (at that level).
    Boundaries(u32),
}

pub struct Algorithm {
    rng: fastrand::Rng,
    allocator: Allocator,
}

impl Default for Algorithm {
    fn default() -> Self {
        Self::with_strategy(Strategy::Boundaries(1024))
    }
}

impl Algorithm {
    pub fn with_strategy(strategy: Strategy) -> Algorithm {
        let algorithm = match strategy {
            Strategy::Boundary => Allocator::BoundaryPlus(1),
            Strategy::BoundaryPlus(limit) => Allocator::BoundaryPlus(limit),
            Strategy::BoundaryMinus(limit) => Allocator::BoundaryMinus(limit),
            Strategy::Boundaries(limit) => Allocator::Boundaries {
                choices: Default::default(),
                limit,
            },
        };

        Algorithm {
            allocator: algorithm,
            rng: Default::default(),
        }
    }

    pub fn with_seed(seed: u64) -> Algorithm {
        let mut new = Self::default();
        new.rng.seed(seed);
        new
    }

    /// Creates an iterator that generates paths between the given `left` and `right` boundaries.
    pub(crate) fn generate<'a>(
        &'a mut self,
        left: &'a [u32],
        right: &'a [u32],
    ) -> impl Iterator<Item = Builder> + 'a {
        let mut path = Builder::from(left);
        let mut level = if let Some(diff) = //
            diff_with(left, right, |p, q| *p == *q)
        {
            match diff {
                Diff::FirstMismatch(level, ..) => level,
                Diff::Longer(level, ..) => level,
                Diff::Shorter(..) => unreachable!(), // `left` can’t match AND be longer
            }
        } else {
            left.len()
        };

        std::iter::repeat_with(move || {
            path = self.alloc(&path, right, &mut level);
            return path.clone();
        })
    }

    #[inline]
    /// Generates a path between the given `left` and `right` boundaries.
    pub fn generate_one<'a>(&'a mut self, left: &'a [u32], right: &'a [u32]) -> Builder {
        // SAFETY: `generate()` will always return a value
        self.generate(left, right).next().unwrap()
    }

    ///
    fn alloc(&mut self, left: &[u32], right: &[u32], level: &mut usize) -> Builder {
        loop {
            let lhs = 1 + *(left.get(*level).unwrap_or(&u32::MIN));
            let rhs = *(right.get(*level).unwrap_or(&u32::MAX));

            if rhs - lhs < 1 {
                *level += 1;
                continue;
            }

            let range = self.allocator.reduce_range(lhs..rhs, *level, &mut self.rng);
            let val = self.rng.u32(range);

            let mut path = Builder::from(&left[..*level]);
            path.push(val);
            return path;
        }
    }
}

#[test]
fn exhausting_level_zero() {
    use super::Position;

    let mut storage = crate::Storage::with_strategy(Strategy::Boundary);

    // place a letter near the end of level zero
    let pos = Position::new(0, 0, &[Position::end_bound(0) - 2]);
    storage.characters.insert(pos, '0');

    // now add more characters than fit in the remaining space
    let string = "abcdefg";
    storage.extend(string.chars());

    // for ch in storage.characters(..) {
    //     println!("{:?} {:?}", ch.0, ch.1);
    // }
}
