macro_rules! impl_t {
    ($($t: ident $k : ident),*) => {
        $(
            impl DirectoryBit for $t {
                const KIND: BlockDirectoryKind = BlockDirectoryKind::$k;
            }
        )*
    };
}

use crate::utils::fast_bitvec::*;

/* The set of block indices that have actual blocks. */
pub struct LiveDirectoryBit;
pub struct EmptyDirectoryBit;
pub struct AllocatedDirectoryBit;
pub struct CanAllocateButNotEmptyDirectoryBit;
pub struct DestructibleDirectoryBit;
pub struct EdenDirectoryBit;
pub struct UnsweptDirectoryBit;
pub struct MarkingNotEmptyDirectoryBit;
pub struct MarkingRetiredDirectoryBit;

pub trait DirectoryBit {
    const KIND: BlockDirectoryKind;
}

impl_t!(
    LiveDirectoryBit Live,
    EmptyDirectoryBit Empty,
    AllocatedDirectoryBit Allocated,
    CanAllocateButNotEmptyDirectoryBit CanAllocateButNotEmpty,
    DestructibleDirectoryBit Destructible,
    EdenDirectoryBit Eden,
    UnsweptDirectoryBit Unswept,
    MarkingNotEmptyDirectoryBit MarkingNotEmpty,
    MarkingRetiredDirectoryBit MarkingRetired
);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub enum BlockDirectoryKind {
    /* The set of block indices that have actual blocks. */
    Live = 0,
    /* The set of all blocks that have no live objects. */
    Empty,
    /* The set of all blocks that are full of live objects. */
    Allocated,
    /* The set of all blocks are neither empty nor retired (i.e. are more than minMarkedBlockUtilization full). */
    CanAllocateButNotEmpty,
    /* The set of all blocks that may have destructors to run. */
    Destructible,
    /* The set of all blocks that have new objects since the last GC. */
    Eden,
    /* The set of all blocks that could be swept by the incremental sweeper. */
    Unswept,
    /* These are computed during marking. */

    /* The set of all blocks that are not empty. */
    MarkingNotEmpty,
    /* The set of all blocks that are retired. */
    MarkingRetired,
}

pub const BITS_PER_SEGMENT: usize = 32;
pub const SEGMENT_SHIFT: usize = 5;
pub const INDEX_MASK: u32 = (1 << SEGMENT_SHIFT) - 1;

pub const NUMBER_OF_BLOCK_DIRECTORY_BITS: usize = 9;

#[derive(Default, Clone)]
pub struct Segment {
    data: [u32; NUMBER_OF_BLOCK_DIRECTORY_BITS],
}

pub struct BlockDirectoryBitVectorWordView<K: DirectoryBit> {
    segments: *mut Segment,
    num_bits: usize,
    _marker: std::marker::PhantomData<K>,
}

impl<K: DirectoryBit> BlockDirectoryBitVectorWordView<K> {
    pub fn new() -> Self {
        Self {
            segments: core::ptr::null_mut(),
            num_bits: 0,
            _marker: Default::default(),
        }
    }

    pub fn clear_all(&self) {
        for i in 0..fast_bit_vec_array_length(self.num_bits) {
            unsafe {
                (&mut *self.segments.offset(i as _)).data[K::KIND as usize] = 0;
            }
        }
    }
}

impl<K: DirectoryBit> Words for BlockDirectoryBitVectorWordView<K> {
    type ViewType = Self;
    fn word_view(&self) -> &Self::ViewType {
        self
    }

    fn word_view_mut(&mut self) -> &mut Self::ViewType {
        self
    }

    fn get_word(&self, n: usize) -> u32 {
        unsafe { (&*self.segments.offset(n as _)).data[K::KIND as usize] }
    }

    fn get_word_mut(&mut self, n: usize) -> &mut u32 {
        unsafe { &mut (&mut *self.segments.offset(n as _)).data[K::KIND as usize] }
    }
    fn get_num_bits(&self) -> usize {
        self.num_bits
    }
}
impl<K: DirectoryBit> Default for BlockDirectoryBitVectorWordView<K> {
    fn default() -> Self {
        Self::new()
    }
}

pub type BlockDirectoryBitVectorView<K> = FastBitVectorImpl<BlockDirectoryBitVectorWordView<K>>;

pub struct BlockDirectoryBitVectorRef<K: DirectoryBit> {
    base: BlockDirectoryBitVectorView<K>,
}

impl<K: DirectoryBit> BlockDirectoryBitVectorRef<K> {
    pub fn new(view: BlockDirectoryBitVectorView<K>) -> Self {
        Self { base: view }
    }

    pub fn set<U: Words>(&mut self, other: &FastBitVectorImpl<U>) {
        for i in 0..self.base.array_length() {
            *self.base.unsafe_words_mut().get_word_mut(i) = other.unsafe_words().get_word(i);
        }
    }

    pub fn set_at(&mut self, index: usize, value: bool) {
        let word = self.base.unsafe_words_mut().get_word_mut(index >> 5);
        let mask = 1 << (index % 31);
        if value {
            *word |= mask;
        } else {
            *word &= !mask;
        }
    }

    pub fn at(&self, index: usize) -> bool {
        let word = self.base.unsafe_words().get_word(index >> 5);
        let mask = 1 << (index % 31);
        (word & mask) != 0
    }

    pub fn clear_all(&mut self) {
        self.base.clear_all();
    }
}

use std::ops;

impl<K: DirectoryBit, G: Words, U: ops::Deref<Target = FastBitVectorImpl<G>>> ops::BitOrAssign<U>
    for BlockDirectoryBitVectorRef<K>
{
    fn bitor_assign(&mut self, rhs: U) {
        for i in 0..self.base.array_length() {
            *self.base.unsafe_words_mut().get_word_mut(i) |= rhs.unsafe_words().get_word(i);
        }
    }
}

pub struct BlockDirectoryBits {
    segments: Vec<Segment>,
    pub num_bits: usize,
}

impl BlockDirectoryBits {
    pub fn new() -> Self {
        Self {
            segments: vec![],
            num_bits: 0,
        }
    }

    pub fn num_bits(&self) -> usize {
        self.num_bits
    }
    pub fn resize(&mut self, num_bits: usize) {
        let mut old_num_bits = self.num_bits();
        self.num_bits = num_bits;
        self.segments
            .resize(fast_bit_vec_array_length(num_bits), Segment::default());

        let used_bits_in_last_segment = num_bits & INDEX_MASK as usize;
        if num_bits < old_num_bits && used_bits_in_last_segment != 0 {
            let segment = self.segments.last_mut().unwrap();
            let mask = (1 << used_bits_in_last_segment) - 1;
            for index in 0..NUMBER_OF_BLOCK_DIRECTORY_BITS {
                segment.data[index] &= mask;
            }
        }
    }

    pub fn live(&self) -> BlockDirectoryBitVectorRef<LiveDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<LiveDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn empty(&self) -> BlockDirectoryBitVectorRef<EmptyDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<EmptyDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn allocated(&self) -> BlockDirectoryBitVectorRef<AllocatedDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<AllocatedDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn can_allocate_but_not_empty(
        &self,
    ) -> BlockDirectoryBitVectorRef<CanAllocateButNotEmptyDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<CanAllocateButNotEmptyDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn destructible(&self) -> BlockDirectoryBitVectorRef<DestructibleDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<DestructibleDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn eden(&self) -> BlockDirectoryBitVectorRef<EdenDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<EdenDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn unswept(&self) -> BlockDirectoryBitVectorRef<UnsweptDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<UnsweptDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn marking_not_empty(&self) -> BlockDirectoryBitVectorRef<MarkingNotEmptyDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<MarkingNotEmptyDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn marking_retired(&self) -> BlockDirectoryBitVectorRef<MarkingRetiredDirectoryBit> {
        BlockDirectoryBitVectorRef {
            base: FastBitVectorImpl {
                words: BlockDirectoryBitVectorWordView::<MarkingRetiredDirectoryBit> {
                    _marker: Default::default(),
                    segments: self.segments.as_ptr() as *mut _,
                    num_bits: self.num_bits,
                },
            },
        }
    }

    pub fn set_is_live(&mut self, index: usize, value: bool) {
        self.live().set_at(index, value)
    }

    pub fn set_is_empty(&mut self, index: usize, value: bool) {
        self.empty().set_at(index, value);
    }

    pub fn set_is_can_allocate_but_not_empty(&mut self, index: usize, value: bool) {
        self.can_allocate_but_not_empty().set_at(index, value);
    }

    pub fn set_is_destructible(&mut self, index: usize, value: bool) {
        self.destructible().set_at(index, value);
    }
    pub fn set_is_unwept(&mut self, index: usize, value: bool) {
        self.unswept().set_at(index, value)
    }

    pub fn set_is_marking_not_empty(&mut self, index: usize, value: bool) {
        self.marking_not_empty().set_at(index, value)
    }

    pub fn set_is_marking_retired(&mut self, index: usize, value: bool) {
        self.marking_retired().set_at(index, value)
    }

    pub fn is_live(&self, index: usize) -> bool {
        self.live().at(index)
    }

    pub fn is_empty(&self, index: usize) -> bool {
        self.empty().at(index)
    }

    pub fn is_can_allocate_but_not_empty(&self, index: usize) -> bool {
        self.can_allocate_but_not_empty().at(index)
    }

    pub fn is_destructible(&self, index: usize) -> bool {
        self.destructible().at(index)
    }

    pub fn is_eden(&self, index: usize) -> bool {
        self.eden().at(index)
    }

    pub fn is_unswept(&self, index: usize) -> bool {
        self.unswept().at(index)
    }

    pub fn is_marking_not_empty(&self, index: usize) -> bool {
        self.marking_not_empty().at(index)
    }

    pub fn is_marking_retired(&self, index: usize) -> bool {
        self.marking_retired().at(index)
    }

    pub fn for_each_segment(&self, mut func: impl FnMut(usize, &Segment)) {
        let mut index = 0;
        for segment in self.segments.iter() {
            func(index, segment);
            index += 1;
        }
    }
    pub fn for_each_segment_mut(&mut self, mut func: impl FnMut(usize, &mut Segment)) {
        let mut index = 0;
        for segment in self.segments.iter_mut() {
            func(index, segment);
            index += 1;
        }
    }
}
