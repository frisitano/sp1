//! The allocator of a static library build (`libzkevm.a`).
//!
//! The guest program that links the library owns the memory between `_end` and the input region
//! (`_heap_start` and `_heap_end` in `zkvm.ld`) and brings its own allocator, so SP1 allocates
//! from a private buffer in `.bss` instead. It registers no `critical-section` implementation, which
//! the guest may define.
//!
//! The buffer is a stack of blocks that frees in any order. Freeing the most recent block pops it,
//! then every block directly beneath it that was freed before; freeing an older block only marks
//! it. An exported function drops everything it allocates before it returns, so each call leaves
//! the stack where it found it, without marking where a call starts or ends.
//!
//! Each block has a header of two words: where the stack's top was before it (to pop it), and the
//! block beneath it, whose lowest bit marks this block freed.

use crate::EMBEDDED_RESERVED_INPUT_START;
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::Cell,
    ptr,
};

/// Size of the private buffer.
const HEAP_SIZE: usize = 16 * 1024 * 1024;
/// Alignment of every block.
const MIN_ALIGN: usize = 8;
/// Bytes of the header before every block.
const HEADER: usize = 2 * size_of::<usize>();
/// The lowest bit of a header's second word: the block is freed.
const FREED: usize = 1;

#[global_allocator]
pub static HEAP: Heap = Heap { next: Cell::new(0), top: Cell::new(0), end: Cell::new(0) };

pub fn init() {
    // Only used through its address.
    #[allow(dead_code)]
    #[repr(align(16))]
    struct Buffer([u8; HEAP_SIZE]);
    static mut BUFFER: Buffer = Buffer([0; HEAP_SIZE]);

    let start = core::ptr::addr_of_mut!(BUFFER) as usize;
    HEAP.next.set(start);
    HEAP.end.set(start + HEAP_SIZE);
}

/// A stack allocator over the private buffer that frees in any order.
pub struct Heap {
    /// The first free byte.
    next: Cell<usize>,
    /// The most recent block, or 0 if there is none.
    top: Cell<usize>,
    /// One past the last byte of the buffer.
    end: Cell<usize>,
}

// SP1 runs a single thread.
unsafe impl Sync for Heap {}

impl Heap {
    /// The two words of `block`'s header.
    fn header(block: usize) -> *mut [usize; 2] {
        (block - HEADER) as *mut [usize; 2]
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mask = layout.align().max(MIN_ALIGN) - 1;
        let Some(block) = self.next.get().checked_add(HEADER + mask).map(|a| a & !mask) else {
            return ptr::null_mut();
        };
        match block.checked_add(layout.size()) {
            Some(end) if end <= self.end.get() => {
                Self::header(block).write([self.next.get(), self.top.get()]);
                self.top.set(block);
                self.next.set(end);
                block as *mut u8
            }
            _ => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        let block = ptr as usize;
        // Deallocating reserved input region memory is not allowed.
        if block >= EMBEDDED_RESERVED_INPUT_START {
            return;
        }
        if block != self.top.get() {
            (*Self::header(block))[1] |= FREED;
            return;
        }
        // Pop the block, then every freed block directly beneath it.
        loop {
            let [next, below] = Self::header(self.top.get()).read();
            self.next.set(next);
            self.top.set(below & !FREED);
            let top = self.top.get();
            if top == 0 || (*Self::header(top))[1] & FREED == 0 {
                break;
            }
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // The most recent block grows or shrinks in place.
        let block = ptr as usize;
        if block == self.top.get() {
            return match block.checked_add(new_size) {
                Some(end) if end <= self.end.get() => {
                    self.next.set(end);
                    ptr
                }
                _ => ptr::null_mut(),
            };
        }
        let new = self.alloc(Layout::from_size_align_unchecked(new_size, layout.align()));
        if !new.is_null() {
            ptr::copy_nonoverlapping(ptr, new, layout.size().min(new_size));
            self.dealloc(ptr, layout);
        }
        new
    }
}
