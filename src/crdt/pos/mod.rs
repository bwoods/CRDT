#![allow(unsafe_code)]

pub use traits::*;

pub mod path;
mod traits;

#[cfg(test)]
mod test;

#[cfg(feature = "serde")]
mod serde;

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
#[doc = include_str!("README.md")]
pub union Position {
    small: Small,
    large: Large,
}

impl Position {
    pub(crate) fn new(site: u16, clock: u16, path: &[u32]) -> Position {
        let len: u16 = path.len() as u16;

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

        new
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
    /// Returns the timestamp for when this Position was created.
    pub(crate) fn clock(&self) -> u16 {
        unsafe { self.small.clock }
    }

    #[inline]
    /// Returns the length of this Position.
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
                &self.small.path[..self.level()]
            } else {
                std::slice::from_raw_parts(self.large.path, self.large.length as usize)
            }
        }
    }
}
