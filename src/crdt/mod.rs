use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;

use pos::path::{self, algorithm::Algorithm, Builder};
pub use pos::{path::algorithm::Strategy, Position};
pub use ranges::*;

mod pos;
mod ranges;

pub struct Storage {
    characters: BTreeMap<Position, char>,
    newlines: BTreeSet<Position>,
    algorithm: Algorithm,
    clock: u16,
    site: u16,
}

impl Default for Storage {
    fn default() -> Self {
        let mut characters = BTreeMap::<Position, char>::default();
        characters.insert(Position::first(), '\u{2402}');
        characters.insert(Position::last(), '\u{2403}');

        let mut newlines = BTreeSet::<Position>::default();
        newlines.insert(Position::first());
        newlines.insert(Position::last());

        Storage {
            characters,
            newlines,
            algorithm: Default::default(),
            clock: Default::default(),
            site: Default::default(),
        }
    }
}

impl FromIterator<char> for Storage {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut new = Self::default();

        let chars = iter.into_iter();
        let positions = new
            .algorithm
            .generate(&path::FIRST, &path::LAST)
            .map(|path| (Position::new(new.site, new.clock, &path)));

        let mut iter = positions.zip(chars);
        new.characters.extend(iter.by_ref());
        new.newlines
            .extend(iter.filter(|(_, ch)| *ch == '\n').map(|(pos, _)| pos));

        new
    }
}

impl Extend<char> for Storage {
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        let chars = iter.into_iter();
        let left = Builder::from_iter(
            self.characters
                .range(..)
                .rev()
                .skip(1) // skip `Position::last()` as is it an `Exclusive` bound
                .map(|(pos, _)| pos.path())
                .next()
                .unwrap() // SAFETY: iterator will always have `Position::first()`
                .iter()
                .cloned(),
        );

        let clock = self.next_clock();
        let positions = self
            .algorithm
            .generate(&left, &path::LAST)
            .map(|path| (Position::new(self.site, clock, &path)));

        let mut iter = positions.zip(chars);
        self.characters.extend(iter.by_ref());
        self.newlines
            .extend(iter.filter(|(_, ch)| *ch == '\n').map(|(pos, _)| pos));
    }
}

impl Storage {
    pub fn with_strategy(strategy: Strategy) -> Self {
        Storage {
            algorithm: Algorithm::with_strategy(strategy),
            ..Default::default()
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Storage {
            algorithm: Algorithm::with_seed(seed),
            ..Default::default()
        }
    }

    #[inline(always)]
    fn from(str: impl AsRef<str>) -> Self {
        Self::from_iter(str.as_ref().chars())
    }

    #[must_use]
    pub fn insert(&mut self, ch: char, before: &Position) -> bool {
        if let Some((right, left)) = self
            .characters
            .range(..=before)
            .rev() // grab `pos` and its predecessor
            .map(|(pos, _)| pos)
            .tuple_windows()
            .next()
        {
            if right == before {
                let path = self.algorithm.generate_one(left.path(), right.path());
                let pos = Position::new(self.site, self.next_clock(), &path);

                return match self.characters.entry(pos) {
                    Entry::Occupied(_) => false, // CRDTs do not replace values; positions must remain unique
                    Entry::Vacant(entry) => {
                        entry.insert(ch);
                        true
                    }
                };
            }
        }

        false
    }

    pub fn remove(&mut self, pos: &Position) -> bool {
        self.characters
            .remove(pos)
            .filter(|ch| *ch == '\n')
            .map(|_| self.newlines.remove(pos))
            .unwrap_or(false)
    }

    #[inline]
    /// The `clock` is incremented every insert to avoid the
    /// [ABA problem](https://en.wikipedia.org/wiki/ABA_problem)
    /// inherent in an insert-delete-insert at the same location.
    fn next_clock(&mut self) -> u16 {
        self.clock = u16::wrapping_add(self.clock, 1);
        self.clock
    }
}

#[test]
fn invalid_insert_position() {
    let mut storage = crate::Storage::with_strategy(Strategy::Boundary);

    // inserting before `Position::first()` always fails
    assert_eq!(storage.insert('d', &Position::first()), false);

    let str = "abc";
    storage.extend(str.chars());

    // Note, that even with a gap between keys…
    let pos = Position::new(0, storage.clock, &[5]);
    storage.characters.insert(pos, 'e');

    // attempting to insert before a non-existent key fails…
    let pos = Position::new(0, storage.clock, &[4]);
    assert_eq!(storage.insert('d', &pos), false);

    // while using the appropriate key works.
    let pos = Position::new(0, storage.clock, &[5]);
    assert_eq!(storage.insert('d', &pos), true);

    let string = storage.string(..);
    assert_eq!(string, "abcde");
}
