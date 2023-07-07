use serde_crate::{Deserialize, Serialize};

use crate::crdt::pos::path::Builder;
use crate::Position;

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "serde_crate")]
struct Payload {
    site: u128, // `site`s are represented by UUIDs externally
    clock: u16,
    path: Builder,
}

impl Payload {
    pub fn from_position(pos: &Position, uuid: impl FnOnce(u16) -> u128) -> Self {
        Payload {
            site: uuid(pos.site_id()),
            clock: pos.clock(),
            path: Builder::from(pos.path()),
        }
    }

    pub fn into_position(self, site_id: impl FnOnce(u128) -> u16) -> Position {
        Position::new(site_id(self.site), self.clock, &self.path)
    }
}

#[test]
#[ignore]
fn notes_quick_uuid() {
    // A standard conforming implementation of UUIDs in three lines.
    let mut uuid = fastrand::u128(..).to_be_bytes();
    uuid[6] = (uuid[6] & 0x0F) | 0x40; // Version 4
    uuid[8] = (uuid[8] & 0x3F) | 0x80; // Variant 1

    let uuid = u128::from_be_bytes(uuid);
    println!("{:x}", uuid);
}

#[test]
#[ignore]
fn notes_quick_site_id_lookup() {
    // As funny/functional as this is, it doesn’t remove
    // the need for a reverse map from uuid → site id.
    // Also, it makes for very predicable uuids…
    let uuid_for_site_id = |site: u16| -> u128 {
        if site == 0 {
            return 0;
        }

        let mut rng = fastrand::Rng::with_seed(site as u64);
        rng.u128(..)
    };

    let a = uuid_for_site_id(1);
    let b = uuid_for_site_id(1);
    assert_eq!(a, b);

    let c = uuid_for_site_id(2);
    assert_ne!(a, c);
}
