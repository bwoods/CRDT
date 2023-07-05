use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;

pub use path::algorithm::Strategy;
pub use pos::path;
pub use pos::{Algorithm, Position};
pub use ranges::*;

use super::path::Builder;

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
                .unwrap()
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
        let mut new = Self::default();
        new.algorithm = Algorithm::with_strategy(strategy);

        new
    }

    pub fn with_seed(seed: u64) -> Self {
        let mut new = Self::default();
        new.algorithm = Algorithm::with_seed(seed);

        new
    }

    #[inline(always)]
    fn from(str: impl AsRef<str>) -> Self {
        Self::from_iter(str.as_ref().chars())
    }

    #[track_caller]
    #[inline(never)]
    pub fn insert_unchecked(&mut self, ch: char, pos: &Position) {
        let (right, left) = self
            .characters
            .range(..=pos)
            .rev() // grab `pos` and its predecessor
            .map(|(pos, _)| pos)
            .tuple_windows()
            .next()
            .unwrap();

        let path = self.algorithm.generate_one(left.path(), right.path());
        let pos = Position::new(pos.site_id(), pos.clock(), &path);

        self.characters.insert(pos, ch);
    }

    #[inline(never)]
    pub fn remove(&mut self, pos: &Position) -> Option<char> {
        self.characters.remove(pos).map(|ch| {
            if ch == '\n' {
                let check = self.newlines.remove(pos);
                debug_assert!(check, "A newline was missing in newlines?");
            };

            ch
        })
    }

    /// The `clock` is incremented every insert to avoid the
    /// [ABA problem](https://en.wikipedia.org/wiki/ABA_problem)
    /// inherent in an insert-delete-insert at the same location.
    fn next_clock(&mut self) -> u16 {
        self.clock = u16::wrapping_add(self.clock, 1);
        self.clock
    }
}

#[test]
#[ignore]
fn invalid_insert() {
    let mut storage = crate::Storage::with_strategy(Strategy::Boundary);

    let str = "abc";
    storage.extend(str.chars());

    // create a gap
    let pos = Position::new(0, storage.clock, &[5]);
    storage.characters.insert(pos, 'e');

    println!("btree: {:?}", &storage.characters);

    // insert before a non-existent key
    let pos = Position::new(0, 0, &[4]);
    storage.insert_unchecked('d', &pos);

    println!("btree: {:?}", &storage.characters);
}
