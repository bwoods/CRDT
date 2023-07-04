use std::collections::BTreeMap;
use std::ops::Range;

pub enum Algorithm {
    /// The naive strategy: always choose the next position after p in (p, q).
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
    pub(crate) fn reduce(&mut self, range: Range<u32>, level: usize) -> Range<u32> {
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
                match choices.entry(level as u32).or_insert_with(fastrand::bool) {
                    true => Algorithm::BoundaryPlus(*limit).reduce(range, level),
                    false => Algorithm::BoundaryMinus(*limit).reduce(range, level),
                }
            }
            Algorithm::Random => Range {
                start: range.start + 1,
                end: range.end,
            },
            Algorithm::Boundary => Range {
                // There is an inefficiency in calling `generate` with `Algorithm::Boundary`
                // in that there is no point calling `rng.u32(p..q)` with a one value range
                start: range.start + 1,
                end: range.start + 2,
            },
        }
    }
}
