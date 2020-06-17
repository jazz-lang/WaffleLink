use super::*;
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
    pub fn finalize(&self) {}
    pub fn size(&self) -> usize {
        use std::mem::size_of;
        match self.type_of() {
            WaffleType::Object => size_of::<WaffleObject>(),
            WaffleType::String => size_of::<WaffleString>(),
            WaffleType::Array => {
                (size_of::<WaffleArray>() - 8) * self.try_as_array().unwrap().value().len()
            }
            WaffleType::FreeObject => self.value().header().fwdptr(),
            _ => todo!(),
        }
    }
    pub fn visit(&self, trace: &mut dyn FnMut(*const WaffleCellPointer)) {
        match self.type_of() {
            WaffleType::Object => {
                let obj = self.try_as_object().unwrap();
                for (k, v) in obj.value().map.iter() {
                    if k.is_cell() {
                        trace(k.as_cell_ref());
                    }
                    if v.is_cell() {
                        trace(v.as_cell_ref());
                    }
                }
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
            _ => (),
        }
    }
    pub fn to_cell(&self) -> WaffleCellPointer {
        WaffleCellPointer {
            value: self.value.cast(),
        }
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
    pub map: HashMap<Value, Value>,
}

impl WaffleObject {
    pub fn lookup(&self, key: Value) -> Result<Option<Value>, Value> {
        match self.map.get(&key) {
            Some(val) => Ok(Some(*val)),
            None => self.prototype.lookup(key),
        }
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
    pub(crate) cap: usize,
    value: Value,
}
impl WaffleArray {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.cap
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
