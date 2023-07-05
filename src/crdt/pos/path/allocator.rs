use std::collections::BTreeMap;
use std::ops::Range;

use fastrand::Rng;

pub(crate) enum Allocator {
    BoundaryPlus(u32),
    BoundaryMinus(u32),
    Boundaries {
        limit: u32,
        choices: BTreeMap<u32, bool>,
    },
}

impl Allocator {
    pub(crate) fn step_size(&self) -> usize {
        match self {
            Allocator::BoundaryPlus(limit) => *limit as usize,
            Allocator::BoundaryMinus(limit) => *limit as usize,
            Allocator::Boundaries { limit, .. } => *limit as usize,
        }
    }

    pub(crate) fn reduce_range(
        &mut self,
        range: Range<u32>,
        level: usize,
        rng: &mut Rng,
    ) -> Range<u32> {
        match self {
            Allocator::BoundaryPlus(limit) => Range {
                start: range.start,
                end: range.end.min(range.start + *limit),
            },
            Allocator::BoundaryMinus(limit) => Range {
                start: (range.end.saturating_sub(*limit)).max(range.start),
                end: range.end,
            },
            Allocator::Boundaries { limit, choices } => {
                match choices.entry(level as u32).or_insert_with(|| rng.bool()) {
                    true => Allocator::BoundaryPlus(*limit).reduce_range(range, level, rng),
                    false => Allocator::BoundaryMinus(*limit).reduce_range(range, level, rng),
                }
            }
        }
    }
}
