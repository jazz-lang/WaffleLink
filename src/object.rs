use super::*;
use crate::gc::*;
use std::mem::size_of;
use value::*;
#[cfg(feature = "use-vtable")]
use vtable::VTable;
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
/*/// Waffle type header. This type stores all information used by GC, runtime and API.
#[derive(Clone)]
#[repr(C)]
pub struct WaffleTypeHeader {
    pub(crate) ty: WaffleType,
    pub(crate) rc: u32,
    pub(crate) fwdptr: TaggedPointer<WaffleCell>,
}*/

pub mod default_header {
    use super::*;
    bitfield::bitfield! {
        pub struct WaffleSmallHeader(u32);
        #[doc="If the object at this address was forwarded somewhere else."]
        pub forwarded,set_forwarded_: 0;
        #[doc="If this object was already visited by the tracing collector.
        
            _Note_: true/false do not mean marked/unmarked. The tracing collector
            will flip the meaning of the value for every collection cycle. See
            `Heap.current_live_mark`."
        ]
        pub marked,set_marked_: 1;
        #[doc="If this object was pushed on the `modBuffer` in `RCCollector`."]
        pub logged,set_logged_: 2;
        #[doc="If this object must not be evacuated (moved) by the collector."]
        pub pinned,set_pinned_: 3;
        #[doc="If this object was never touched by the collectors."]
        pub new,set_new_: 4;
        #[doc="Object type"]
        pub ty_,set_ty: 13,5;
        #[doc="18 bit reference counter, on overflow we use hashmap."]
        pub rc,set_rc: 31,13;
        #[doc="31 bit forwarding difference between pointer and chunk of memory."]
        pub forwarding,set_forwarding_: 31,1;
    }
    impl WaffleSmallHeader {
        pub fn increment(&mut self) -> bool {
            let rc = self.rc();
            self.set_rc(rc + 1);
            if self.is_new() {
                self.unsed_new();
                true
            } else {
                false
            }
        }
        pub fn decrement(&mut self) -> bool {
            let rc = self.rc();
            if rc == 0 {
                return false;
            }
            self.set_rc(rc - 1);
            self.rc() == 0
        }

        pub fn set_logged(&mut self) -> bool {
            let x = self.logged();
            self.set_logged_(true);
            x
        }

        pub fn unset_logged(&mut self) -> bool {
            let x = self.logged();
            self.set_logged_(false);
            x
        }

        pub fn mark(&mut self, x: bool) -> bool {
            let y = self.marked();
            if x {
                self.set_marked_(true);
            } else {
                self.set_marked_(false);
            }
            y
        }

        pub fn unmark(&mut self) {
            self.set_marked_(false);
        }

        pub fn is_marked(&self, b: bool) -> bool {
            if b {
                self.marked()
            } else {
                !self.marked()
            }
        }

        pub fn is_pinned(&self) -> bool {
            self.pinned()
        }

        pub fn set_pinned(&mut self) {
            self.set_pinned_(true);
        }

        pub fn unpin(&mut self) {
            self.set_pinned_(false);
        }

        pub fn is_forwarded(&self) -> Option<u32> {
            if self.forwarded() {
                Some(self.forwarding())
            } else {
                None
            }
        }

        pub fn set_forwarded(&mut self, ptr: i32) {
            self.set_forwarding_(ptr as u32);
            self.set_forwarded_(true);
        }

        pub fn set_new(&mut self) {
            self.set_new_(true);
        }

        pub fn is_new(&self) -> bool {
            self.new()
        }

        pub fn unsed_new(&mut self) {
            self.set_new_(false);
        }

        pub fn ty(&self) -> WaffleType {
            let t = self.ty_();
            unsafe { std::mem::transmute(t as u8) }
        }

        pub fn set_type(&mut self, ty: WaffleType) {
            let x = ty as u8;
            self.set_ty(x as _);
        }
    }
    bitfield::bitfield! {
    pub struct WaffleHeader(u64);
        pub forwarded,set_forwarded_: 0;
        pub marked,set_marked_: 1;
        pub logged,set_logged_: 2;
        pub pinned,set_pinned_: 3;
        pub new,set_new_: 4;
        pub ty_,set_ty: 13,5;
        pub forwarding,set_forwarding_: 63,1;
        pub rc,set_rc: 63,31;
    }

    impl WaffleHeader {
        pub fn increment(&mut self) -> bool {
            let rc = self.rc();
            self.set_rc(rc + 1);
            if self.is_new() {
                self.unsed_new();
                true
            } else {
                false
            }
        }
        pub fn decrement(&mut self) -> bool {
            let rc = self.rc();
            if rc == 0 {
                return false;
            }
            self.set_rc(rc - 1);
            self.rc() == 0
        }

        pub fn set_logged(&mut self) -> bool {
            let x = self.logged();
            self.set_logged_(true);
            x
        }

        pub fn unset_logged(&mut self) -> bool {
            let x = self.logged();
            self.set_logged_(false);
            x
        }

        pub fn mark(&mut self, x: bool) -> bool {
            let y = self.marked();
            if x {
                self.set_marked_(true);
            } else {
                self.set_marked_(false);
            }
            y
        }

        pub fn unmark(&mut self) {
            self.set_marked_(false);
        }

        pub fn is_marked(&self, b: bool) -> bool {
            if b {
                self.marked()
            } else {
                !self.marked()
            }
        }

        pub fn is_pinned(&self) -> bool {
            self.pinned()
        }

        pub fn set_pinned(&mut self) {
            self.set_pinned_(true);
        }

        pub fn unpin(&mut self) {
            self.set_pinned_(false);
        }

        pub fn is_forwarded(&self) -> Option<WaffleCellPointer> {
            if self.forwarded() {
                Some(WaffleCellPointer::from_ptr(
                    self.forwarding() as *mut WaffleCell
                ))
            } else {
                None
            }
        }

        pub fn set_forwarded(&mut self, ptr: WaffleCellPointer) {
            self.set_forwarding_(ptr.raw() as u64);
            self.set_forwarded_(true);
        }

        pub fn set_new(&mut self) {
            self.set_new_(true);
        }

        pub fn is_new(&self) -> bool {
            self.new()
        }

        pub fn unsed_new(&mut self) {
            self.set_new_(false);
        }

        pub fn ty(&self) -> WaffleType {
            let t = self.ty_();
            unsafe { std::mem::transmute(t as u8) }
        }

        pub fn set_type(&mut self, ty: WaffleType) {
            let x = ty as u8;
            self.set_ty(x as _);
        }
    }
}

pub mod fat_header {
    use super::*;
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
            let x = self.fwdptr.bit_is_set(4);
            self.fwdptr.set_bit(5);
            x
        }

        pub fn ty(&self) -> WaffleType {
            self.ty
        }

        pub fn set_type(&mut self, ty: WaffleType) {
            self.ty = ty;
        }
        pub fn unset_logged(&mut self) -> bool {
            let x = self.fwdptr.bit_is_set(4);
            self.fwdptr.unset_bit(5);
            x
        }
        pub fn mark(&mut self, x: bool) -> bool {
            let y = self.fwdptr.bit_is_set(2);
            if x {
                self.fwdptr.set_bit(2);
            } else {
                self.fwdptr.unset_bit(2);
            }
            y
        }
        pub fn unmark(&mut self) {
            self.fwdptr.unset_bit(2);
        }
        pub fn is_marked(&self, b: bool) -> bool {
            if b {
                self.fwdptr.bit_is_set(2)
            } else {
                !self.fwdptr.bit_is_set(2)
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
            if self.fwdptr.bit_is_set(0) {
                Some(WaffleCellPointer::from_ptr(self.fwdptr.untagged()))
            } else {
                None
            }
        }
        pub fn set_forwarded(&mut self, ptr: WaffleCellPointer) {
            let tagged = TaggedPointer::new(ptr.raw());
            self.fwdptr = tagged;
            self.fwdptr.set_bit(0);
        }
        pub fn set_new(&mut self) {
            self.fwdptr.set_bit(3);
        }
        pub fn unset_new(&mut self) {
            self.fwdptr.unset_bit(3);
        }
        pub fn is_new(&self) -> bool {
            self.fwdptr.bit_is_set(3)
        }
        pub fn forwarding(&self) -> usize {
            self.fwdptr.untagged() as usize
        }
    }
}
//pub use fat_header::WaffleTypeHeader;
pub use default_header::WaffleSmallHeader as WaffleTypeHeader;
#[repr(C)]
pub struct WaffleCell {
    pub header: WaffleTypeHeader,
}
#[derive(Ord, PartialOrd)]
pub struct WaffleCellPointer<T: WaffleCellTrait = WaffleCell> {
    pub(crate) value: *mut T,
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
            value: ptr as *mut _,
        }
    }

    pub fn is_null(&self) -> bool {
        self.value as usize == 0
    }

    pub fn null() -> Self {
        Self { value: 0 as *mut T }
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
                (size_of::<WaffleArray>() - 8) + 8 * self.try_as_array().unwrap().value().len()
            }
            WaffleType::Map => size_of::<HMap>(),
            WaffleType::FreeObject => self.value().header().forwarding() as usize,
            WaffleType::MapNode => size_of::<MapNode>(),
            WaffleType::Module => size_of::<crate::module::Module>(),
            _ => todo!("{:?}", self.type_of()),
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
        unsafe { &*self.value }
    }
    pub fn value_mut(&self) -> &mut T {
        unsafe { &mut *self.value }
    }
    pub fn type_of(&self) -> WaffleType {
        match self.value().ty() {
            Some(t) if t != WaffleType::None => t,
            _ => T::TYPE,
        }
    }
    pub fn raw(&self) -> *mut T {
        self.value
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

#[repr(C)]
pub struct WaffleObject {
    pub header: WaffleTypeHeader,
    pub prototype: Value,
    pub mask: u32,
    pub map: WaffleCellPointer<HMap>,
}

impl WaffleObject {
    pub fn lookup(&self, key: Value) -> Option<&mut Value> {
        self.map.value().getp(key)
    }

    pub fn set(&self, key: Value, val: Value) {
        self.map.value_mut().set(key, val);
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
        Some(self.header.ty())
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
    pub fn new(init: Value, count: usize) -> RootedCell<Self> {
        if count >= 1024 * 1024 * 1024 {
            panic!("Array is too large");
        }
        let mem: RootedCell<Self> = crate::VM
            .state
            .heap
            .allocate(
                WaffleType::Array,
                (size_of::<WaffleArray>() - 8) + 8 * count,
            )
            .unwrap();
        for ix in 0..count {
            *mem.value_mut().at_ref_mut(ix) = init;
        }
        mem.value_mut().len = count;

        mem
    }
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

impl WaffleObject {
    pub fn new_empty(size: usize) -> RootedCell<Self> {
        let obj: RootedCell<Self> = super::VM
            .state
            .heap
            .allocate(WaffleType::Object, std::mem::size_of::<Self>())
            .unwrap();
        obj.cell_ref().value_mut().init(size);
        obj
    }
    pub fn init(&mut self, size: usize) {
        self.map = HMap::new_empty(size).to_heap();
        self.mask = (size as u32 - 1) * 8 as u32;
    }
}

#[repr(C)]
pub struct HMap {
    pub header: WaffleTypeHeader,
    pub nnodes: u32,
    pub size: u32,
    pub nodes: WaffleCellPointer<MapNode>,
    pub lastfree: WaffleCellPointer<MapNode>,
}

impl HMap {
    pub fn new_empty(cap: usize) -> RootedCell<Self> {
        let map: RootedCell<Self> = super::VM
            .state
            .heap
            .allocate(WaffleType::Map, std::mem::size_of::<Self>())
            .unwrap();
        map.value_mut().nodes = WaffleCellPointer::null();
        map.value_mut().lastfree = WaffleCellPointer::null();
        map.value_mut().alloc_nodes(cap);
        map.value_mut().nnodes = cap as _;
        map
    }
    pub fn find(&self, key: Value) -> Option<WaffleCellPointer<MapNode>> {
        let hash = crate::runtime::hash::waffle_get_hash_of(key);
        let mut current = &self.nodes;
        while current.is_null() == false {
            //log::trace!("{:p}", current.raw());
            if current.value().hash == hash {
                return Some(*current);
            }
            current = &current.value().next;
        }
        return None;
    }

    pub fn getp(&self, key: Value) -> Option<&mut Value> {
        let hash = crate::runtime::hash::waffle_get_hash_of(key);
        let mut current = &self.nodes;
        while current.is_null() == false {
            if current.value().hash == hash {
                return Some(&mut current.value_mut().val);
            }
            current = &current.value().next;
        }
        return None;
    }

    fn take_free(&mut self) -> WaffleCellPointer<MapNode> {
        /*match self.lastfree {
            Some(node) => {
                self.lastfree = node.value().next;
                Some(node)
            }
            None => None,
        }*/
        if self.lastfree.is_null() {
            return WaffleCellPointer::null();
        } else {
            let val = self.lastfree;
            assert!(val.raw() as u64 != 0x7fff0000000000b8);
            assert!(self.lastfree.value().next.raw() as u64 != 0x7fff0000000000b8);
            self.lastfree = self.lastfree.value().next;
            val
        }
    }

    fn alloc_nodes(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        for _ in 0..count {
            let node = MapNode::new_empty().to_heap();
            node.value_mut().next = self.lastfree;
            assert!(self.lastfree.raw() as u64 != 0x7fff0000000000b8);
            assert!(node.raw() as u64 != 0x7fff0000000000b8);
            self.lastfree = node;
        }
    }
    /// Insert a *new* key into a hash table, growing table if necessary.
    /// * Do not use this function if key is already in the table
    pub fn add(&mut self, key: Value, val: Value) {
        let node = self.take_free();
        if !node.is_null() {
            node.value_mut().key = key;
            node.value_mut().val = val;
            node.value_mut().hash = crate::runtime::hash::waffle_get_hash_of(key);
            node.value_mut().next = self.nodes;
            self.nodes = node;
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
            assert!(self.lastfree.raw() as u64 != 0x7fff0000000000b8);
            assert!(node.raw() as u64 != 0x7fff0000000000b8);
            self.lastfree = node;
            return Some(v);
        }
        None
    }
    pub fn visit(&self, trace: &mut dyn FnMut(*const WaffleCellPointer)) {
        let mut c = self.nodes;
        let mut c2 = self.lastfree;
        assert!(c2.raw() as u64 != 0x7fff0000000000b8);
        unsafe {
            while c.is_null() == false {
                trace(std::mem::transmute(&c));
                c = c.value().next;
            }
            while c2.is_null() == false {
                if c2.raw() as u64 == 0x7fff0000000000b8 {
                    return;
                }
                assert!(
                    &c2 as *const WaffleCellPointer<_> as *const u8
                        == std::mem::transmute::<_, *const u8>(&c2)
                );
                trace(std::mem::transmute(&c2));
                //log::trace!("{}", c2.raw() as u64 == 0x7fff0000000000b8);
                c2 = c2.value().next;
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
    pub next: WaffleCellPointer<Self>,
}
impl MapNode {
    pub fn new_empty() -> RootedCell<Self> {
        let node: RootedCell<MapNode> = crate::VM
            .state
            .heap
            .allocate(WaffleType::MapNode, std::mem::size_of::<Self>())
            .unwrap();
        node.value_mut().next = WaffleCellPointer::null();
        node.value_mut().key = Value::default();
        node.value_mut().val = Value::default();
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

use std::ops::{Deref, DerefMut};

impl<T: WaffleCellTrait> Deref for WaffleCellPointer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl<T: WaffleCellTrait> DerefMut for WaffleCellPointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value_mut()
    }
}
