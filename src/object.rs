use super::*;
use crate::gc::*;
use std::mem::size_of;
use std::sync::atomic::AtomicU32;
use std::thread::JoinHandle;
use value::*;
#[cfg(feature = "use-vtable")]
use vtable::VTable;
const MARK_BITS: usize = 2;
const MARK_MASK: usize = (2 << MARK_BITS) - 1;
const FWD_MASK: usize = !0 & !MARK_MASK;
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum WaffleType {
    None,
    FreeObject,
    Object,
    Array,
    String,
    BigInt,
    Regex,
    Function,
    Thread,
    File,
    Map,
    MapNode,
    Module,
    /// Type for "external" objects.
    Abstract,
}
use crate::tagged::TaggedPointer;
/// Waffle type header. This type stores all information used by GC, runtime and API.
#[derive(Clone)]
#[repr(C)]
pub struct WaffleTypeHeader {
    pub(crate) ty: WaffleType,
    pub(crate) rc: u32,
    pub(crate) fwdptr: TaggedPointer<WaffleCell>,
}

impl WaffleTypeHeader {
    pub fn increment(&mut self) -> bool {
        self.rc += 1;
        if self.is_new() {
            self.unset_new();
            true
        } else {
            false
        }
    }
    pub fn decrement(&mut self) -> bool {
        if self.rc == 0 {
            return false;
        }
        self.rc -= 1;
        self.rc == 0
    }
    pub fn set_logged(&mut self) -> bool {
        let x = self.fwdptr.bit_is_set(5);
        self.fwdptr.set_bit(5);
        x
    }

    pub fn unset_logged(&mut self) -> bool {
        let x = self.fwdptr.bit_is_set(5);
        self.fwdptr.unset_bit(5);
        x
    }
    pub fn mark(&mut self, x: bool) -> bool {
        let y = self.fwdptr.bit_is_set(0);
        if x {
            self.fwdptr.set_bit(0);
        } else {
            self.fwdptr.unset_bit(0);
        }
        y
    }
    pub fn unmark(&mut self) {
        self.fwdptr.unset_bit(0);
    }

    pub fn is_marked(&self, b: bool) -> bool {
        if b {
            self.fwdptr.bit_is_set(0)
        } else {
            !self.fwdptr.bit_is_set(0)
        }
    }

    pub fn is_pinned(&self) -> bool {
        self.fwdptr.bit_is_set(1)
    }

    pub fn set_pinned(&mut self) {
        self.fwdptr.set_bit(1);
    }

    pub fn unpin(&mut self) {
        self.fwdptr.unset_bit(1);
    }

    pub fn is_forwarded(&self) -> Option<WaffleCellPointer> {
        if self.fwdptr.bit_is_set(2) {
            Some(WaffleCellPointer::from_ptr(self.fwdptr.untagged()))
        } else {
            None
        }
    }
    pub fn set_forwarded(&mut self, ptr: WaffleCellPointer) {
        let tagged = TaggedPointer::new(ptr.raw());
        self.fwdptr = tagged;
        self.fwdptr.set_bit(2);
    }
    pub fn set_new(&mut self) {
        self.fwdptr.set_bit(4);
    }
    pub fn unset_new(&mut self) {
        self.fwdptr.unset_bit(4);
    }
    pub fn is_new(&self) -> bool {
        self.fwdptr.bit_is_set(4)
    }
    pub fn fwdptr(&self) -> usize {
        self.fwdptr.untagged() as usize
    }
}
#[repr(C)]
pub struct WaffleCell {
    pub header: WaffleTypeHeader,
}
#[derive(Ord, PartialOrd)]
pub struct WaffleCellPointer<T: WaffleCellTrait = WaffleCell> {
    pub(crate) value: std::ptr::NonNull<T>,
}

impl<T: WaffleCellTrait> Copy for WaffleCellPointer<T> {}

impl<T: WaffleCellTrait> Clone for WaffleCellPointer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: WaffleCellTrait> WaffleCellPointer<T> {
    pub fn from_ptr(ptr: *const T) -> Self {
        Self {
            value: std::ptr::NonNull::new(ptr as *mut T).unwrap(),
        }
    }
    pub fn try_as<U: WaffleCellTrait>(&self) -> Option<WaffleCellPointer<U>> {
        if self.type_of() == T::TYPE {
            Some(WaffleCellPointer {
                value: self.value.cast(),
            })
        } else {
            None
        }
    }
    pub fn cast<U: WaffleCellTrait>(&self) -> WaffleCellPointer<U> {
        WaffleCellPointer {
            value: self.value.cast(),
        }
    }
    pub fn finalize(&self) {}
    pub fn size(&self) -> usize {
        match self.type_of() {
            WaffleType::Object => size_of::<WaffleObject>(),
            WaffleType::String => size_of::<WaffleString>(),
            WaffleType::Array => {
                (size_of::<WaffleArray>() - crate::WORD)
                    + crate::WORD * self.try_as_array().unwrap().value().len()
            }
            WaffleType::Map => size_of::<WaffleMap>(),
            WaffleType::FreeObject => self.value().header().fwdptr(),
            WaffleType::MapNode => size_of::<MapNode>(),
            _ => todo!(),
        }
    }
    pub fn visit(&self, trace: &mut dyn FnMut(*const WaffleCellPointer)) {
        unsafe {
            match self.type_of() {
                WaffleType::Object => {
                    let obj = self.try_as_object().unwrap();
                    trace(std::mem::transmute(&obj.value().map));
                    /*for (k, v) in obj.value().map.iter() {
                        if k.is_cell() {
                            trace(k.as_cell_ref());
                        }
                        if v.is_cell() {
                            trace(v.as_cell_ref());
                        }
                    }*/
                }
                WaffleType::Array => {
                    let arr = self.try_as_array().unwrap();
                    for i in 0..arr.value().len() {
                        let item = arr.value().at_ref(i);
                        if item.is_cell() {
                            trace(item.as_cell_ref());
                        }
                    }
                }
                WaffleType::Map => {
                    let map = self.as_map();
                    map.value().visit(trace);
                }
                _ => (),
            }
        }
    }
    pub fn to_cell(&self) -> WaffleCellPointer {
        WaffleCellPointer {
            value: self.value.cast(),
        }
    }
    pub fn try_as_map(&self) -> Option<WaffleCellPointer<HMap>> {
        if self.type_of() == WaffleType::Map {
            Some(WaffleCellPointer {
                value: self.value.cast(),
            })
        } else {
            None
        }
    }
    pub fn as_map(&self) -> WaffleCellPointer<HMap> {
        self.try_as_map().unwrap()
    }
    /// Try to cast this cell to `WaffleObject` type, otherwise return None.
    pub fn try_as_object(&self) -> Option<WaffleCellPointer<WaffleObject>> {
        if self.type_of() == WaffleType::Object {
            Some(WaffleCellPointer {
                value: self.value.cast(),
            })
        } else {
            None
        }
    }
    /// Try to cast this cell to `WaffleString` type, otherwise return None.
    pub fn try_as_string(&self) -> Option<WaffleCellPointer<WaffleString>> {
        if self.type_of() == WaffleType::String {
            Some(WaffleCellPointer {
                value: self.value.cast(),
            })
        } else {
            None
        }
    }

    pub fn try_as_array(&self) -> Option<WaffleCellPointer<WaffleArray>> {
        if self.type_of() == WaffleType::Array {
            Some(WaffleCellPointer {
                value: self.value.cast(),
            })
        } else {
            None
        }
    }
    /// Unchecked cast to `WaffleObject`
    pub unsafe fn as_object(&self) -> WaffleCellPointer<WaffleObject> {
        WaffleCellPointer {
            value: self.value.cast(),
        }
    }
    /// Unchecked cast to `WaffleString`
    pub unsafe fn as_string(&self) -> WaffleCellPointer<WaffleString> {
        WaffleCellPointer {
            value: self.value.cast(),
        }
    }
    /// Unchecked cast to `WaffleArray`
    pub unsafe fn as_array(&self) -> WaffleCellPointer<WaffleArray> {
        WaffleCellPointer {
            value: self.value.cast(),
        }
    }
    pub fn value(&self) -> &T {
        unsafe { &*self.value.as_ptr() }
    }
    pub fn value_mut(&self) -> &mut T {
        unsafe { &mut *self.value.as_ptr() }
    }
    pub fn type_of(&self) -> WaffleType {
        match self.value().ty() {
            Some(t) if t != WaffleType::None => t,
            _ => T::TYPE,
        }
    }
    pub fn raw(&self) -> *mut T {
        self.value.as_ptr()
    }
}
use std::hash;

impl<T: WaffleCellTrait> hash::Hash for WaffleCellPointer<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.raw().hash(state);
    }
}

impl<T: WaffleCellTrait> PartialEq for WaffleCellPointer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw() == other.raw()
    }
}
impl<T: WaffleCellTrait> Eq for WaffleCellPointer<T> {}

pub trait WaffleCellTrait {
    const TYPE: WaffleType;
    fn ty(&self) -> Option<WaffleType> {
        None
    }

    fn header(&self) -> &WaffleTypeHeader;
    fn header_mut(&mut self) -> &mut WaffleTypeHeader;
}

use std::collections::HashMap;

#[repr(C)]
pub struct WaffleObject {
    pub header: WaffleTypeHeader,
    pub prototype: Value,
    pub mask: u32,
    pub map: WaffleCellPointer<HMap>,
}

impl WaffleObject {
    pub fn lookup(&self, key: Value) -> Result<Option<Value>, Value> {
        unimplemented!()
    }
}

impl WaffleCellTrait for WaffleObject {
    const TYPE: WaffleType = WaffleType::Object;
    fn ty(&self) -> Option<WaffleType> {
        Some(Self::TYPE)
    }
    fn header(&self) -> &WaffleTypeHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut WaffleTypeHeader {
        &mut self.header
    }
}

impl WaffleCellTrait for WaffleCell {
    const TYPE: WaffleType = WaffleType::None;
    fn ty(&self) -> Option<WaffleType> {
        Some(self.header.ty)
    }
    fn header(&self) -> &WaffleTypeHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut WaffleTypeHeader {
        &mut self.header
    }
}

#[repr(C)]
pub struct WaffleString {
    pub header: WaffleTypeHeader,
    pub string: String,
    pub hash: u64,
}

impl WaffleCellTrait for WaffleString {
    const TYPE: WaffleType = WaffleType::String;
    fn ty(&self) -> Option<WaffleType> {
        Some(Self::TYPE)
    }
    fn header(&self) -> &WaffleTypeHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut WaffleTypeHeader {
        &mut self.header
    }
}

#[repr(C)]
pub struct WaffleArray {
    pub header: WaffleTypeHeader,
    pub(crate) len: usize,
    value: Value,
}
impl WaffleArray {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn at(&self, ix: usize) -> Value {
        unsafe { *(&self.value as *const Value).offset(ix as _) }
    }

    pub fn at_ref(&self, ix: usize) -> &Value {
        unsafe { &*(&self.value as *const Value).offset(ix as _) }
    }
    pub fn at_ref_mut(&self, ix: usize) -> &mut Value {
        unsafe { &mut *(&self.value as *const Value as *mut Value).offset(ix as _) }
    }
}

use std::ops::*;

impl Index<usize> for WaffleArray {
    type Output = Value;
    fn index(&self, index: usize) -> &Self::Output {
        self.at_ref(index)
    }
}

impl IndexMut<usize> for WaffleArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.at_ref_mut(index)
    }
}

impl WaffleCellTrait for WaffleArray {
    const TYPE: WaffleType = WaffleType::Array;
    fn ty(&self) -> Option<WaffleType> {
        Some(Self::TYPE)
    }

    fn header(&self) -> &WaffleTypeHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut WaffleTypeHeader {
        &mut self.header
    }
}

pub struct WaffleMap {
    pub header: WaffleTypeHeader,
    pub(crate) size: usize,
}

impl WaffleCellTrait for WaffleMap {
    const TYPE: WaffleType = WaffleType::Map;
    fn ty(&self) -> Option<WaffleType> {
        Some(Self::TYPE)
    }

    fn header(&self) -> &WaffleTypeHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut WaffleTypeHeader {
        &mut self.header
    }
}

impl WaffleObject {
    pub fn new_empty(size: usize) -> WaffleCellPointer<Self> {
        unsafe {
            let obj = super::VM
                .state
                .heap
                .allocate(WaffleType::Object, std::mem::size_of::<Self>())
                .unwrap()
                .as_object();
            obj.value_mut().init(size);
            obj
        }
    }
    pub fn init(&mut self, size: usize) {
        self.map = HMap::new_empty(size);
        self.mask = (size as u32 - 1) * 8 as u32;
    }
}

#[repr(C)]
pub struct HMap {
    pub header: WaffleTypeHeader,
    pub nnodes: u32,
    pub size: u32,
    pub nodes: Option<WaffleCellPointer<MapNode>>,
    pub lastfree: Option<WaffleCellPointer<MapNode>>,
}

impl HMap {
    pub fn new_empty(cap: usize) -> WaffleCellPointer<Self> {
        let map: WaffleCellPointer<Self> = super::VM
            .state
            .heap
            .allocate(WaffleType::Map, std::mem::size_of::<Self>())
            .unwrap()
            .cast();
        map.value_mut().alloc_nodes(cap);
        map.value_mut().nnodes = cap as _;
        map
    }
    pub fn find(&self, key: Value) -> Option<WaffleCellPointer<MapNode>> {
        let hash = crate::runtime::hash::waffle_get_hash_of(key);
        let mut current = &self.nodes;
        while let Some(c) = current {
            if c.value().hash == hash {
                return Some(*c);
            }
            current = &c.value().next;
        }
        return None;
    }

    pub fn getp(&self, key: Value) -> Option<&mut Value> {
        let hash = crate::runtime::hash::waffle_get_hash_of(key);
        let mut current = &self.nodes;
        while let Some(c) = current {
            if c.value().hash == hash {
                return Some(&mut c.value_mut().val);
            }
            current = &c.value().next;
        }
        return None;
    }

    fn take_free(&mut self) -> Option<WaffleCellPointer<MapNode>> {
        match self.lastfree.take() {
            Some(node) => {
                self.lastfree = node.value().next;
                Some(node)
            }
            None => None,
        }
    }

    fn alloc_nodes(&mut self, count: usize) {
        for _ in 0..count {
            let node = MapNode::new_empty();
            node.value_mut().next = self.lastfree;
            self.lastfree = Some(node);
        }
    }
    /// Insert a *new* key into a hash table, growing table if necessary.
    /// * Do not use this function if key is already in the table
    pub fn add(&mut self, key: Value, val: Value) {
        if let Some(node) = self.take_free() {
            node.value_mut().key = key;
            node.value_mut().val = val;
            node.value_mut().hash = crate::runtime::hash::waffle_get_hash_of(key);
            node.value_mut().next = self.nodes;
            self.nodes = Some(node);
            self.size += 1;
            return;
        }
        self.resize((self.nnodes as f64 / 0.7).floor() as usize);
        self.add(key, val)
    }
    pub fn set(&mut self, key: Value, val: Value) -> bool {
        if let Some(node) = self.find(key) {
            node.value_mut().val = val;
            return false;
        }

        self.add(key, val);
        true
    }
    fn resize(&mut self, mut new_size: usize) {
        if new_size == 0 {
            new_size = 4;
        }
        self.nnodes = new_size as _;
        self.alloc_nodes(new_size);
    }
    pub fn delete(&mut self, key: Value) -> Option<Value> {
        if let Some(node) = self.find(key) {
            let val = node.value_mut();
            let v = val.val;
            val.val = Value::default();
            val.key = Value::default();
            val.hash = 0;
            val.next = self.lastfree;
            self.lastfree = Some(node);
            return Some(v);
        }
        None
    }
    pub fn visit(&self, trace: &mut dyn FnMut(*const WaffleCellPointer)) {
        let mut c = self.nodes;
        unsafe {
            while let Some(x) = c {
                trace(std::mem::transmute(&x));
                c = x.value().next;
            }
            let mut c = self.lastfree;
            while let Some(x) = c {
                trace(std::mem::transmute(&x));
                c = x.value().next;
            }
        }
    }
}

#[repr(C)]
pub struct MapNode {
    pub header: WaffleTypeHeader,
    pub val: Value,
    pub key: Value,
    pub hash: u32,
    pub next: Option<WaffleCellPointer<Self>>,
}
impl MapNode {
    pub fn new_empty() -> WaffleCellPointer<Self> {
        let node = crate::VM
            .state
            .heap
            .allocate(WaffleType::MapNode, std::mem::size_of::<Self>())
            .unwrap()
            .cast::<MapNode>();
        node.value_mut().next = None;
        node.value_mut().key = Value::default();
        node.value_mut().hash = 0;

        node
    }

    pub fn visit(&self, trace: &mut dyn FnMut(*const WaffleCellPointer)) {
        if self.val.is_cell() {
            trace(self.val.as_cell_ref());
        }

        if self.key.is_cell() {
            trace(self.val.as_cell_ref());
        }
    }
}
impl WaffleCellTrait for MapNode {
    const TYPE: WaffleType = WaffleType::MapNode;
    fn ty(&self) -> Option<WaffleType> {
        Some(Self::TYPE)
    }
    fn header(&self) -> &WaffleTypeHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut WaffleTypeHeader {
        &mut self.header
    }
}

impl WaffleCellTrait for HMap {
    const TYPE: WaffleType = WaffleType::Map;
    fn ty(&self) -> Option<WaffleType> {
        Some(Self::TYPE)
    }

    fn header(&self) -> &WaffleTypeHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut WaffleTypeHeader {
        &mut self.header
    }
}
