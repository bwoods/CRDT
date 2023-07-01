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
        let len: u16 = path.len().try_into()?; // let this fail before `alloc` is called
        let mut new = Position {
            small: Small {
                site,
                clock,
                ..Default::default()
            },
        };

        unsafe {
            if len as usize <= INLINE {
                std::ptr::copy_nonoverlapping(
                    path.as_ptr(),
                    new.small.path.as_mut_ptr(),
                    len as usize,
                )
            } else {
                let layout = std::alloc::Layout::array::<u32>(len as usize).unwrap();
                let ptr = std::alloc::alloc(layout) as *mut u32;

                if ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }

                std::ptr::copy_nonoverlapping(path.as_ptr(), ptr, len as usize);
                new.large.path = ptr;
                new.large.tag = 0xff;
                new.large.length = len;
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
    /// Returns the site id for this Position.
    fn clock(&self) -> u16 {
        unsafe { self.small.clock }
    }

    #[inline]
    /// Returns the site id for this Position.
    fn level(&self) -> usize {
        unsafe {
            if self.is_heap() {
                self.large.length as usize
            } else {
                self.small
                    .path
                    .iter()
                    .position(|n| *n == 0)
                    .unwrap_or(INLINE)
            }
        }
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
                std::alloc::dealloc(
                    self.large.path.cast_mut() as *mut u8,
                    std::alloc::Layout::array::<u32>(self.large.length as usize).unwrap(),
                );
            }
        }
    }
}
