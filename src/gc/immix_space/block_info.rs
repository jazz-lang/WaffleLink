use crate::gc::constants::{BLOCK_SIZE, LINE_SIZE, NUM_LINES_PER_BLOCK};
use crate::gc::*;
use bit_set::BitSet;
use std::collections::HashSet;
use vec_map::{Entry, VecMap};

/// A per block object map.
struct ObjectMap {
    set: BitSet,
}

impl ObjectMap {
    /// Create a new `ObjectMap`.
    fn new() -> ObjectMap {
        ObjectMap {
            set: BitSet::with_capacity(BLOCK_SIZE),
        }
    }

    /// Reduce the objects address to an offset within the block.
    fn index(object: GCObjectRef) -> usize {
        (object.raw() as usize) % BLOCK_SIZE
    }

    /// Set the address as a valid object.
    fn set_object(&mut self, object: GCObjectRef) {
        self.set.insert(ObjectMap::index(object));
    }

    /// Unset the address as a valid object.
    fn unset_object(&mut self, object: GCObjectRef) {
        self.set.remove(ObjectMap::index(object));
    }

    /// Return `true` is the address is a valid object.
    fn is_object(&self, object: GCObjectRef) -> bool {
        self.set.contains(ObjectMap::index(object))
    }

    /// Update this `ObjectMap` with the difference of this `ObjectMap` and
    /// the other.
    fn difference(&mut self, other: &ObjectMap) {
        self.set.difference_with(&other.set);
    }

    /// Clear all entries.
    fn clear(&mut self) {
        self.set.clear();
    }

    /// Retrieve the values as a `HashSet`.
    fn as_hashset(&self, base: *mut u8) -> HashSet<GCObjectRef> {
        self.set
            .iter()
            .map(|i| unsafe {
                WaffleCellPointer::<WaffleCell>::from_ptr(base.offset(i as isize).cast())
            })
            .collect()
    }
}
/// A `BlockInfo` contains management information for the immix garbage
/// collectors.
///
/// It is the first bytes in a chunk of memory of size `BLOCK_SIZE` and must
/// not exeed `LINE_SIZE` bytes in size.
pub struct BlockInfo {
    /// A counter of live objects for every line in this block.
    line_counter: VecMap<usize>,

    /// A set of addresses that are valid objects. Needed for the conservative
    /// part.
    object_map: ObjectMap,

    /// Objects in this block that were never touched by the garbage
    /// collector.
    new_objects: ObjectMap,

    /// If this block is actually in use.
    allocated: bool,

    /// How many holen are in this block.
    hole_count: usize,

    /// If this block is a candidate for opportunistic evacuation.
    evacuation_candidate: bool,
}

impl BlockInfo {
    /// Create a new `BlockInfo`.
    pub fn new() -> BlockInfo {
        let mut line_counter = VecMap::with_capacity(NUM_LINES_PER_BLOCK);
        for index in 0..NUM_LINES_PER_BLOCK {
            line_counter.insert(index, 0);
        }
        BlockInfo {
            line_counter: line_counter,
            object_map: ObjectMap::new(),
            new_objects: ObjectMap::new(),
            allocated: false,
            hole_count: 0,
            evacuation_candidate: false,
        }
    }

    /// Set this block as allocated (actually in use).
    pub fn set_allocated(&mut self) {
        self.allocated = true;
    }
    /// Get the block for the given object.
    pub unsafe fn get_block_ptr(object: GCObjectRef) -> *mut BlockInfo {
        let block_offset = object.raw() as usize % BLOCK_SIZE;
        let block = std::mem::transmute((object.raw() as *mut u8).offset(-(block_offset as isize)));
        /*log::debug!(
            "Block for object {:p}: {:p} with offset: {}",
            object.raw(),
            block,
            block_offset
        );*/
        block
    }
    /// Set an address in this block as a valid object.
    pub fn set_gc_object(&mut self, object: GCObjectRef) {
        debug_assert!(
            self.is_in_block(object),
            "set_gc_object() on invalid block: {:p} (allocated={})",
            self,
            self.allocated
        );
        self.object_map.set_object(object);
    }

    /// Unset an address in this block as a valid object.
    pub fn unset_gc_object(&mut self, object: GCObjectRef) {
        debug_assert!(
            self.is_in_block(object),
            "unset_gc_object() on invalid block: {:p} (allocated={})",
            self,
            self.allocated
        );
        self.object_map.unset_object(object);
    }

    /// Return if an address in this block is a valid object.
    pub fn is_gc_object(&self, object: GCObjectRef) -> bool {
        if self.is_in_block(object) {
            self.object_map.is_object(object)
        } else {
            false
        }
    }

    /// Get a copy of the object map.
    pub fn get_object_map(&mut self) -> HashSet<GCObjectRef> {
        let self_ptr = self as *mut BlockInfo;
        self.object_map.as_hashset(self_ptr as *mut u8)
    }

    /// Clear the object map.
    pub fn clear_object_map(&mut self) {
        self.object_map.clear();
    }

    /// Set an object in this block as new (not the `GCHeader.new` bit).
    pub fn set_new_object(&mut self, object: GCObjectRef) {
        debug_assert!(
            self.is_in_block(object),
            "set_new_object() on invalid block: {:p} (allocated={})",
            self,
            self.allocated
        );
        self.new_objects.set_object(object);
    }

    /// Get the new objects in this block.
    pub fn get_new_objects(&mut self) -> HashSet<GCObjectRef> {
        let self_ptr = self as *mut BlockInfo;
        self.new_objects.as_hashset(self_ptr as *mut u8)
    }

    /// Remove all the new objects from the object map and clear the new
    /// objects set.
    pub fn remove_new_objects_from_map(&mut self) {
        self.object_map.difference(&self.new_objects);
        self.new_objects.clear();
    }

    /// Set as an evacuation candidate if this block has at least `hole_count`
    /// holes.
    pub fn set_evacuation_candidate(&mut self, hole_count: usize) {
        log::debug!(
            "Set block {:p} to evacuation_candidate={} ({} holes)",
            &self,
            self.hole_count >= hole_count,
            self.hole_count
        );
        self.evacuation_candidate = true //self.hole_count >= hole_count;
    }

    /// Return if this is an evacuation candidate.
    pub fn is_evacuation_candidate(&self) -> bool {
        self.evacuation_candidate
    }

    /// Increment the lines on which the object is allocated.
    pub fn increment_lines(&mut self, object: GCObjectRef) {
        self.update_line_nums(object, true);
    }

    /// Decrement the lines on which the object is allocated.
    pub fn decrement_lines(&mut self, object: GCObjectRef) {
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
        self.clear_object_map();
        self.allocated = false;
        self.hole_count = 0;
        self.evacuation_candidate = false;
    }

    /// Return true if no line is marked (every line has a count of zero).
    pub fn is_empty(&self) -> bool {
        self.line_counter.values().all(|v| *v == 0)
    }

    /// Get a pointer to an address `offset` bytes into this block.
    pub fn offset(&mut self, offset: usize) -> GCObjectRef {
        let self_ptr = self as *mut BlockInfo;
        let object = unsafe { (self_ptr as *mut u8).offset(offset as isize) };
        GCObjectRef::from_ptr(object as *mut WaffleCell)
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
        log::debug!(
            "Scanning block {:p} for a hole with last_high_offset {}",
            self,
            last_high_index
        );
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
            log::debug!("Rescan: Found single line hole? in block {:p}", self);
            return self.scan_block((high_index * LINE_SIZE - 1) as u16);
        } else if low_index < (NUM_LINES_PER_BLOCK - 1) {
            log::debug!(
                "Found low index {} and high index {} in block {:p}",
                low_index,
                high_index,
                self
            );
            return Some((
                (low_index * LINE_SIZE) as u16,
                (high_index * LINE_SIZE - 1) as u16,
            ));
        }
        log::debug!("Found no hole in block {:p}", self);
        None
    }

    /// Count the holes in this block.
    ///
    /// Holes are lines with no objects allocated.
    pub fn count_holes(&mut self) {
        let holes = self
            .line_counter
            .values()
            .fold((0, false), |(holes, in_hole), &elem| {
                match (in_hole, elem) {
                    (false, 0) => (holes + 1, true),
                    (_, _) => (holes, false),
                }
            })
            .0;
        self.hole_count = holes;
    }
}

impl BlockInfo {
    /// Returns true if this block is allocated and the address is within the
    /// bounds of this block.
    pub fn is_in_block(&self, object: GCObjectRef) -> bool {
        // This works because we get zeroed memory from the OS, so
        // self.allocated will be false if this block is not initialized and
        // this method gets only called for objects within the ImmixSpace.
        // After the first initialization the field is properly managed.
        if self.allocated {
            let self_ptr = self as *const BlockInfo as *const u8;
            let self_bound = unsafe { self_ptr.offset(BLOCK_SIZE as isize) };
            self_ptr < (object.raw() as *const u8) && (object.raw() as *const u8) < self_bound
        } else {
            false
        }
    }

    /// Convert an address on this block into a line number.
    pub fn object_to_line_num(object: GCObjectRef) -> usize {
        (object.raw() as usize % BLOCK_SIZE) / LINE_SIZE
    }

    /// Update the line counter for the given object.
    ///
    /// Increment if `increment`, otherwise do a saturating substraction.
    fn update_line_nums(&mut self, object: GCObjectRef, increment: bool) {
        // This calculates how many lines are affected starting from a
        // LINE_SIZE aligned address. So it might not mark enough lines. But
        // that does not matter as we always skip a line in scan_block()
        let line_num = BlockInfo::object_to_line_num(object);
        let object_size = object.size();
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
            if increment {
                log::debug!(
                    "Incremented line count for line {} to {}",
                    line,
                    self.line_counter.get(line).expect("a line count")
                );
            } else {
                log::debug!(
                    "Decremented line count for line {} to {}",
                    line,
                    self.line_counter.get(line).expect("a line count")
                );
            }
        }
    }
}
