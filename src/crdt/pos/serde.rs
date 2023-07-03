use serde_crate::{Deserialize, Serialize};

use super::path::Builder;

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "serde_crate")]
struct Payload {
    site: u128, // `site`s are represented by UUIDs externally
    clock: u16,
    path: Builder,
}

#[test]
#[ignore]
fn quick_uuid() {
    // A standard conforming implementation of UUIDs in three lines.
    let mut uuid = fastrand::u128(..).to_be_bytes();
    uuid[6] = (uuid[6] & 0x0F) | 0x40; // Version 4
    uuid[8] = (uuid[8] & 0x3F) | 0x80; // Variant 1

    let uuid = u128::from_be_bytes(uuid);
    println!("{:x}", uuid);
}
