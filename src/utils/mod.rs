#![allow(missing_docs)]
pub mod atomics;
pub mod fast_bitvec;
pub mod linked_list;
pub mod segmented_vec;
pub mod uniset;
pub trait Bool {
    const RES: bool;
}

pub struct True;
pub struct False;

impl Bool for True {
    const RES: bool = true;
}
impl Bool for False {
    const RES: bool = false;
}
/// Trait that provides `retain_mut` method.
pub trait RetainMut<T> {
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool;
}

impl<T> RetainMut<T> for Vec<T> {
    // The implementation is based on
    // https://github.com/rust-lang/rust/blob/0eb878d2aa6e3a1cb315f3f328681b26bb4bffdb/src/liballoc/vec.rs#L1072-L1093
    fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&mut v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }
}
