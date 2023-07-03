use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;

pub(crate) use pos::path;
pub use pos::{Error, Position};
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
        self.newlines.extend(
            iter.by_ref()
                .filter(|(_, ch)| *ch == '\n')
                .map(|(pos, _)| pos),
        );
    }
}

impl Storage {
    /// The `clock` is incremented every insert to avoid the
    /// [ABA problem](https://en.wikipedia.org/wiki/ABA_problem)
    /// inherent in an insert-delete-insert at the same location.
    fn next_clock(&mut self) -> u16 {
        self.clock = u16::wrapping_add(self.clock, 1);
        self.clock
    }

    #[inline(always)]
    /// The preferred constructor for [`Storage`] objects.
    /// Uses [`sparse`] encoding of the positions for performance.
    fn try_from(str: impl AsRef<str>) -> Result<Self, Error> {
        Self::sparse(str.as_ref())
    }

    #[track_caller]
    #[inline(never)]
    /// Constructs a CRDT using the Logoot algorithm.
    /// In the future, the LSEQ algorithm may be adopted as well
    pub fn sparse(str: &str) -> Result<Self, Error> {
        if str.len() >= Position::last().path()[0] as usize {
            return Err(Error::StringTooLarge);
        }

        let mut new = Self::default();
        new.extend(
            std::iter::zip(
                path::generate(
                    str.len() as u32, // checked above
                    Position::first().path(),
                    Position::last().path(),
                ),
                str.chars(),
            )
            .map(|(path, ch)| (Position::new(0, 0, &path).unwrap(), ch)),
        );

        Ok(new)
    }

    #[track_caller]
    #[inline(never)]
    /// Constructs a CRDT using only level-1 paths, and with **no** space in between them.
    ///
    /// It exists primarily for benchmarking (showing how badly this mode scales) and
    /// for testing and debugging the implementation.
    ///
    /// Use [`Storage::try_from`] (or call [`Storage::sparse`] directly) instead.
    pub fn dense(str: &str) -> Result<Self, Error> {
        if str.len() >= Position::last().path()[0] as usize {
            return Err(Error::StringTooLarge);
        }

        let mut new = Self::default();
        new.extend(
            std::iter::zip(1.., str.chars())
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
            .map(|builder| Position::new(pos.site_id(), pos.clock(), &builder).unwrap())
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
