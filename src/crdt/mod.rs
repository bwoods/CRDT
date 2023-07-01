use std::collections::{BTreeMap, BTreeSet};

use tinyvec;

pub use pos::Position;

mod pos;

struct Storage {
    characters: BTreeMap<Position, char>, // TODO: Graphemes, not `char`s
    newlines: BTreeSet<Position>,
    strategy: tinyvec::TinyVec<[bool; 6]>,
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
            strategy: Default::default(),
        }
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn size() {
    let len = std::mem::size_of::<tinyvec::TinyVec<[bool; 6]>>();
    assert_eq!(len, 24);

    let len = std::mem::size_of::<tinyvec::TinyVec<[bool; 7]>>();
    assert_eq!(len, 32);
}
