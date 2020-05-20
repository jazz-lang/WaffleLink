use smallvec::SmallVec;

const WORDS: usize = ((255 + 1) + 4 * 8 - 1) / (4 * 8);
pub const WORD_SIZE: usize = 4 * 8;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct BitMap {
    bits: SmallVec<[u32; WORDS]>,
}
const ONE: u32 = 1;
impl BitMap {
    pub fn new() -> Self {
        Self {
            bits: SmallVec::new(),
        }
    }
    pub fn size(&self) -> usize {
        256
    }

    pub fn get(&self, n: usize) -> bool {
        let x = self.bits[n as usize / WORD_SIZE] & (1 << (n as usize % WORD_SIZE));
        x != 0
    }

    pub fn set(&mut self, n: usize) {
        self.bits[n / WORD_SIZE] |= (1 << (n % WORD_SIZE));
    }

    pub fn set_val(&mut self, n: usize, val: bool) {
        if val {
            self.set(n);
        } else {
            self.clear(n);
        }
    }

    pub fn test_and_set(&mut self, n: usize) -> bool {
        let mask = 1 << (n % WORD_SIZE);
        let index = n / WORD_SIZE;
        let result = self.bits[index] & mask;
        self.bits[index] |= mask;
        result != 0
    }

    pub fn test_and_clear(&mut self, n: usize) -> bool {
        let mask = 1 << (n % WORD_SIZE);
        let index = n / WORD_SIZE;
        let result = self.bits[index] & mask;
        self.bits[index] &= !mask;
        result != 0
    }

    pub fn next_possible_unset(&mut self, start: usize) -> usize {
        if self.bits[start / WORD_SIZE] == 0 {
            return ((start / WORD_SIZE) + 1) * WORD_SIZE;
        }
        return start + 1;
    }

    pub fn find_run_of_zeros(&self, mut run_len: usize) -> i64 {
        if run_len == 0 {
            run_len = 1;
        }
        for i in 0..=(256 - run_len) {
            let mut found = true;
            for j in i..(i + run_len - 1) {
                if self.get(j as _) {
                    found = true;
                }
            }
            if found {
                return i as _;
            }
        }
        return -1;
    }

    pub fn count(&self, mut start: usize) -> usize {
        let mut result = 0;
        while start % WORD_SIZE != 0 {
            if self.get(start) {
                result += 1;
            }
            start += 1;
        }
        for i in start / WORD_SIZE..WORDS {
            result += bitcnt(self.bits[i]) as usize;
        }
        return result;
    }
    pub fn is_empty(&self) -> bool {
        for i in 0..WORDS {
            if self.bits[i] != 0 {
                return false;
            }
        }
        true
    }

    pub fn is_full(&self) -> bool {
        for i in 0..WORDS {
            if !self.bits[i] != 0 {
                return false;
            }
        }
        true
    }
    pub fn merge(&mut self, other: &Self) {
        for i in 0..WORDS {
            self.bits[i] |= other.bits[i];
        }
    }
    pub fn exlude(&mut self, other: &Self) {
        for i in 0..WORDS {
            self.bits[i] &= !other.bits[i];
        }
    }

    pub fn clear(&mut self, n: usize) {
        self.bits[n / WORD_SIZE] &= !(1 << (n % WORD_SIZE));
    }
    pub fn clear_all(&mut self) {
        self.bits.iter_mut().for_each(|x| *x = 0);
    }
    pub fn subsumes(&self, other: &Self) -> bool {
        for i in 0..WORDS {
            let mybits = self.bits[i];
            let otherbits = other.bits[i];
            if (mybits | otherbits) != mybits {
                return false;
            }
        }
        return true;
    }
    pub fn for_each_set_bit(&self, mut f: impl FnMut(usize)) {
        for i in 0..WORDS {
            let mut word = self.bits[i];
            if word == 0 {
                continue;
            }
            let base = i * WORD_SIZE;
            for j in 0..WORD_SIZE {
                if (word & 1) != 0 {
                    f(base + j);
                }
                word >>= 1;
            }
        }
    }

    pub fn filter(&mut self, other: &Self) {
        for i in 0..WORDS {
            self.bits[i] &= other.bits[i];
        }
    }
}

fn bitcnt(x: u32) -> u32 {
    (!x).leading_zeros()
}
