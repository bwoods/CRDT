use std::mem::size_of;

use quickcheck_macros::quickcheck;

use super::*;

#[test]
fn assumptions() {
    assert_eq!(size_of::<Position>(), 16);

    let first = Position::first();
    let last = Position::last();

    assert!(first < last);

    assert!(first.is_inline());
    assert!(last.is_inline());
}

#[test]
fn boundaries() {
    let keys = [1, 2, 3, 4];

    let position = Position::new(0, 0, &keys[0..3]);
    assert!(position.is_inline());

    let position = Position::new(0, 0, &keys[0..4]);
    assert!(position.is_heap());
}

#[test]
fn layout() {
    let valid = Position::new(0, 0, &[0xff]);
    assert!(valid.is_inline());

    let invalid = Position::new(0, 0, &[0xffffffff]);
    assert!(invalid.is_heap()); // this is why this path MUST never be generated!

    // …and this is why it never will be.
    assert!(unsafe { Position::last().small.path[0] < invalid.small.path[0] });

    // we can't even `Drop` it correctly
    std::mem::forget(invalid);
}

#[quickcheck]
fn property_testing(site: u16, clock: u16, nums: Vec<std::num::NonZeroU32>) {
    let nums: Vec<_> = nums.iter().map(|n| n.get()).collect();

    let position = Position::new(site, clock, &nums);

    // small positions will be zero-padded; remove them before we compare
    let result = &position.path()[..position.level()];

    assert_eq!(&nums, result);
}
