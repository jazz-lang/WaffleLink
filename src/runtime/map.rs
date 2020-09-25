use super::cell_type::CellType;
use crate::prelude::*;
use crate::{gc::object::*, isolate::Isolate, values::Value};
use std::sync::Arc;
pub type ComputeHash = fn(isolate: &Arc<Isolate>, seed: u64, value: Value) -> u64;
pub type IsEqual = fn(isolate: &Arc<Isolate>, left: Value, right: Value) -> bool;
use std::alloc;
pub struct MapNode {
    hash: u64,
    key: Value,
    pub val: Value,
    next: *mut Self,
}
impl MapNode {
    pub const fn key(&self) -> &Value {
        &self.key
    }

    pub const fn hash(&self) -> u64 {
        self.hash
    }
}

#[repr(C)]
pub struct Map {
    cell_type: CellType,
    compute: ComputeHash,
    iseq: IsEqual,
    size: u32,
    count: u32,
    nodes: *mut *mut MapNode,
}

fn calc_layout_for_nodes(n: u32) -> alloc::Layout {
    alloc::Layout::array::<MapNode>(n as _).unwrap()
}

fn alloc_nodes(n: u32) -> *mut *mut MapNode {
    unsafe { alloc::alloc_zeroed(calc_layout_for_nodes(n)) }.cast()
}

pub const HASH_SEED_VALUE: u64 = 5381;

impl Map {
    pub fn new(
        isolate: &Arc<Isolate>,
        compute: ComputeHash,
        iseq: IsEqual,
        mut size: u32,
    ) -> Local<Self> {
        if size < 8 {
            size = 8;
        }
        let mut map = Self {
            cell_type: CellType::Map,
            compute,
            iseq,
            size,
            count: 0,
            nodes: core::ptr::null_mut(),
        };
        map.nodes = alloc_nodes(size);
        isolate.new_local(map)
    }

    unsafe fn resize(&mut self, isolate: &Arc<Isolate>) {
        let size = (self.size as f64 * 1.50) as u32;
        let mut new_tbl = Self {
            cell_type: CellType::Map,
            iseq: self.iseq,
            compute: self.compute,
            count: 0,
            size: size,
            nodes: core::ptr::null_mut(),
        };
        new_tbl.nodes = alloc_nodes(size);

        let mut node;
        let mut next;
        let mut n = 0;
        while n < self.size {
            cfor!((node = *(self.nodes.offset(n as _));!node.is_null(); node = next){
                next = (&*node).next;
                new_tbl.insert(isolate,(&*node).key,(&*node).val);
                self.remove(isolate,(&*node).key);
            });
        }
        *self = new_tbl;
    }
    pub fn remove(&mut self, isolate: &Arc<Isolate>, key: Value) -> bool {
        let hash = (self.compute)(isolate, HASH_SEED_VALUE, key);
        let mut position = hash % self.size as u64;
        unsafe {
            let mut node = self.nodes.offset(position as _).read();
            let mut prev_node: *mut MapNode = core::ptr::null_mut();
            while !node.is_null() {
                if (&*node).hash == hash && (self.iseq)(isolate, (&*node).key, key) {
                    if !prev_node.is_null() {
                        (&mut *prev_node).next = (&*node).next;
                    } else {
                        self.nodes.offset(position as _).write((&*node).next);
                    }
                    let _ = Box::from_raw(node);
                    self.count -= 1;
                    return true;
                }
                prev_node = node;
                node = (&*node).next;
            }
        }
        false
    }
    pub fn insert(&mut self, isolate: &Arc<Isolate>, key: Value, value: Value) -> bool {
        let hash = (self.compute)(isolate, HASH_SEED_VALUE, key);
        let mut position = hash % self.size as u64;
        unsafe {
            let mut node = *(self.nodes.offset(position as _));
            while !node.is_null() {
                if (&*node).hash == hash && (self.iseq)(isolate, key, (&*node).key) {
                    (&mut *node).val = value;
                    return false;
                }
                node = (&*node).next;
            }

            if self.count >= (self.size as f64 * 0.75) as u32 {
                self.resize(isolate);
                position = hash % self.size as u64;
            }
            node = Box::into_raw(Box::new(MapNode {
                hash,
                key,
                val: value,
                next: self.nodes.offset(position as _).read(),
            }));
            self.nodes.offset(position as _).write(node);
            self.count += 1;
            true
        }
    }

    pub fn lookup(&self, isolate: &Arc<Isolate>, key: Value) -> Option<Value> {
        let hash = (self.compute)(isolate, HASH_SEED_VALUE, key);
        let position = hash % self.size as u64;
        unsafe {
            let mut node = self.nodes.offset(position as _).read();
            while !node.is_null() {
                if (&*node).hash == hash && (self.iseq)(isolate, key, (&*node).key) {
                    return Some((&*node).val);
                }
                node = (&*node).next;
            }
        }
        None
    }
    pub fn lookup_mut(&mut self, isolate: &Arc<Isolate>, key: Value) -> Option<&mut Value> {
        let hash = (self.compute)(isolate, HASH_SEED_VALUE, key);
        let position = hash % self.size as u64;
        unsafe {
            let mut node = self.nodes.offset(position as _).read();
            while !node.is_null() {
                if (&*node).hash == hash && (self.iseq)(isolate, key, (&*node).key) {
                    return Some(&mut (&mut *node).val);
                }
                node = (&*node).next;
            }
        }
        None
    }

    pub fn for_each(&self, mut x: impl FnMut(&MapNode)) {
        for i in 0..self.size {
            unsafe {
                let mut node = self.nodes.offset(i as _).read();
                x(&*node);
            }
        }
    }
    pub fn for_each_mut(&self, mut x: impl FnMut(&mut MapNode)) {
        for i in 0..self.size {
            unsafe {
                let mut node = self.nodes.offset(i as _).read();
                x(&mut *node);
            }
        }
    }

    pub fn count(&self) -> usize {
        self.count as _
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        unsafe {
            for n in 0..self.size {
                let mut node = self.nodes.offset(n as _).read();
                self.nodes.offset(n as _).write(0 as *mut _);
                while !node.is_null() {
                    let old_node = node;
                    node = (&*node).next;
                    let _ = Box::from_raw(old_node);
                }
            }
            alloc::dealloc(self.nodes.cast(), calc_layout_for_nodes(self.size));
        }
    }
}

impl GcObject for Map {}
pub fn iseq_default(isolate: &Arc<Isolate>, lhs: Value, rhs: Value) -> bool {
    if lhs.is_number() && rhs.is_number() {
        lhs.to_number() == rhs.to_number()
    } else if lhs.is_undefined_or_null() || rhs.is_undefined_or_null() {
        false
    } else {
        if lhs.is_cell() && rhs.is_cell() {
            if lhs.as_cell().ty() == CellType::String && rhs.as_cell().ty() == CellType::String {
                return lhs.as_cell().cast::<WString>().string
                    == rhs.as_cell().cast::<WString>().string;
            }
        }
        lhs == rhs
    }
}
pub fn compute_hash_default(isolate: &Arc<Isolate>, seed: u64, key: Value) -> u64 {
    let mut hash = seed;
    if key.is_int32() {
        hash = murmur3::murmur3_32_seed(&key.as_int32().to_le_bytes(), hash as _) as _;
    } else if key.is_double() {
        hash = murmur3::murmur3_32_seed(&key.as_double().to_bits().to_le_bytes(), hash as _) as _;
    } else if key.is_undefined_or_null() {
        hash = murmur3::murmur3_32_seed(&[], hash as _) as _;
    } else if key.is_boolean() {
        hash = murmur3::murmur3_32_seed(&[key.is_true() as u8], hash as _) as _;
    } else if key.is_sym() {
        hash = murmur3::murmur3_32_seed(&key.sym_as_u32().to_le_bytes(), hash as _) as _;
    } else {
        if key.is_cell() {
            let cell = key.as_cell();
            if cell.ty() == CellType::String {
                hash = murmur3::murmur3_32_seed(
                    cell.cast::<super::string::WString>().string.as_bytes(),
                    hash as _,
                ) as _;
            } else if cell.ty() == CellType::ComObj {
                if let Some(hash_fn) = cell.cast::<super::cell::FFIObject>().hash {
                    hash = hash_fn(isolate, hash, cell.cast::<super::cell::FFIObject>().data)
                } else {
                    hash = murmur3::murmur3_32_seed(
                        &(cell.cast::<super::cell::FFIObject>().data as usize).to_le_bytes(),
                        hash as _,
                    ) as _;
                }
            } else {
                hash =
                    murmur3::murmur3_32_seed(&(cell.raw() as usize).to_le_bytes(), hash as _) as _;
            }
        } else {
            hash = murmur3::murmur3_32_seed(&[], hash as _) as _;
        }
    }

    hash
}

pub mod murmur3 {
    //! The murmur hash is a relatively fast non-cryptographic hash function for platforms with efficient multiplication.
    //!
    //! This implementation is based on the murmurhash3 variant.
    static C1: u32 = 0xcc9e2d51u32;
    static C2: u32 = 0x1b873593u32;
    static R1: u32 = 15u32;
    static R2: u32 = 13u32;
    static M: u32 = 5u32;
    static N: u32 = 0xe6546b64u32;

    pub fn murmur3_32(data: &[u8]) -> u32 {
        murmur3_32_seed(data, 0)
    }

    pub fn murmur3_32_seed(data: &[u8], seed: u32) -> u32 {
        let mut hash = seed;
        let length = data.len() as u32;

        let n_blocks = length / 4;
        for i in 0..n_blocks {
            let mut k = get_u32(&data[(i * 4) as usize..]);
            k = k.wrapping_mul(C1);
            k = (k << R1) | (k >> (32 - R1));
            k = k.wrapping_mul(C2);

            hash ^= k;
            hash = ((hash << R2) | (hash >> (32 - R2)))
                .wrapping_mul(M)
                .wrapping_add(N);
        }

        let tail = &data[(n_blocks * 4) as usize..];
        let remainder = length & 3;
        let mut k1 = 0u32;

        if remainder == 3 {
            k1 ^= (tail[2] as u32) << 16;
        }

        if remainder >= 2 {
            k1 ^= (tail[1] as u32) << 8
        }

        if remainder >= 1 {
            k1 ^= tail[0] as u32;

            k1 = k1.wrapping_mul(C1);
            k1 = (k1 << R1) | (k1 >> (32 - R1));
            k1 = k1.wrapping_mul(C2);
            hash ^= k1;
        }

        hash ^= length;
        hash ^= hash >> 16;
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= hash >> 13;
        hash = hash.wrapping_mul(0xc2b2ae35);
        hash ^= hash >> 16;

        hash
    }

    fn get_u32(data: &[u8]) -> u32 {
        ((0xff & (data[3] as u32)) << 24)
            | ((0xff & (data[2] as u32)) << 16)
            | ((0xff & (data[1] as u32)) << 8)
            | (0xff & (data[0] as u32))
    }

    #[test]
    fn basic_tests() {
        assert_eq!(0, murmur3_32("".as_bytes()));
        assert_eq!(3530670207, murmur3_32("0".as_bytes()));
        assert_eq!(1642882560, murmur3_32("01".as_bytes()));
        assert_eq!(3966566284, murmur3_32("012".as_bytes()));
        assert_eq!(3558446240, murmur3_32("0123".as_bytes()));
        assert_eq!(433070448, murmur3_32("01234".as_bytes()));
        assert_eq!(1364076727, murmur3_32_seed("".as_bytes(), 1));
        assert_eq!(
            2832214938,
            murmur3_32("I will not buy this record, it is scratched.".as_bytes())
        );
    }
}
