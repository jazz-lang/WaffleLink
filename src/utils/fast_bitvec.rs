use std::sync::atomic::{AtomicU32, Ordering};

pub const fn fast_bit_vec_array_length(num_bits: usize) -> usize {
    return (num_bits + 31) / 32;
}

pub struct FastBitVectorWordView {
    words: *mut u32,
    num_bits: usize,
}

impl FastBitVectorWordView {
    pub fn word(&self, index: usize) -> u32 {
        unsafe { self.words.offset(index as _).read() }
    }

    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    pub fn new(array: &[u32], num_bits: usize) -> Self {
        Self {
            words: array.as_ptr() as *mut _,
            num_bits,
        }
    }

    pub fn as_slice(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.words, fast_bit_vec_array_length(self.num_bits)) }
    }

    pub fn as_slice_mut(&self) -> &mut [u32] {
        unsafe {
            std::slice::from_raw_parts_mut(self.words, fast_bit_vec_array_length(self.num_bits))
        }
    }
}

pub struct FastBitVectorWordOwner {
    words: *mut u32,
    num_bits: usize,
}

impl FastBitVectorWordOwner {
    pub fn view(&self) -> FastBitVectorWordView {
        FastBitVectorWordView {
            words: self.words,
            num_bits: self.num_bits,
        }
    }
    pub fn as_slice(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.words, fast_bit_vec_array_length(self.num_bits)) }
    }

    pub fn as_slice_mut(&self) -> &mut [u32] {
        unsafe {
            std::slice::from_raw_parts_mut(self.words, fast_bit_vec_array_length(self.num_bits))
        }
    }

    fn resize_slow(&mut self, num_bits: usize) {
        let new_len = fast_bit_vec_array_length(num_bits);
        unsafe {
            let new_array =
                std::alloc::alloc_zeroed(std::alloc::Layout::array::<u32>(new_len).unwrap())
                    .cast::<u32>();
            core::ptr::copy_nonoverlapping(self.words, new_array, self.array_len());
            if self.words.is_null() == false {
                std::alloc::dealloc(
                    self.words.cast(),
                    std::alloc::Layout::array::<u32>(fast_bit_vec_array_length(self.num_bits))
                        .unwrap(),
                );
            }
            self.words = new_array;
        }
    }

    pub fn array_len(&self) -> usize {
        fast_bit_vec_array_length(self.num_bits)
    }

    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    pub fn resize(&mut self, num_bits: usize) {
        if self.array_len() != fast_bit_vec_array_length(num_bits) {
            self.resize_slow(num_bits);
        }
        self.num_bits = num_bits;
    }

    pub fn word(&self, index: usize) -> u32 {
        unsafe { self.words.offset(index as _).read() }
    }

    pub fn word_mut(&mut self, index: usize) -> &mut u32 {
        unsafe { &mut *self.words.offset(index as _) }
    }

    pub fn clear_all(&self) {
        unsafe {
            core::ptr::write_bytes(self.words.cast::<u8>(), 0, self.array_len() * 4);
        }
    }

    pub fn set_all(&self) {
        unsafe {
            core::ptr::write_bytes(self.words.cast::<u8>(), 255, self.array_len() * 4);
        }
    }

    pub fn set(&self, other: &Self) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                other.words.cast::<u8>(),
                self.words.cast::<u8>(),
                self.array_len() * 4,
            );
        }
    }
}

impl Drop for FastBitVectorWordOwner {
    fn drop(&mut self) {
        if self.words.is_null() == false {
            unsafe {
                std::alloc::dealloc(
                    self.words.cast(),
                    std::alloc::Layout::array::<u32>(fast_bit_vec_array_length(self.num_bits))
                        .unwrap(),
                );
            }
        }
    }
}

impl Words for FastBitVectorWordOwner {
    type ViewType = Self;
    fn word_view(&self) -> &Self::ViewType {
        self
    }

    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        self
    }

    fn get_word(&self, n: usize) -> u32 {
        self.word(n)
    }

    fn get_word_mut(&mut self, n: usize) -> &mut u32 {
        self.word_mut(n)
    }

    fn get_num_bits(&self) -> usize {
        self.num_bits()
    }
}

pub trait Words {
    type ViewType: Words;
    fn get_num_bits(&self) -> usize;
    fn get_word(&self, n: usize) -> u32;
    fn get_word_mut(&mut self, n: usize) -> &mut u32;
    fn word_view(&self) -> &Self::ViewType;
    fn word_view_mut(&mut self) -> &mut Self::ViewType;
}

pub struct FastBitVectorImpl<W: Words> {
    pub words: W,
}

impl<W: Words> FastBitVectorImpl<W> {
    pub fn new() -> Self
    where
        W: Default,
    {
        Self {
            words: Default::default(),
        }
    }

    pub fn num_bits(&self) -> usize {
        self.words.get_num_bits()
    }

    pub fn size(&self) -> usize {
        self.words.get_num_bits()
    }

    pub fn array_length(&self) -> usize {
        fast_bit_vec_array_length(self.num_bits())
    }

    pub fn at(&self, index: usize) -> bool {
        self.at_impl(index)
    }
    pub fn bitcount(&self) -> usize {
        let mut result = 0;
        for i in 0..self.array_length() {
            result += bitcount(self.words.get_word(i));
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        for i in 0..self.array_length() {
            if self.words.get_word(i) != 0 {
                return false;
            }
        }
        true
    }
    pub fn or<U: Words>(&self, other: &U) -> FastBitVector {
        let mut bvec = FastBitVector::new();
        bvec.resize(self.num_bits());

        for i in 0..self.array_length() {
            *bvec.unsafe_words_mut().get_word_mut(i) = self.get_word(i) | other.get_word(i);
        }
        bvec
    }
    pub fn and<U: Words>(&self, other: &U) -> FastBitVector {
        let mut bvec = FastBitVector::new();
        bvec.resize(self.num_bits());

        for i in 0..self.array_length() {
            *bvec.unsafe_words_mut().get_word_mut(i) = self.get_word(i) & other.get_word(i);
        }
        bvec
    }
    pub fn unsafe_words(&self) -> &W {
        &self.words
    }

    pub fn unsafe_words_mut(&mut self) -> &mut W {
        &mut self.words
    }

    pub fn into_words(self) -> W {
        self.words
    }
    #[inline(always)]
    pub fn find_bit(&self, start_index: usize, value: bool) -> usize {
        let skip_value = (-((value as u32 ^ 1) as i32)) as u32;
        let num_words = fast_bit_vec_array_length(self.words.get_num_bits());
        let mut word_index = start_index / 32;
        let mut start_index_in_word = start_index - word_index * 32;
        while word_index < num_words {
            let word = self.words.get_word(word_index);
            if word != skip_value {
                let mut index = start_index_in_word;
                if let Some(index) = find_bit_in_word(word, index, 32, value) {
                    return word_index * 32 + index;
                }
            }
            word_index += 1;
            start_index_in_word = 0;
        }
        self.num_bits()
    }

    #[inline(always)]
    pub fn find_set_bit(&self, index: usize) -> usize {
        self.find_bit(index, true)
    }
    #[inline(always)]
    pub fn find_clear_bit(&self, index: usize) -> usize {
        self.find_bit(index, false)
    }
    #[inline(always)]
    pub fn for_each_set_bit(&self, mut func: impl FnMut(usize)) {
        let n = self.array_length();
        for i in 0..n {
            let mut word = self.words.get_word(i);
            let mut j = i * 32;
            while word != 0 {
                if (word & 1) != 0 {
                    func(j);
                }
                word >>= 1;
                j += 1;
            }
        }
    }
    #[inline(always)]
    pub fn for_each_clear_bit(&self, mut func: impl FnMut(usize)) {
        (!self).for_each_set_bit(func)
    }
    fn at_impl(&self, index: usize) -> bool {
        (self.words.get_word(index >> 5) & (1 << (index & 31))) != 0
    }
}

use std::fmt;

impl<W: Words> fmt::Debug for FastBitVectorImpl<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.num_bits() {
            write!(f, "{}", if self.at(i) { "1" } else { "-" })?
        }
        Ok(())
    }
}

impl<W: Words, U: Words> PartialEq<U> for FastBitVectorImpl<W> {
    fn eq(&self, other: &U) -> bool {
        if self.num_bits() != other.get_num_bits() {
            return false;
        }
        for i in 0..self.array_length() {
            if self.words.get_word(i) != other.get_word(i) {
                return false;
            }
        }
        true
    }
}
use std::ops;

impl<'a, W: Words> ops::Not for &'a FastBitVectorImpl<W> {
    type Output = FastBitVectorImpl<FastBitVectorNotWords<&'a W::ViewType>>;
    fn not(self) -> Self::Output {
        FastBitVectorImpl {
            words: FastBitVectorNotWords::<&W::ViewType> {
                view: self.word_view(),
            },
        }
    }
}

impl<W: Words, U: Words> ops::BitOr<U> for FastBitVectorImpl<W> {
    type Output = FastBitVectorImpl<FastBitVectorOrWords<W, U>>;
    fn bitor(self, rhs: U) -> Self::Output {
        FastBitVectorImpl {
            words: FastBitVectorOrWords {
                left: self.words,
                right: rhs,
            },
        }
    }
}
impl<W: Words, U: Words> ops::BitAnd<U> for FastBitVectorImpl<W> {
    type Output = FastBitVectorImpl<FastBitVectorAndWords<W, U>>;
    fn bitand(self, rhs: U) -> Self::Output {
        FastBitVectorImpl {
            words: FastBitVectorAndWords {
                left: self.words,
                right: rhs,
            },
        }
    }
}

impl<'a, W: Words> Words for &'a mut W {
    type ViewType = W::ViewType;
    fn get_word_mut(&mut self, n: usize) -> &mut u32 {
        (**self).get_word_mut(n)
    }

    fn get_word(&self, n: usize) -> u32 {
        (**self).get_word(n)
    }

    fn get_num_bits(&self) -> usize {
        (**self).get_num_bits()
    }
    fn word_view(&self) -> &Self::ViewType {
        (**self).word_view()
    }

    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        (**self).word_view_mut()
    }
}

impl<'a, W: Words> Words for &'a W {
    type ViewType = W::ViewType;
    fn get_word_mut(&mut self, n: usize) -> &mut u32 {
        unreachable!("Immutable words");
    }

    fn word_view(&self) -> &Self::ViewType {
        (**self).word_view()
    }
    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        unreachable!("Immutable words");
    }

    fn get_word(&self, n: usize) -> u32 {
        (**self).get_word(n)
    }

    fn get_num_bits(&self) -> usize {
        (**self).get_num_bits()
    }
}

impl<W: Words + Default> Default for FastBitVectorImpl<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Words> Words for FastBitVectorImpl<W> {
    type ViewType = W::ViewType;
    fn get_num_bits(&self) -> usize {
        self.words.get_num_bits()
    }

    fn get_word(&self, n: usize) -> u32 {
        self.words.get_word(n)
    }

    fn get_word_mut(&mut self, n: usize) -> &mut u32 {
        self.words.get_word_mut(n)
    }

    fn word_view(&self) -> &Self::ViewType {
        self.words.word_view()
    }
    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        self.words.word_view_mut()
    }
}

impl<W: Words> std::ops::Deref for FastBitVectorImpl<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.words
    }
}

impl<W: Words> std::ops::DerefMut for FastBitVectorImpl<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.words
    }
}

#[inline]
pub const fn bitcount(bits: u32) -> usize {
    let bits = (bits >> 1) & 0x55555555;
    let bits = (bits & 0x33333333) + ((bits >> 2) & 0x33333333);
    ((((bits + (bits >> 4)) & 0xF0F0F0F) * 0x1010101) >> 24) as usize
}

#[inline]
pub const fn bitcount64(bits: u64) -> usize {
    bitcount(bits as u32) + bitcount((bits >> 32) as u32)
}

pub fn find_bit_in_word(
    mut word: u32,
    start_index: usize,
    end_index: usize,
    value: bool,
) -> Option<usize> {
    let mut index = start_index;
    word >>= index;
    // We should only use trailing_zeros() when we know that trailing_zeros() is implementated using
    // a fast hardware instruction. Otherwise, this will actually result in
    // worse performance.
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        word ^= value as u32 - 1;
        index += word.trailing_zeros() as usize;
        if index < end_index {
            return Some(index);
        }
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        while index < end_index {
            if (word & 1) == value as u32 {
                return Some(index);
            }
            index += 1;
            word >>= 1;
        }
    }
    None
}

pub struct FastBitVectorNotWords<W: Words> {
    pub view: W,
}

impl<W: Words> FastBitVectorNotWords<W> {
    pub fn num_bits(&self) -> usize {
        self.view.get_num_bits()
    }

    pub fn word(&self, i: usize) -> u32 {
        !self.view.get_word(i)
    }
}

impl<W: Words> Words for FastBitVectorNotWords<W> {
    type ViewType = W::ViewType;
    fn get_num_bits(&self) -> usize {
        self.view.get_num_bits()
    }

    fn get_word(&self, n: usize) -> u32 {
        !self.view.get_word(n)
    }

    fn get_word_mut(&mut self, n: usize) -> &mut u32 {
        self.view.get_word_mut(n)
    }

    fn word_view(&self) -> &Self::ViewType {
        self.view.word_view()
    }
    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        self.view.word_view_mut()
    }
}

impl Default for FastBitVectorWordOwner {
    fn default() -> Self {
        Self {
            words: core::ptr::null_mut(),
            num_bits: 0,
        }
    }
}

pub struct FastBitVector {
    impl_: FastBitVectorImpl<FastBitVectorWordOwner>,
}

impl FastBitVector {
    pub fn new() -> Self {
        Self {
            impl_: Default::default(),
        }
    }

    pub fn resize(&mut self, num_bits: usize) {
        self.impl_.resize(num_bits);
    }
    pub fn num_bits(&self) -> usize {
        self.impl_.get_num_bits()
    }

    pub fn array_len(&self) -> usize {
        fast_bit_vec_array_length(self.num_bits())
    }
    pub fn clear_all(&mut self) {
        self.impl_.clear_all();
    }
    pub fn set_all(&mut self) {
        self.impl_.set_all();
    }
    pub fn fill(&mut self, value: bool) {
        if value {
            self.set_all();
        } else {
            self.clear_all();
        }
    }

    pub fn grow(&mut self, num_bits: usize) {
        self.resize(num_bits);
    }

    pub fn at(&self, index: usize) -> bool {
        self.impl_.at_impl(index)
    }

    pub fn set_at(&mut self, index: usize, value: bool) {
        let word = &mut self.impl_.words.as_slice_mut()[index >> 5];
        let mask = 1 << (index & 31);

        if value {
            *word |= mask;
        } else {
            *word &= !mask;
        }
    }
    /// Returns true if the contents changed.
    pub fn atomic_set_and_check(&self, index: usize, value: bool) -> bool {
        let entry =
            unsafe { &*(&self.impl_.as_slice()[index >> 5] as *const u32 as *const AtomicU32) };
        let mask = 1 << (index & 31);
        let mut old_value = self.impl_.as_slice()[index >> 5];
        loop {
            let new_value = if value {
                if (old_value & mask) != 0 {
                    return false;
                }
                old_value | mask
            } else {
                if (old_value & mask) == 0 {
                    return false;
                }
                old_value & !mask
            };
            match entry.compare_exchange_weak(
                old_value,
                new_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => {
                    old_value = x;
                }
            }
        }
        true
    }

    pub fn or<U: Words>(&self, other: &U) -> FastBitVector {
        let mut bvec = FastBitVector::new();
        bvec.resize(self.num_bits());

        for i in 0..self.array_len() {
            *bvec.unsafe_words_mut().get_word_mut(i) = self.get_word(i) | other.get_word(i);
        }
        bvec
    }
    pub fn and<U: Words>(&self, other: &U) -> FastBitVector {
        let mut bvec = FastBitVector::new();
        bvec.resize(self.num_bits());

        for i in 0..self.array_len() {
            *bvec.unsafe_words_mut().get_word_mut(i) = self.get_word(i) & other.get_word(i);
        }
        bvec
    }
}

impl<W: Words> ops::BitOrAssign<W> for FastBitVector {
    fn bitor_assign(&mut self, rhs: W) {
        for i in 0..self.impl_.array_len() {
            *self.impl_.word_mut(i) |= rhs.get_word(i);
        }
    }
}
impl<W: Words> ops::BitAndAssign<W> for FastBitVector {
    fn bitand_assign(&mut self, rhs: W) {
        for i in 0..self.impl_.array_len() {
            *self.impl_.word_mut(i) &= rhs.get_word(i);
        }
    }
}

impl fmt::Debug for FastBitVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.impl_)
    }
}

impl<W: Words> PartialEq<W> for FastBitVector {
    fn eq(&self, other: &W) -> bool {
        self.impl_.eq(other)
    }
}
impl Eq for FastBitVector {}

impl Words for FastBitVector {
    type ViewType = Self;
    fn word_view(&self) -> &Self::ViewType {
        self
    }

    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        self
    }

    fn get_word(&self, n: usize) -> u32 {
        self.impl_.get_word(n)
    }

    fn get_word_mut(&mut self, n: usize) -> &mut u32 {
        self.impl_.get_word_mut(n)
    }

    fn get_num_bits(&self) -> usize {
        self.impl_.get_num_bits()
    }
}

impl ops::Deref for FastBitVector {
    type Target = FastBitVectorImpl<FastBitVectorWordOwner>;
    fn deref(&self) -> &Self::Target {
        &self.impl_
    }
}
impl ops::DerefMut for FastBitVector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.impl_
    }
}

pub struct FastBitReference {
    word: *mut u32,
    mask: u32,
}

impl FastBitReference {
    pub fn bool(&self) -> bool {
        unsafe { (*self.word & self.mask) != 0 }
    }
}

impl<'a> ops::BitXorAssign<bool> for FastBitReference {
    fn bitxor_assign(&mut self, rhs: bool) {
        if rhs {
            unsafe {
                *self.word |= self.mask;
            }
        } else {
            unsafe {
                *self.word &= !self.mask;
            }
        }
    }
}

impl ops::BitAndAssign<bool> for FastBitReference {
    fn bitand_assign(&mut self, rhs: bool) {
        if rhs {
            *self ^= rhs;
        }
    }
}

impl ops::BitOrAssign<bool> for FastBitReference {
    fn bitor_assign(&mut self, rhs: bool) {
        if !rhs {
            *self ^= rhs;
        }
    }
}

pub struct FastBitVectorOrWords<T: Words, U: Words> {
    left: T,
    right: U,
}

impl<T: Words, U: Words> Words for FastBitVectorOrWords<T, U> {
    fn get_num_bits(&self) -> usize {
        self.left.get_num_bits()
    }

    type ViewType = Self;

    fn get_word(&self, n: usize) -> u32 {
        self.left.get_word(n) | self.right.get_word(n)
    }
    fn get_word_mut(&mut self, _: usize) -> &mut u32 {
        unreachable!()
    }

    fn word_view(&self) -> &Self::ViewType {
        self
    }

    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        self
    }
}
pub struct FastBitVectorAndWords<T: Words, U: Words> {
    left: T,
    right: U,
}

impl<T: Words, U: Words> Words for FastBitVectorAndWords<T, U> {
    fn get_num_bits(&self) -> usize {
        self.left.get_num_bits()
    }

    type ViewType = Self;

    fn get_word(&self, n: usize) -> u32 {
        self.left.get_word(n) & self.right.get_word(n)
    }
    fn get_word_mut(&mut self, _: usize) -> &mut u32 {
        unreachable!()
    }

    fn word_view(&self) -> &Self::ViewType {
        self
    }

    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        self
    }
}
