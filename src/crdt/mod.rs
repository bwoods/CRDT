use std::collections::{BTreeMap, BTreeSet};
use tinyvec;

mod pos;
pub use pos::Position;

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
fn size() {
    let len = std::mem::size_of::<tinyvec::TinyVec<[bool; 6]>>();
    assert_eq!(len, 24);

    let len = std::mem::size_of::<tinyvec::TinyVec<[bool; 7]>>();
    assert_eq!(len, 32);
}
