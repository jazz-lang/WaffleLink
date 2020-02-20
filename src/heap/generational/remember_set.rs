use crate::runtime::cell::CellPointer;
use crate::util::ptr::Ptr;
use std::mem;

const CHUNK_VALUES: usize = 4;
pub struct Chunk {
    next: Option<Box<Chunk>>,
    index: usize,
    values: [CellPointer; CHUNK_VALUES],
}

fn compare_and_swap<T: PartialEq + Copy>(x: &mut T, current: T, next: T) -> T {
    if *x == current {
        std::mem::replace(x, next)
    } else {
        *x
    }
}

impl Chunk {
    pub fn boxed() -> (Box<Self>, *mut Chunk) {
        let chunk = Chunk {
            next: None,
            index: 0,
            values: [CellPointer {
                raw: crate::util::tagged::TaggedPointer::null(),
            }; CHUNK_VALUES],
        };
        let boxed = Box::new(chunk);
        let ptr = &*boxed as *const _ as *mut _;

        (boxed, ptr)
    }
    pub fn remember(&mut self, cell: CellPointer) -> bool {
        if self.index == CHUNK_VALUES {
            return false;
        }

        self.index += 1;
        let i = self.index;
        self.values[i] = cell;
        true
    }
}

/// A collection of pointers to mature/intermediate objects that contain pointers to young
/// objects.
///
/// Values can be added to a remembered set, and an iterator can be obtained to
/// iterate over these values. Removing individual values is not supported,
/// instead one must prune the entire remembered set.
pub struct RemembrSet {
    head: Box<Chunk>,
    tail: Ptr<Chunk>,
}

impl RemembrSet {
    pub fn new() -> Self {
        let (head, tail) = Chunk::boxed();
        Self {
            head,
            tail: Ptr { raw: tail },
        }
    }
    pub fn remember(&mut self, value: CellPointer) {
        loop {
            let tail_ptr = self.tail.get();

            if tail_ptr.remember(value) {
                return;
            }

            let (chunk, new_tail_ptr) = Chunk::boxed();
            tail_ptr.next = Some(chunk);
            self.tail = Ptr { raw: new_tail_ptr };
        }
    }

    /// Returns an iterator over the pointers in the remembered set.
    ///
    /// This method takes a mutable reference to `self` as iteration can not
    /// take place when the set is modified concurrently.
    pub fn iter(&self) -> RememberedSetIterator {
        RememberedSetIterator {
            chunk: &*self.head,
            index: 0,
        }
    }

    /// Prunes the remembered set by removing pointers to unmarked objects.
    pub fn prune(&mut self) {
        let (mut head, tail) = Chunk::boxed();

        // After this `head` is the old head, and `self.head` will be an
        // empty chunk.
        mem::swap(&mut head, &mut self.head);
        self.tail = Ptr { raw: tail };

        let mut current = Some(head);

        while let Some(mut chunk) = current {
            for value in &chunk.values {
                if value.raw.raw.is_null() {
                    // Once we encounter a NULL value there can not be any
                    // non-NULL values that follow it.
                    break;
                }

                if !value.is_marked() {
                    // Pointers that are not marked should no longer be
                    // remembered.
                    continue;
                }

                self.remember(*value);
            }

            current = chunk.next.take();
        }
    }

    /// Returns `true` if this RememberedSet is empty.
    pub fn is_empty(&self) -> bool {
        self.head.values[0].raw.raw.is_null()
    }
}

pub struct RememberedSetIterator<'a> {
    chunk: &'a Chunk,
    index: usize,
}

impl<'a> Iterator for RememberedSetIterator<'a> {
    type Item = &'a CellPointer;

    fn next(&mut self) -> Option<&'a CellPointer> {
        if self.index == CHUNK_VALUES {
            if let Some(chunk) = self.chunk.next.as_ref() {
                self.chunk = chunk;
                self.index = 0;
            } else {
                return None;
            }
        }

        let value = &self.chunk.values[self.index];

        if value.raw.raw.is_null() {
            None
        } else {
            self.index += 1;

            Some(value)
        }
    }
}
