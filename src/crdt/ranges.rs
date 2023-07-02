use std::{ops::Bound::Unbounded, ops::RangeBounds};

use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

use super::{Position, Storage};

impl Storage {
    pub fn lines(
        &self,
        range: impl RangeBounds<Position>,
    ) -> impl Iterator<Item = (&Position, &Position)> {
        self.newlines.range(range).tuple_windows()
    }

    pub fn characters(
        &self,
        range: impl RangeBounds<Position>,
    ) -> impl Iterator<Item = (&Position, &char)> {
        // skip `Position::first()` as is it an `Exclusive` bound
        let skip = (range.start_bound() == Unbounded) as usize;

        self.characters.range(range).skip(skip)
    }

    pub fn graphemes<'a>(
        &'a self,
        range: impl RangeBounds<Position> + 'a,
    ) -> impl Iterator<Item = (&Position, &Position)> + 'a {
        GraphemeBoundary {
            iter: self.characters(range),
            string: Default::default(),
        }
        .tuple_windows()
    }
}

struct GraphemeBoundary<'a, Iter>
where
    Iter: Iterator<Item = (&'a Position, &'a char)>,
{
    iter: Iter,
    string: String,
}

impl<'a, Iter> Iterator for GraphemeBoundary<'a, Iter>
where
    Iter: Iterator<Item = (&'a Position, &'a char)>,
{
    type Item = &'a Position;

    fn next(&mut self) -> Option<Self::Item> {
        for (pos, ch) in self.iter.by_ref() {
            if self.string.is_empty() {
                self.string.push(*ch);
                return Some(pos); // return the first one
            }

            self.string.push(*ch);

            if let Some((_, (next, _))) = self.string.grapheme_indices(true).tuple_windows().next()
            {
                self.string.replace_range(..next, "");
                return Some(pos);
            }
        }

        None
    }
}

#[test]
fn grapheme_segmentation() {
    let string = "👧👧🏻👧🏼👧🏽👧🏾👧🏿";
    let _bytes = string.len(); // 44 utf-8 bytes

    let storage = Storage::dense(string).unwrap();
    let ours = storage
        .graphemes(..)
        .map(|(start, stop)| {
            let mut string = String::default();
            string.extend(storage.characters(start..stop).map(|(_, ch)| ch));
            string
        })
        .collect::<Vec<_>>();

    let theirs = string
        .grapheme_indices(true)
        .map(|(_, str)| str.to_owned())
        .collect::<Vec<_>>();

    assert_eq!(theirs, ours);
}
