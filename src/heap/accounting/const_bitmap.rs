//! Bitmap similar to SpaceBitmap but uses compile time known arrays
use crate::heap::*;
use atomic::*;
use std::mem::*;
pub const BITS_PER_INTPTR: usize = 8 * size_of::<usize>();
#[macro_export]
macro_rules! new_const_bitmap {
    ($name: ident, $alignment: expr,$heap_cap: expr) => {
        pub struct $name {
            pub bitmap_memory: [u8; {
                let bytes_covered_per_word = $alignment * (core::mem::size_of::<usize>() * 8);
                (round_up_!($heap_cap as u64, bytes_covered_per_word as u64)
                    / bytes_covered_per_word as u64) as usize
                    * core::mem::size_of::<isize>()
            }],
            bitmap_begin: *mut atomic::Atomic<usize>,
            heap_begin: usize,
            heap_limit: usize,
        }

        impl $name {
            const SIZE: usize = {
                let bytes_covered_per_word = $alignment * (core::mem::size_of::<usize>() * 8);
                (round_up_!($heap_cap as u64, bytes_covered_per_word as u64)
                    / bytes_covered_per_word as u64) as usize
                    * core::mem::size_of::<isize>()
            };
            const ALIGNMENT: usize = $alignment;
            const HEAP_CAP: usize = $heap_cap;
            pub const fn offset_bit_index(offset: usize) -> usize {
                (offset / $alignment) % (core::mem::size_of::<usize>() * 8)
            }

            pub const fn offset_to_index(offset: usize) -> usize {
                offset / Self::ALIGNMENT / (core::mem::size_of::<usize>() * 8)
            }

            pub const fn index_to_offset(index: usize) -> isize {
                return index as isize
                    * Self::ALIGNMENT as isize
                    * (core::mem::size_of::<usize>() as isize * 8);
            }

            pub const fn offset_to_mask(offset: usize) -> usize {
                1 << Self::offset_bit_index(offset)
            }

            pub unsafe fn sweep_walk(
                live_bitmap: &Self,
                mark_bitmap: &Self,
                sweep_begin: usize,
                sweep_end: usize,
                mut callback: impl FnMut(usize, usize),
            ) {
                if sweep_end <= sweep_begin {
                    return;
                }

                let buffer_size =
                    core::mem::size_of::<isize>() * (core::mem::size_of::<isize>() * 8);

                let live = live_bitmap.bitmap_begin;
                let mark = mark_bitmap.bitmap_begin;

                let start = Self::offset_to_index(sweep_begin - live_bitmap.heap_begin as usize);
                let end = Self::offset_to_index(sweep_end - live_bitmap.heap_begin - 1);
                let mut pointer_buf = vec![0usize; buffer_size];
                let mut cur_pointer = pointer_buf.as_mut_ptr();
                let pointer_end = cur_pointer
                    .offset(buffer_size as isize - (core::mem::size_of::<usize>() * 8) as isize);

                for i in start..=end {
                    let mut garbage = (&*live.offset(i as _)).load(atomic::Ordering::Relaxed)
                        & !((&*mark.offset(i as _)).load(atomic::Ordering::Relaxed));
                    if garbage != 0 {
                        let ptr_base = Self::index_to_offset(i) + live_bitmap.heap_begin as isize;
                        let ptr_base = ptr_base as usize;
                        while {
                            let shift = garbage.trailing_zeros() as usize;
                            garbage ^= 1 << shift;
                            cur_pointer.write(ptr_base + shift + Self::ALIGNMENT);
                            cur_pointer = cur_pointer.offset(1);
                            garbage != 0
                        } {}

                        if cur_pointer >= pointer_end {
                            callback(
                                cur_pointer.offset_from(pointer_buf.as_ptr()) as _,
                                pointer_buf.as_ptr() as usize,
                            );
                            cur_pointer = pointer_buf.as_ptr() as *mut _;
                        }
                    }
                }

                if cur_pointer >= &mut pointer_buf[0] as *mut _ {
                    callback(
                        cur_pointer.offset_from(pointer_buf.as_ptr()) as _,
                        pointer_buf.as_ptr() as usize,
                    );
                }
            }

            #[inline]
            pub fn atomic_test_and_set(&self, object: usize) -> bool {
                unsafe {
                    let offset = object as isize - self.heap_begin as isize;
                    let index = Self::offset_to_index(offset as _);
                    let mask = Self::offset_to_mask(offset as _);
                    let atomic_entry = &*self.bitmap_begin.offset(index as _);
                    let mut old_word;
                    while {
                        old_word = atomic_entry.load(atomic::Ordering::Relaxed);
                        if (old_word & mask) != 0 {
                            return true;
                        }
                        atomic_entry.compare_exchange_weak(
                            old_word,
                            old_word | mask,
                            atomic::Ordering::Relaxed,
                            atomic::Ordering::Relaxed,
                        ) != Ok(old_word)
                    } {}

                    false
                }
            }
            #[inline]
            pub fn test(&self, object: usize) -> bool {
                let offset = object as isize - self.heap_begin as isize;
                let index = Self::offset_to_index(offset as _);
                let mask = Self::offset_to_mask(offset as _);
                let atomic_entry = unsafe { &*self.bitmap_begin.offset(index as _) };
                (atomic_entry.load(atomic::Ordering::Relaxed) & mask) != 0
            }

            pub fn visit_marked_range(
                &self,
                visit_begin: usize,
                visit_end: usize,
                mut visitor: impl FnMut(usize),
            ) {
                unsafe {
                    let offset_start = visit_begin - self.heap_begin;
                    let offset_end = visit_end - self.heap_begin;

                    let index_start = Self::offset_to_index(offset_start);
                    let index_end = Self::offset_to_index(offset_end);

                    let bit_start =
                        (offset_start / Self::ALIGNMENT) * (core::mem::size_of::<usize>() * 8);
                    let bit_end =
                        (offset_end / Self::ALIGNMENT) * (core::mem::size_of::<usize>() * 8);

                    let mut left_edge = self
                        .bitmap_begin
                        .offset(index_start as _)
                        .cast::<usize>()
                        .read();

                    left_edge &= !((1 << bit_start) - 1);

                    let mut right_edge;
                    if index_start < index_end {
                        if left_edge != 0 {
                            let ptr_base =
                                Self::index_to_offset(index_start) as usize + self.heap_begin;
                            while {
                                let shift = left_edge.trailing_zeros() as usize;
                                let obj = ptr_base + shift * Self::ALIGNMENT;
                                visitor(obj);
                                left_edge ^= 1usize.wrapping_shl(shift as _);
                                left_edge != 0
                            } {}
                        }

                        for i in index_start + 1..index_end {
                            let mut w = (&*self.bitmap_begin.offset(i as _))
                                .load(atomic::Ordering::Relaxed);
                            if w != 0 {
                                let ptr_base = Self::index_to_offset(i) as usize + self.heap_begin;
                                while {
                                    let shift = w.trailing_zeros() as usize;
                                    let obj = ptr_base + shift * Self::ALIGNMENT;
                                    visitor(obj);
                                    w ^= 1usize.wrapping_shl(shift as _);
                                    w != 0
                                } {}
                            }
                        }

                        if bit_end == 0 {
                            right_edge = 0;
                        } else {
                            right_edge = self
                                .bitmap_begin
                                .offset(index_end as _)
                                .cast::<usize>()
                                .read();
                        }
                    } else {
                        right_edge = left_edge;
                    }

                    right_edge &= (1usize.wrapping_shl(bit_end as _)) - 1;

                    if right_edge != 0 {
                        let ptr_base = Self::index_to_offset(index_end) as usize + self.heap_begin;
                        while {
                            let shift = right_edge.trailing_zeros() as usize;
                            let obj = ptr_base + shift * Self::ALIGNMENT;
                            visitor(obj);
                            right_edge ^= 1usize.wrapping_shl(shift as _);
                            right_edge != 0
                        } {}
                    }
                }
            }

            pub fn walk(&self, mut visitor: impl FnMut(usize)) {
                unsafe {
                    let end = Self::offset_to_index(self.heap_limit - self.heap_begin - 1);
                    let bitmap_begin = self.bitmap_begin;
                    for i in 0..=end {
                        let mut w = (&*bitmap_begin.offset(i as _)).load(atomic::Ordering::Relaxed);
                        if w != 0 {
                            let ptr_base = Self::index_to_offset(i) as usize + self.heap_begin;

                            while {
                                let shift = w.trailing_zeros() as usize;
                                let obj = ptr_base + shift * Self::ALIGNMENT;
                                visitor(obj);
                                w ^= 1 << shift;
                                w != 0
                            } {}
                        }
                    }
                }
            }
            #[inline]
            pub fn modify<const SET_BIT: bool>(&self, obj: usize) -> bool {
                unsafe {
                    let offset = obj - self.heap_begin;
                    let index = Self::offset_to_index(offset);
                    let mask = Self::offset_to_mask(offset);

                    let atomic_entry = &*self.bitmap_begin.offset(index as _);
                    let old_word = atomic_entry.load(atomic::Ordering::Relaxed);
                    if SET_BIT {
                        if (old_word & mask) == 0 {
                            atomic_entry.store(old_word | mask, atomic::Ordering::Relaxed);
                        }
                    } else {
                        atomic_entry.store(old_word & !mask, atomic::Ordering::Relaxed);
                    }

                    (old_word & mask) != 0
                }
            }

            #[inline]
            pub fn clear_to_zeros(&mut self) {
                if !self.bitmap_begin.is_null() {
                    for elem in self.bitmap_memory.iter_mut() {
                        *elem = 0;
                    }
                }
            }

            #[inline]
            pub fn clear_range(&self, begin: usize, end: usize) {
                unsafe {
                    let mut begin_offset = begin - self.heap_begin;
                    let mut end_offset = end - self.heap_begin;

                    while begin_offset < end_offset && Self::offset_bit_index(begin_offset) != 0 {
                        self.clear(self.heap_begin + begin_offset);
                        begin_offset += Self::ALIGNMENT;
                    }
                    while begin_offset < end_offset && Self::offset_bit_index(end_offset) != 0 {
                        end_offset -= Self::ALIGNMENT;
                        self.clear(self.heap_begin + end_offset);
                    }

                    let start_index = Self::offset_to_index(begin_offset);
                    let end_index = Self::offset_to_index(end_offset);
                    std::ptr::write_bytes(
                        self.bitmap_begin.offset(start_index as _).cast::<u8>(),
                        0,
                        (end_index - start_index) * core::mem::size_of::<usize>(),
                    );
                }
            }
            #[inline(always)]
            pub const fn size(&self) -> usize {
                Self::SIZE
            }

            pub fn heap_begin(&self) -> usize {
                self.heap_begin
            }

            pub fn heap_limit(&self) -> usize {
                self.heap_limit
            }
            pub fn begin(&self) -> *mut atomic::Atomic<usize> {
                self.bitmap_begin
            }

            pub fn copy_from(&self, source_bitmap: &Self) {
                let count = source_bitmap.size() / core::mem::size_of::<usize>();
                unsafe {
                    let src = source_bitmap.begin();
                    let dest = self.begin();
                    for i in 0..count {
                        (&*dest.offset(i as _)).store(
                            (&*src.offset(i as _)).load(atomic::Ordering::Relaxed),
                            atomic::Ordering::Relaxed,
                        );
                    }
                }
            }
            pub fn new(heap_begin: *mut u8) -> Self {
                let mut this = Self {
                    bitmap_memory: [0; Self::SIZE],
                    bitmap_begin: std::ptr::null_mut(),
                    heap_begin: heap_begin as _,
                    heap_limit: heap_begin as usize + Self::HEAP_CAP,
                };
                this.bitmap_begin =
                    this.bitmap_memory.as_ptr() as *const u8 as *mut atomic::Atomic<usize>;
                this
            }
            #[allow(unused_braces)]
            #[inline]
            pub fn clear(&self, obj: usize) {
                self.modify::<{ false }>(obj);
            }

            #[allow(unused_braces)]
            #[inline]
            pub fn set(&self, obj: usize) {
                self.modify::<{ true }>(obj);
            }
        }
    };
}
