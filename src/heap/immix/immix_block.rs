use super::*;
use crate::heap::*;
use atomic::*;
use memmap::MmapMut;
use vec_map::Entry;
pub struct BlockBitmap {
    bitmap: [u8; BLOCK_BITMAP_SIZE],
    heap_begin: usize,
}
const BITS_PER_INTPTR: usize = core::mem::size_of::<usize>() * 8;

impl BlockBitmap {
    #[inline(always)]
    pub fn set(&self, obj: usize) -> bool {
        return self.modify(obj, true);
    }
    #[inline(always)]
    pub fn clear(&self, obj: usize) -> bool {
        return self.modify(obj, false);
    }
    pub fn atomic_test_and_set(&self, obj: usize) -> bool {
        let offset = obj - self.heap_begin;
        let index = Self::offset_to_index(offset);
        let mask = Self::offset_to_mask(offset);
        let entry = &self.begin_as_slice()[index];
        let mut old_word;
        while {
            old_word = entry.load(Ordering::Relaxed);
            if (old_word & mask) != 0 {
                return true;
            }
            entry.compare_exchange_weak(
                old_word,
                old_word | mask,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) != Ok(old_word)
        } {}

        false
    }

    pub fn test(&self, obj: usize) -> bool {
        let offset = obj - self.heap_begin;
        let index = Self::offset_to_index(offset);
        (self.begin_as_slice()[index].load(Ordering::Relaxed) & Self::offset_to_mask(offset)) != 0
    }
    pub fn test_unsync(&self, obj: usize) -> bool {
        let offset = obj - self.heap_begin;
        let index = Self::offset_to_index(offset);
        (self.begin_as_slice_()[index] & Self::offset_to_mask(offset)) != 0
    }
    pub fn walk(&self, mut visitor: impl FnMut(usize)) {
        let end = Self::offset_to_index((self.heap_begin + 32 * 1024) - self.heap_begin - 1);
        let bitmap_begin = self.begin() as *const Atomic<usize>;
        for i in 0..=end {
            let mut w = unsafe { &*bitmap_begin.offset(i as _) }.load(Ordering::Relaxed);
            if w != 0 {
                let ptr_base = Self::index_to_offset(i) + self.heap_begin;
                while {
                    let shift = w.trailing_zeros() as usize;
                    let obj = ptr_base + shift * 16;
                    visitor(obj);
                    w ^= 1 << shift;
                    w != 0
                } {}
            }
        }
    }

    pub fn modify(&self, obj: usize, set: bool) -> bool {
        let offset = obj - self.heap_begin;
        let index = Self::offset_to_index(offset);
        let mask = Self::offset_to_mask(offset);
        let atomic_entry = &self.begin_as_slice()[index];
        let old_word = atomic_entry.load(Ordering::Relaxed);
        if set {
            if (old_word & mask) == 0 {
                atomic_entry.store(old_word | mask, Ordering::Relaxed);
            }
        } else {
            atomic_entry.store(old_word & !mask, Ordering::Relaxed)
        }
        (old_word & mask) != 0
    }
    pub fn clear_all(&mut self) {
        for entry in self.bitmap.iter_mut() {
            *entry = 0;
        }
    }
    pub fn new(heap_begin: usize) -> Self {
        Self {
            heap_begin,
            bitmap: [0; BLOCK_BITMAP_SIZE],
        }
    }
    pub fn begin_as_slice(
        &self,
    ) -> &'static [Atomic<usize>; BLOCK_BITMAP_SIZE / core::mem::size_of::<usize>()] {
        unsafe { std::mem::transmute::<&[u8; BLOCK_BITMAP_SIZE], _>(&self.bitmap) }
    }
    pub fn begin_as_slice_(
        &self,
    ) -> &'static [usize; BLOCK_BITMAP_SIZE / core::mem::size_of::<usize>()] {
        unsafe { std::mem::transmute::<&[u8; BLOCK_BITMAP_SIZE], _>(&self.bitmap) }
    }
    pub fn begin(&self) -> &'static Atomic<usize> {
        unsafe { &*self.bitmap.as_ptr().cast::<Atomic<usize>>() }
    }
    const fn offset_to_index(offset: usize) -> usize {
        offset / 16 / BITS_PER_INTPTR
    }

    const fn index_to_offset(index: usize) -> usize {
        index * 16 * BITS_PER_INTPTR
    }

    const fn offset_bit_index(offset: usize) -> usize {
        (offset / 16) % BITS_PER_INTPTR
    }

    const fn offset_to_mask(offset: usize) -> usize {
        1 << Self::offset_bit_index(offset)
    }
}

use vec_map::VecMap;

#[repr(C)]
pub struct ImmixBlock {
    pub next: *mut ImmixBlock,
    pub bitmap: BlockBitmap,
    /// A counter of live objects for every line in this block.
    pub line_counter: VecMap<usize>,
    /// If this block is actually in use.
    allocated: bool,

    /// How many holes are in this block.
    hole_count: usize,

    /// If this block is a candidate for opportunistic evacuation.
    evacuation_candidate: bool,
}

impl ImmixBlock {
    pub fn new(next: *mut Self) -> &'static mut ImmixBlock {
        unsafe {
            let mut line_counter = VecMap::with_capacity(NUM_LINES_PER_BLOCK);
            for index in 0..NUM_LINES_PER_BLOCK {
                line_counter.insert(index, 0);
            }

            let mem = std::alloc::alloc(std::alloc::Layout::array::<u8>(BLOCK_SIZE).unwrap())
                .cast::<Self>();
            mem.write(Self {
                next,
                line_counter,
                bitmap: BlockBitmap::new(mem as _),
                evacuation_candidate: false,
                allocated: false,
                hole_count: 0,
            });
            &mut *mem
        }
    }
    /// Set this block as allocated (actually in use).
    pub fn set_allocated(&mut self) {
        self.allocated = true;
    }

    /// Set an address in this block as a valid object.
    pub fn set_gc_object(&mut self, object: usize) {
        debug_assert!(
            self.is_in_block(object),
            "set_gc_object() on invalid block: {:p} (allocated={})",
            self,
            self.allocated
        );
        self.bitmap.set(object);
    }

    /// Unset an address in this block as a valid object.
    pub fn unset_gc_object(&mut self, object: usize) {
        debug_assert!(
            self.is_in_block(object),
            "unset_gc_object() on invalid block: {:p} (allocated={})",
            self,
            self.allocated
        );
        self.bitmap.clear(object);
    }

    /// Return if an address in this block is a valid object.
    pub fn is_gc_object(&self, object: usize) -> bool {
        if self.is_in_block(object) {
            self.bitmap.test(object)
        } else {
            false
        }
    }

    /// Clear the object map.
    pub fn clear_object_map(&mut self) {
        self.bitmap.clear_all();
    }

    /// Set as an evacuation candidate if this block has at least `hole_count`
    /// holes.
    pub fn set_evacuation_candidate(&mut self, hole_count: usize) {
        self.evacuation_candidate = self.hole_count >= hole_count;
    }
    /// Return true if no line is marked (every line has a count of zero).
    pub fn is_empty(&self) -> bool {
        self.line_counter.values().all(|v| *v == 0)
    }

    /// Get a pointer to an address `offset` bytes into this block.
    pub fn offset(&mut self, offset: usize) -> usize {
        let self_ptr = self as *mut Self;
        let object = unsafe { (self_ptr as *mut u8).offset(offset as isize) };
        object as usize
    }

    /// Scan the block for a hole to allocate into.
    ///
    /// The scan will start at `last_high_offset` bytes into the block and
    /// return a tuple of `low_offset`, `high_offset` as the lowest and
    /// highest usable offsets for a hole.
    ///
    /// `None` is returned if no hole was found.
    pub fn scan_block(&self, last_high_offset: u16) -> Option<(u16, u16)> {
        let last_high_index = last_high_offset as usize / LINE_SIZE;
        let mut low_index = NUM_LINES_PER_BLOCK - 1;
        for index in (last_high_index + 1)..NUM_LINES_PER_BLOCK {
            if self.line_counter.get(index).map_or(true, |c| *c == 0) {
                // +1 to skip the next line in case an object straddles lines
                low_index = index + 1;
                break;
            }
        }
        let mut high_index = NUM_LINES_PER_BLOCK;
        for index in low_index..NUM_LINES_PER_BLOCK {
            if self.line_counter.get(index).map_or(false, |c| *c != 0) {
                high_index = index;
                break;
            }
        }
        if low_index == high_index && high_index != (NUM_LINES_PER_BLOCK - 1) {
            return self.scan_block((high_index * LINE_SIZE - 1) as u16);
        } else if low_index < (NUM_LINES_PER_BLOCK - 1) {
            return Some((
                (low_index * LINE_SIZE) as u16,
                (high_index * LINE_SIZE - 1) as u16,
            ));
        }
        None
    }

    /// Return if this is an evacuation candidate.
    pub fn is_evacuation_candidate(&self) -> bool {
        self.evacuation_candidate
    }

    /// Increment the lines on which the object is allocated.
    pub fn increment_lines(&mut self, object: usize) {
        self.update_line_nums(object, true);
    }

    /// Decrement the lines on which the object is allocated.
    pub fn decrement_lines(&mut self, object: usize) {
        self.update_line_nums(object, false);
    }

    /// Return the number of holes and marked lines in this block.
    ///
    /// A marked line is a line with a count of at least one.
    ///
    /// _Note_: You must call count_holes() bevorhand to set the number of
    /// holes.
    pub fn count_holes_and_marked_lines(&self) -> (usize, usize) {
        (
            self.hole_count,
            self.line_counter.values().filter(|&e| *e != 0).count(),
        )
    }

    /// Return the number of holes and available lines in this block.
    ///
    /// An available line is a line with a count of zero.
    ///
    /// _Note_: You must call count_holes() bevorhand to set the number of
    /// holes.
    pub fn count_holes_and_available_lines(&self) -> (usize, usize) {
        (
            self.hole_count,
            self.line_counter.values().filter(|&e| *e == 0).count(),
        )
    }

    /// Clear the line counter map.
    pub fn clear_line_counts(&mut self) {
        for index in 0..NUM_LINES_PER_BLOCK {
            self.line_counter.insert(index, 0);
        }
    }

    /// Reset all member field for this block.
    pub fn reset(&mut self) {
        self.clear_line_counts();
        self.bitmap.clear_all();
        self.allocated = false;
        self.hole_count = 0;
        self.evacuation_candidate = false;
    }

    /// Returns true if this block is allocated and the address is within the
    /// bounds of this block.
    fn is_in_block(&self, object: usize) -> bool {
        // This works because we get zeroed memory from the OS, so
        // self.allocated will be false if this block is not initialized and
        // this method gets only called for objects within the ImmixSpace.
        // After the first initialization the field is properly managed.
        if self.allocated {
            let self_ptr = self as *const Self as *const u8;
            let self_bound = unsafe { self_ptr.offset(BLOCK_SIZE as isize) };
            self_ptr < (object as *const u8) && (object as *const u8) < self_bound
        } else {
            false
        }
    }

    /// Convert an address on this block into a line number.
    fn object_to_line_num(object: usize) -> usize {
        (object as usize % BLOCK_SIZE) / LINE_SIZE
    }

    /// Update the line counter for the given object.
    ///
    /// Increment if `increment`, otherwise do a saturating substraction.
    fn update_line_nums(&mut self, object: usize, increment: bool) {
        // This calculates how many lines are affected starting from a
        // LINE_SIZE aligned address. So it might not mark enough lines. But
        // that does not matter as we always skip a line in scan_block()
        let line_num = Self::object_to_line_num(object);
        let object = unsafe { &mut *(object as *mut RawGc) };
        let object_size = object.object_size();
        for line in line_num..(line_num + (object_size / LINE_SIZE) + 1) {
            match self.line_counter.entry(line) {
                Entry::Vacant(view) => {
                    view.insert(if increment { 1 } else { 0 });
                }
                Entry::Occupied(mut view) => {
                    let val = view.get_mut();
                    if increment {
                        *val += 1;
                    } else {
                        *val = (*val).saturating_sub(1);
                    }
                }
            };
        }
    }
}
