use super::allocator::Allocator;
use super::Builder;
use super::Position;

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

    pub(crate) fn generate_one<'a>(&'a mut self, left: &'a [u32], right: &'a [u32]) -> Builder {
        self.generate(left, right).next().unwrap()
    }

    /// Creates an iterator that generates paths between the given `left` and `right` boundaries.
    pub(crate) fn generate<'a>(
        &'a mut self,
        left: &'a [u32],
        right: &'a [u32],
    ) -> impl Iterator<Item = Builder> + 'a {
        let mut prefix = Builder::new();
        let mut level = 0; // the level of the path being built
        let mut index = 0; // how many steps we are into that level

        let step = self.allocator.step_size();
        let mut prev = None;

        std::iter::repeat_with(move || loop {
            let mut lhs = *(left.get(level).unwrap_or(&u32::MIN)) as usize;
            let rhs = *(right.get(level).unwrap_or(&u32::MAX)) as usize;
            debug_assert!(lhs < rhs);

            lhs += step * index;
            if rhs - lhs < 2 {
                level += 1;
                index = 0;

                prefix.push(prev.unwrap_or(lhs as u32));
                continue;
            }

            let p = lhs as u32 + 1;
            let q = rhs as u32;

            let range = self.allocator.reduce_range(p..q, level, &mut self.rng);
            let val = self.rng.u32(range);
            prev = val.into();

            let mut path = prefix.clone();
            path.push(val);
            index += 1;

            return path;
        })
    }
}

#[test]
#[ignore]
fn exhausting_level_zero() {
    let mut storage = crate::Storage::with_strategy(Strategy::Boundary);

    // place a letter near the end of level zero
    let pos = Position::new(0, 0, &[Position::end_bound(0) - 2]);
    storage.characters.insert(pos, '0');

    // now add more characters than fit in the remaining space
    let string = "abcdefg";
    storage.extend(string.chars());

    for ch in storage.characters(..) {
        println!("{:?} {:?}", ch.0, ch.1);
    }
}
