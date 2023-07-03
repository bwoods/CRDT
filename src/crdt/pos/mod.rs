#![allow(unsafe_code)]

use std::convert::TryInto;

pub use traits::*;

pub mod path;
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
    length: u16, // up to 2¹⁶ 32-bit words
    pad: u8,
    tag: u8, // 0xff
    path: *const u32,
}

#[repr(C)]
pub union Position {
    small: Small,
    large: Large,
}

#[derive(Debug)]
/// The type returned in the event of a construction error.
pub enum Error {
    /// Path with up to 65535 components are supported.
    PathTooLong(usize),

    /// Strings larger than 4 GiB are not supported.
    StringTooLarge,
}

impl Position {
    pub(crate) fn new(site: u16, clock: u16, path: &[u32]) -> Result<Position, Error> {
        let len: u16 = path
            .len()
            .try_into() // let this fail *before* `alloc` is called below to avoid possible memory leaks
            .map_err(|_| Error::PathTooLong(path.len()))?;

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
                new.large.length = len;
                new.large.tag = 0xff; // tag it last for `Drop` safety
            }
        }

        Ok(new)
    }
}

impl Clone for Position {
    /// # Safety
    ///
    /// The [`Position`] must have been created with [`Position::new()`]
    /// to guarantee that is is correctly tagged as `is_heap` or `is_inline`.    
    ///
    fn clone(&self) -> Self {
        unsafe {
            if self.is_inline() {
                Position { small: self.small }
            } else {
                let layout = std::alloc::Layout::array::<u32>(self.large.length as usize).unwrap();
                let ptr = std::alloc::alloc(layout) as *mut u32;

                if ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }

                std::ptr::copy_nonoverlapping(self.large.path, ptr, self.large.length as usize);

                Position {
                    large: Large {
                        path: ptr,
                        ..self.large
                    },
                }
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
                let layout = std::alloc::Layout::array::<u32>(self.large.length as usize).unwrap();
                std::alloc::dealloc(self.large.path.cast_mut() as *mut u8, layout);
            }
        }
    }
}

impl Position {
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
    /// Returns whether the position’s path was allocated on the heap.
    pub fn is_heap(&self) -> bool {
        unsafe { self.large.tag == 0xff }
    }

    #[inline]
    #[allow(clippy::bool_comparison)]
    /// Returns whether the position is completely held inline.
    pub fn is_inline(&self) -> bool {
        self.is_heap() == false
    }

    #[inline]
    /// Returns the the Position’s path.
    pub(crate) fn path(&self) -> &[u32] {
        unsafe {
            if self.is_inline() {
                &self.small.path
            } else {
                std::slice::from_raw_parts(self.large.path, self.large.length as usize)
            }
        }
    }
}

#[test]
fn tag_position() -> Result<(), Error> {
    let valid = Position::new(0, 0, &[0xff])?;
    assert!(valid.is_inline());

    let invalid = Position::new(0, 0, &[0xffffffff])?;
    assert!(invalid.is_heap()); // this is why this path MUST never be generated!

    // …and this is why it never will be.
    assert!(unsafe { Position::last().small.path[0] < invalid.small.path[0] });

    // we can't even `Drop` it correctly
    std::mem::forget(invalid);
    Ok(())
} //
