use std::iter::IntoIterator;
use std::ops::{Index, IndexMut};
use std::slice::Iter as SliceIter;

pub struct SegmentedVec<T> {
    chunks: Vec<Vec<T>>,
    chunk_size: usize,
}

impl<T> SegmentedVec<T> {
    /// Constructs a new, empty SegmentedVec<T> with a default chunk size of 256.
    ///
    /// The segmented vector will not allocate until elements are pushed onto it.
    pub fn new() -> Self {
        SegmentedVec::with_chunk_size(256)
    }

    /// Constructs a new, empty SegmentedVec<T> with the provided chunk size.
    ///
    /// The segmented vector will not allocate until elements are pushed onto it.
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        SegmentedVec {
            chunks: Vec::new(),
            chunk_size,
        }
    }

    /// Appends an element to the back of a collection.
    pub fn push(&mut self, val: T) {
        let mut new_chunk = true;
        if let Some(chunk) = self.chunks.last() {
            new_chunk = chunk.len() >= self.chunk_size;
        }

        if new_chunk {
            self.chunks.push(Vec::with_capacity(self.chunk_size));
        }

        self.chunks.last_mut().unwrap().push(val)
    }

    /// Removes the last element from a vector and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        loop {
            match self.chunks.last_mut() {
                Some(chunk) => {
                    let popped = chunk.pop();
                    if popped.is_some() {
                        return popped;
                    }
                }
                None => {
                    return None;
                }
            }
            self.chunks.pop();
        }
    }

    /// Clears the vector, removing all values.
    ///
    /// This method deallocates the chunks of the segmented vector.
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Returns the number of elements in the segmented vector, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        match self.chunks.last() {
            Some(chunk) => (self.chunks.len() - 1) * self.chunk_size + chunk.len(),
            None => 0,
        }
    }

    /// Returns a reference to an element at the provided index if it exists.
    pub fn get(&self, idx: usize) -> Option<&T> {
        let c = idx / self.chunk_size;
        let sub_idx = idx % self.chunk_size;
        if let Some(chunk) = self.chunks.get(c) {
            return chunk.get(sub_idx);
        }

        None
    }

    /// Returns a mutable reference to an element at the provided index if it exists.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        let c = idx / self.chunk_size;
        let sub_idx = idx % self.chunk_size;
        if let Some(chunk) = self.chunks.get_mut(c) {
            return chunk.get_mut(sub_idx);
        }

        None
    }

    /// Return size of the `nth` allocated chunk in the segmented vector.
    pub fn chunk(&self, nth: usize) -> Option<&[T]> {
        self.chunks.get(nth).map(|chunk| &chunk[..])
    }

    /// Returns an iterator over the segmented vector.
    pub fn iter<'l>(&'l self) -> Iter<'l, T> {
        Iter {
            chunks: self.chunks.iter(),
            current: [].iter(),
        }
    }
}

impl<T> Index<usize> for SegmentedVec<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        self.get(idx).unwrap()
    }
}

impl<T> IndexMut<usize> for SegmentedVec<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        self.get_mut(idx).unwrap()
    }
}

/// An iterator over a `SegmentedVector<T>`
pub struct Iter<'l, T> {
    chunks: SliceIter<'l, Vec<T>>,
    current: SliceIter<'l, T>,
}

impl<'l, T> Iterator for Iter<'l, T> {
    type Item = &'l T;
    fn next(&mut self) -> Option<&'l T> {
        if let Some(v) = self.current.next() {
            return Some(v);
        }

        if let Some(chunk) = self.chunks.next() {
            self.current = chunk.iter();
        } else {
            return None;
        }

        self.next()
    }
}

impl<'l, T> IntoIterator for &'l SegmentedVec<T> {
    type Item = &'l T;
    type IntoIter = Iter<'l, T>;
    fn into_iter(self) -> Iter<'l, T> {
        self.iter()
    }
}

#[test]
fn test_basic() {
    let mut v = SegmentedVec::with_chunk_size(8);
    let n = 100usize;
    for i in 0..n {
        v.push(i);
    }
    assert_eq!(v.len(), 100);

    for i in 0..n {
        assert_eq!(*v.get(i).unwrap(), i);
    }

    let mut i = 0;
    for val in &v {
        assert_eq!(*val, i);
        i += 1;
    }
    assert_eq!(i, n);

    assert!(v.get(n).is_none());

    for i in 0..(n + 10) {
        if i < n {
            assert_eq!(v.pop(), Some(n - 1 - i));
            assert_eq!(v.len(), n - i - 1);
        } else {
            assert_eq!(v.pop(), None);
            assert_eq!(v.len(), 0);
        }
    }

    assert_eq!(v.len(), 0);
}

pub fn unaligned_store<T: Copy>(pointer: *mut u8, value: T) {
    unsafe {
        std::ptr::write_unaligned(pointer.cast::<T>(), value);
    }
}

pub fn unaligned_load<T: Copy>(pointer: *mut u8) -> T {
    unsafe { std::ptr::read_unaligned(pointer.cast::<T>()) }
}
