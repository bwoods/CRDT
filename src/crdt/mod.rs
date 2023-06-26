#![allow(unsafe_code)]

use std::convert::TryInto;
use std::num::TryFromIntError;

pub use traits::*;

#[cfg(test)]
mod test;
mod traits;

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct Small {
    site: u16,
    clock: u16,
    idx: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Large {
    site: u16,
    clock: u16,
    tag: u8, // 0xff
    pad: u8,
    len: u16, // up to 2¹⁶ 32-bit words
    idx: *const u32,
}

#[repr(C)]
pub union Position {
    small: Small,
    large: Large,
}

impl Position {
    pub fn new(site: u16, clock: u16, slice: &[u32]) -> Result<Position, TryFromIntError> {
        let len = slice.len();
        let mut new = Position {
            small: Default::default(),
        };

        unsafe {
            if slice.len() <= new.small.idx.len() {
                new.small.site = site;
                new.small.clock = clock;
                std::ptr::copy_nonoverlapping(slice.as_ptr(), new.small.idx.as_mut_ptr(), len)
            } else {
                let mut vec = Vec::with_capacity(len);
                vec.extend(slice.iter()); // copy bytes to the heap
                vec.shrink_to_fit();

                new.large.site = site;
                new.small.clock = clock;
                new.large.tag = 0xff;
                new.large.len = len.try_into()?;

                // SAFETY: If we use `.as_prt()` here, instead of `.as_mut_ptr()`, `miri test` fails
                //         the `.cast_mut()` in `Drop` (which is required by `Vec::from_raw_parts`).
                new.large.idx = vec.leak().as_mut_ptr();
            }
        }

        Ok(new)
    }

    #[inline]
    /// Returns the site id for this Position.
    pub fn site_id(&self) -> u16 {
        unsafe { self.small.site }
    }

    #[inline]
    /// Returns whether the (arbitrarily long) index was allocated on the heap.
    pub fn is_heap(&self) -> bool {
        unsafe { self.large.tag == 0xff }
    }

    #[inline]
    #[allow(clippy::bool_comparison)]
    /// Returns whether the (arbitrarily long) index is held inline.
    pub fn is_inline(&self) -> bool {
        self.is_heap() == false
    }

    #[inline]
    /// Returns the encoded bytes of the Position’s index.
    fn as_slice(&self) -> &[u32] {
        unsafe {
            if self.is_inline() {
                // zero values only exist in unused slots; trim them off
                self.small.idx.split(|&n| n == 0).next().unwrap_or(&[])
            } else {
                std::slice::from_raw_parts(self.large.idx, self.large.len as usize)
            }
        }
    }
}

impl Drop for Position {
    /// # Safety
    ///
    /// The [`Position`] must have been created with [`Position::new()`]
    /// to guarantee that is is correctly tagged as `is_heap` or `is_inline`.    
    ///
    fn drop(&mut self) {
        unsafe {
            if self.is_heap() {
                let len = self.large.len as usize;
                let ptr = self.large.idx.cast_mut();

                // deallocate the bytes
                let _ = Vec::<_>::from_raw_parts(ptr, len, len);
            }
        }
    }
}

impl Position {
    fn first() -> Position {
        Position {
            small: Default::default(),
        }
    }

    fn last() -> Position {
        Position {
            small: Small {
                idx: [
                    0xfffffffe_u32.to_le(), // 0xfe, because 0xff is used as a tag
                    0,                      // u32::from_le(n) should be used below
                    0,
                ],
                ..Default::default()
            },
        }
    }
}
