use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;

pub use pos::{path, Error, Position};
pub use ranges::*;

mod pos;
mod ranges;

struct Storage {
    characters: BTreeMap<Position, char>,
    newlines: BTreeSet<Position>,
    clock: u16,
}

impl Default for Storage {
    fn default() -> Self {
        let mut characters = BTreeMap::<Position, char>::default();
        characters.insert(Position::first(), '\0');
        characters.insert(Position::last(), '\0');

        let mut newlines = BTreeSet::<Position>::default();
        newlines.insert(Position::first());
        newlines.insert(Position::last());

        Storage {
            characters,
            newlines,
            clock: 0,
        }
    }
}

impl Extend<(Position, char)> for Storage {
    fn extend<I: IntoIterator<Item = (Position, char)>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();

        self.characters.extend(iter.by_ref());
        self.newlines.extend(iter.by_ref().filter_map(
            |(pos, ch)| {
                if ch == '\n' {
                    Some(pos)
                } else {
                    None
                }
            },
        ));
    }
}

impl Storage {
    fn next_clock(&mut self) -> u16 {
        self.clock = u16::wrapping_add(self.clock, 1);
        self.clock
    }

    #[track_caller]
    #[inline(never)]
    pub fn sparse(string: &str) -> Result<Self, Error> {
        if string.len() >= u32::MAX as usize {
            return Err(Error::StringTooLarge);
        }

        let mut new = Self::default();
        new.extend(
            std::iter::zip(
                path::generate(
                    string.len() as u32, // checked above
                    Position::first().path(),
                    Position::last().path(),
                ),
                string.chars(),
            )
            .map(|(path, ch)| (Position::new(0, 0, &path).unwrap(), ch)),
        );

        Ok(new)
    }

    #[track_caller]
    #[inline(never)]
    pub fn dense(string: &str) -> Result<Self, Error> {
        if string.len() >= Position::last().path()[0] as usize {
            return Err(Error::StringTooLarge);
        }

        let mut new = Self::default();
        new.extend(
            std::iter::zip(1.., string.chars())
                .map(|(n, ch)| (Position::new(0, 0, &[n]).unwrap(), ch)),
        );

        Ok(new)
    }

    #[track_caller]
    #[inline(never)]
    pub fn insert(&mut self, ch: char, pos: &Position) {
        let (right, left) = self
            .characters
            .range(..=pos)
            .rev() // grab `pos` and its predecessor
            .map(|(pos, _)| pos)
            .tuple_windows()
            .next()
            .unwrap();

        let pos = path::between(left.path(), right.path())
            .map(|builder| Position::new(0, self.next_clock(), &builder).unwrap())
            .unwrap();

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
}
