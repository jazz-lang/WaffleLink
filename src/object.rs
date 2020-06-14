use super::*;
use std::thread::JoinHandle;
use value::*;
#[cfg(feature = "use-vtable")]
use vtable::VTable;
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum WaffleType {
    None,
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

/// Waffle type header. This type stores all information used by GC, runtime and API.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WaffleTypeHeader {
    #[cfg(feature = "use-vtable")]
    #[cfg_attr(feature = "use-vtable", doc = "virtual method table")]
    pub vtable: &'static VTable,
    /// Mark byte for GC
    pub(crate) mark: u8,
    pub(crate) ty: WaffleType,
}
#[repr(C)]
pub struct WaffleCell {
    pub header: WaffleTypeHeader,
}
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct WaffleCellPointer<T: WaffleCellTrait = WaffleCell> {
    value: std::ptr::NonNull<T>,
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
}

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
    len: usize,
    cap: usize,
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
