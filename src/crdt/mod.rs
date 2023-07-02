use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;

pub use pos::{Error, Position};

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

impl Storage {
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

        let pos = Position::between(left.path(), right.path())
            .map(|builder| {
                Position::new(
                    0,
                    {
                        self.clock = u16::wrapping_add(self.clock, 1);
                        self.clock
                    },
                    &builder,
                )
                .unwrap()
            })
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
