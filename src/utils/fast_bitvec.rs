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
