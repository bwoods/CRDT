use std::mem::size_of;
use std::num::TryFromIntError;

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
fn boundary_size() -> Result<(), TryFromIntError> {
    let keys = [1, 2, 3, 4];

    let position = Position::new(0, 0, &keys[0..3])?;
    assert!(position.is_inline());

    let position = Position::new(0, 0, &keys[0..4])?;
    assert!(position.is_heap());
    Ok(())
}

#[quickcheck]
fn property_testing(site: u16, clock: u16, bytes: Vec<u32>) -> Result<(), TryFromIntError> {
    let position = Position::new(site, clock, &bytes)?;
    let result = position.as_slice();

    assert_eq!(&bytes, result);
    Ok(())
}
