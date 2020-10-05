use core::alloc::Layout;
use core::fmt;
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^22 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
///   
pub struct Allocator {
    // FIXME: Add the necessary fields.
    current: usize,
    end: usize,
    bins: [LinkedList; 33],
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let mut list_arr: [LinkedList; 33] = [LinkedList::new(); 33];
        return Allocator {
            current: start,
            end: end,
            bins: list_arr,
        };
    }
}

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if (layout.align() & (layout.align() - 1)) != 0 {
            return core::ptr::null_mut();
        }
        if (layout.size() <= 0) {
            return core::ptr::null_mut();
        }
        let mut size: usize = 0;
        let mut idx: usize = 32;
        // 2 ^ 5 is the smallest alloc size
        for i in 5..33 {
            if layout.size() <= (1 << i) {
                size = 1 << i;
                idx = i - 5;
                break;
            }
        }
        match self.bins[idx].peek() {
            Some(ptr) => {
                /*
                for node in self.bins[idx].iter_mut() {
                    if node.value() as usize % layout.align() == 0 {
                        let v = node.value() as *mut u8;
                        node.pop();
                        return v;
                    }
                }
                */
                if ptr as usize % layout.align() == 0 {
                    self.bins[idx].pop();
                    return ptr as *mut u8;
                }
        
            },
            None => (),
        }
        let cur = align_up(self.current, layout.align());
        if cur + size > self.end {
            return core::ptr::null_mut();
        }
        self.current = cur + size;
        return cur as *mut u8;
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let mut idx: usize = 32;
        for i in 5..32 {
            if layout.size() <= (1 << i) {
                idx = i - 5;
                break;
            }
        }
        self.bins[idx].push(ptr as *mut usize);
    }
}

// FIXME: Implement `Debug` for `Allocator`.
impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Allocator")
            .field("current", &self.current)
            .field("end", &self.end)
            .finish()
    }
}
