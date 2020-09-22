use super::cell_type::CellType;
use crate::{gc::object::*, isolate::Isolate, values::Value};
pub type ComputeHash = fn(isolate: &mut Isolate, seed: u64, value: Value) -> u64;
pub type IsEqual = fn(isolate: &mut Isolate, left: Value, right: Value) -> bool;
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
        isolate: &mut Isolate,
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

    unsafe fn resize(&mut self, isolate: &mut Isolate) {
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
    pub fn remove(&mut self, isolate: &mut Isolate, key: Value) -> bool {
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
    pub fn insert(&mut self, isolate: &mut Isolate, key: Value, value: Value) -> bool {
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

    pub fn lookup(&self, isolate: &mut Isolate, key: Value) -> Option<Value> {
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
    pub fn lookup_mut(&mut self, isolate: &mut Isolate, key: Value) -> Option<&mut Value> {
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
