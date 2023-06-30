#![allow(unsafe_code)]

use std::convert::TryInto;
use std::num::TryFromIntError;

pub use traits::*;

mod paths;
mod traits;

#[cfg(test)]
mod test;

const INLINE: usize = 3;

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct Small {
    site: u16,
    clock: u16,
    path: [u32; INLINE],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Large {
    site: u16,
    clock: u16,
    tag: u8, // 0xff
    pad: u8,
    length: u16, // up to 2¹⁶ 32-bit words
    path: *const u32,
}

#[repr(C)]
pub union Position {
    small: Small,
    large: Large,
}

impl Position {
    pub fn new(site: u16, clock: u16, path: &[u32]) -> Result<Position, TryFromIntError> {
        let len = path.len();
        let mut new = Position {
            small: Default::default(),
        };

        unsafe {
            if path.len() <= new.small.path.len() {
                new.small.site = site;
                new.small.clock = clock;
                std::ptr::copy_nonoverlapping(path.as_ptr(), new.small.path.as_mut_ptr(), len)
            } else {
                let mut vec = Vec::with_capacity(len);
                vec.extend(path.iter()); // copy bytes to the heap
                vec.shrink_to_fit();

                new.large.site = site;
                new.small.clock = clock;
                new.large.tag = 0xff;
                new.large.length = len.try_into()?;

                // SAFETY: If we use `.as_ptr()` here, instead of `.as_mut_ptr()`, `miri test` fails
                //         the `.cast_mut()` in `Drop` (which is required by `Vec::from_raw_parts`).
                new.large.path = vec.leak().as_mut_ptr();
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
                &self.small.path
            } else {
                std::slice::from_raw_parts(self.large.path, self.large.length as usize)
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
                let len = self.large.length as usize;
                let ptr = self.large.path.cast_mut();

                // deallocate the bytes
                let _ = Vec::<_>::from_raw_parts(ptr, len, len);
            }
        }
    }
}
