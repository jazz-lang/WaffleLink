use crate::common::mem::Address;

#[derive(Copy, Clone, PartialEq, Eq)]

pub struct SizeClass(usize);
pub const K: usize = 1024;
pub const SIZE_CLASSES: usize = 6;

pub const SIZE_CLASS_SMALLEST: SizeClass = SizeClass(0);
pub const SIZE_SMALLEST: usize = 1;

pub const SIZE_CLASS_TINY: SizeClass = SizeClass(1);
pub const SIZE_TINY: usize = 32;

pub const SIZE_CLASS_SMALL: SizeClass = SizeClass(2);
pub const SIZE_SMALL: usize = 128;

pub const SIZE_CLASS_MEDIUM: SizeClass = SizeClass(3);
pub const SIZE_MEDIUM: usize = 2 * K;

pub const SIZE_CLASS_LARGE: SizeClass = SizeClass(4);
pub const SIZE_LARGE: usize = 8 * K;

pub const SIZE_CLASS_HUGE: SizeClass = SizeClass(5);
pub const SIZE_HUGE: usize = 32 * K;

pub const SIZES: [usize; SIZE_CLASSES] = [
    SIZE_SMALLEST,
    SIZE_TINY,
    SIZE_SMALL,
    SIZE_MEDIUM,
    SIZE_LARGE,
    SIZE_HUGE,
];

impl SizeClass {
    fn next_up(size: usize) -> SizeClass {
        assert!(size >= SIZE_SMALLEST);

        if size <= SIZE_SMALLEST {
            SIZE_CLASS_SMALLEST
        } else if size <= SIZE_TINY {
            SIZE_CLASS_TINY
        } else if size <= SIZE_SMALL {
            SIZE_CLASS_SMALL
        } else if size <= SIZE_MEDIUM {
            SIZE_CLASS_MEDIUM
        } else if size <= SIZE_LARGE {
            SIZE_CLASS_LARGE
        } else {
            SIZE_CLASS_HUGE
        }
    }

    fn next_down(size: usize) -> SizeClass {
        assert!(size >= SIZE_SMALLEST);

        if size < SIZE_TINY {
            SIZE_CLASS_SMALLEST
        } else if size < SIZE_SMALL {
            SIZE_CLASS_TINY
        } else if size < SIZE_MEDIUM {
            SIZE_CLASS_SMALL
        } else if size < SIZE_LARGE {
            SIZE_CLASS_MEDIUM
        } else if size < SIZE_HUGE {
            SIZE_CLASS_LARGE
        } else {
            SIZE_CLASS_HUGE
        }
    }

    fn idx(self) -> usize {
        self.0
    }
}

pub struct FreeList {
    pub classes: Vec<FreeListClass>,
}
impl FreeList {
    pub fn new() -> FreeList {
        let mut classes = Vec::with_capacity(SIZE_CLASSES);

        for _ in 0..SIZE_CLASSES {
            classes.push(FreeListClass::new());
        }

        FreeList { classes }
    }

    pub fn fragmentation(&self) -> f32 {
        let mut largest = 0;
        let mut total = 0;

        for class in self.classes.iter() {
            for (_, size) in class.sizes.iter() {
                let size = *size;
                total += size;
                if size > largest {
                    largest = size;
                }
            }
        }

        if total == 0 {
            return 0.0;
        }

        1.0 - largest as f32 / total as f32
    }

    pub fn add(&mut self, addr: Address, size: usize) {
        if size < SIZE_SMALLEST {
            //fill_region(vm, addr, addr.offset(size));
            println!("not smallest");
            return;
        }

        debug_assert!(size >= SIZE_SMALLEST);
        let szclass = SizeClass::next_down(size);

        let free_class = &mut self.classes[szclass.idx()];
        free_class.sizes.insert(addr, size);
        free_class.head = FreeSpace(addr);
    }

    pub fn alloc(&mut self, size: usize) -> (FreeSpace, usize) {
        let szclass = SizeClass::next_up(size).idx();
        let last = SIZE_CLASS_HUGE.idx();

        for class in szclass..last {
            let result = self.classes[class].first();

            if result.is_non_null() {
                assert!(self.classes[class].size(result.0) >= size);
                let size = self.classes[class].sizes.remove(&result.0).unwrap();
                return (result, size);
            }
        }

        self.classes[SIZE_CLASS_HUGE.idx()].find(size)
    }
}

use fxhash::FxBuildHasher;

use std::collections::HashMap;

pub struct FreeListClass {
    head: FreeSpace,
    sizes: HashMap<Address, usize, FxBuildHasher>,
}

impl FreeListClass {
    fn new() -> FreeListClass {
        FreeListClass {
            head: FreeSpace::null(),
            sizes: HashMap::with_capacity_and_hasher(100, FxBuildHasher::default()),
        }
    }
    #[allow(dead_code)]
    fn add(&mut self, addr: FreeSpace, size: usize) {
        addr.set_next(self.head);
        self.sizes.insert(addr.0, size);
        self.head = addr;
    }

    fn first(&mut self) -> FreeSpace {
        if self.head.is_non_null() {
            let ret = self.head;
            self.head = ret.next();
            ret
        } else {
            FreeSpace::null()
        }
    }

    fn size(&mut self, addr: Address) -> usize {
        *self.sizes.get(&addr).expect("Address is not in freelist")
    }

    fn find(&mut self, minimum_size: usize) -> (FreeSpace, usize) {
        let mut curr = self.head;
        let mut prev = FreeSpace::null();

        while curr.is_non_null() {
            if self.size(curr.0) >= minimum_size {
                if prev.is_null() {
                    self.head = curr.next();
                } else {
                    prev.set_next(curr.next());
                }

                return (curr, self.size(curr.0));
            }

            prev = curr;
            curr = curr.next();
        }

        (FreeSpace::null(), 0)
    }
}

#[derive(Copy, Clone)]
pub struct FreeSpace(Address);

impl FreeSpace {
    #[inline(always)]
    pub fn null() -> FreeSpace {
        FreeSpace(Address::null())
    }

    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.addr().is_null()
    }

    #[inline(always)]
    pub fn is_non_null(self) -> bool {
        self.addr().is_non_null()
    }

    #[inline(always)]
    pub fn addr(self) -> Address {
        self.0
    }

    #[inline(always)]
    pub fn next(self) -> FreeSpace {
        assert!(self.is_non_null());
        let next = unsafe { *self.addr().add_ptr(1).to_mut_ptr::<Address>() };
        FreeSpace(next)
    }

    #[inline(always)]
    pub fn set_next(&self, next: FreeSpace) {
        assert!(self.is_non_null());
        unsafe { *self.addr().add_ptr(1).to_mut_ptr::<Address>() = next.addr() }
    }
}
