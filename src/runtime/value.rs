use super::cell::*;
use super::function::Type;
use super::process::*;
use super::symbol::*;
use crate::common::ptr::*;
pub type EncodedValue = i64;

#[derive(Copy, Clone)]
#[repr(C)]
pub union EncodedValueDescriptor {
    pub as_int64: i64,
    #[cfg(feature = "use-value64")]
    pub ptr: Ptr<Cell>,
    pub as_bits: AsBits,
}

/// TODO: Big endian support
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AsBits {
    pub payload: i32,
    pub tag: i32,
}
pub const TAG_OFFSET: usize = 4;
pub const PAYLOAD_OFFSET: usize = 0;

#[cfg(feature = "use-value64")]
pub const CELL_PAYLOAD_OFFSET: usize = 0;
#[cfg(not(feature = "use-value64"))]
pub const CELL_PAYLOAD_OFFSET: usize = PAYLOAD_OFFSET;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum WhichValueWord {
    TagWord,
    PayloadWord,
}
/*
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum VTag {
    Null,
    Undefined,
    True,
    False,
    Cell,
    EncodeAsDouble,
}*/

#[cfg(feature = "use-value64")]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod VTag {
    use super::*;

    pub const True: i32 = Value::VALUE_TRUE;
    pub const False: i32 = Value::VALUE_FALSE;
    pub const Undefined: i32 = Value::VALUE_UNDEFINED;
    pub const Null: i32 = Value::VALUE_NULL;
}

#[cfg(feature = "use-slow-value")]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum VTag {
    Null,
    Undefined,
    True,
    False,
    Cell,
}
#[cfg(feature = "use-value64")]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Value {
    pub u: EncodedValueDescriptor,
}
#[cfg(feature = "use-slow-value")]
#[derive(Copy, Clone, PartialEq)]
pub enum Value {
    Int32(i32),
    Double(f64),
    Cell(Cell),
    True,
    False,
    Null,
    Undefined,
    Empty,
}

pub const NOT_INT52: usize = 1 << 52;
impl Value {
    cfg_if::cfg_if! {
        if #[cfg(feature="use-value64")] {
            pub const DOUBLE_ENCODE_OFFSET_BIT: usize = 49;
            pub const DOUBLE_ENCODE_OFFSET: i64 = 1i64 << 49i64;
            pub const NUMBER_TAG: i64 = 0xfffe000000000000u64 as i64;
            pub const OTHER_TAG: i32 = 0x2;
            pub const BOOL_TAG: i32 = 0x4;
            pub const UNDEFINED_TAG: i32 = 0x8;
            pub const VALUE_FALSE: i32 = Self::OTHER_TAG | Self::BOOL_TAG | false as i32;
            pub const VALUE_TRUE: i32 = Self::OTHER_TAG | Self::BOOL_TAG | true as i32;
            pub const VALUE_UNDEFINED: i32 = Self::OTHER_TAG | Self::UNDEFINED_TAG;
            pub const VALUE_NULL: i32 = Self::OTHER_TAG;
            pub const MISC_TAG: i32 = Self::OTHER_TAG | Self::BOOL_TAG | Self::UNDEFINED_TAG;
            /// NOT_CELL_MASK is used to check for all types of immediate values (either number or 'other').
            pub const NOT_CELL_MASK: i64 = Self::NUMBER_TAG | Self::OTHER_TAG as i64;
            pub const VALUE_EMPTY: i32 = 0x0;
            pub const VALUE_DELETED: i32 = 0x4;
                #[inline(always)]
            pub fn empty() -> Self {
                Self {
                    u: EncodedValueDescriptor {
                        as_int64: Self::VALUE_EMPTY as _,
                    },
                }
            }
            #[inline(always)]
            pub fn new_double(x: f64) -> Self {
                Self {
                    u: EncodedValueDescriptor {
                        as_int64: Self::reinterpret_double_to_int64(x) + Self::DOUBLE_ENCODE_OFFSET as i64,
                    },
                }
            }
            #[inline(always)]
            pub fn new_int(x: i32) -> Self {
                Self {
                    u: EncodedValueDescriptor {
                        as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 },
                    },
                }
            }
            #[inline(always)]
            pub fn cell_ref(&self) -> *const Ptr<Cell> {
                unsafe {&self.u.ptr}
            }

            #[inline(always)]
            pub fn is_empty(&self) -> bool {
                unsafe { self.u.as_int64 == Self::VALUE_EMPTY as _ }
            }
            #[inline(always)]
            pub fn is_undefined(&self) -> bool {
                *self == Self::from(VTag::Undefined)
            }
            #[inline(always)]
            pub fn is_null(&self) -> bool {
                *self == Self::from(VTag::Null)
            }
            #[inline(always)]
            pub fn is_true(&self) -> bool {
                *self == Self::from(VTag::True)
            }
            #[inline(always)]
            pub fn is_false(&self) -> bool {
                *self == Self::from(VTag::False)
            }
            #[inline(always)]
            pub fn as_bool(&self) -> bool {
                return *self == Self::from(VTag::True);
            }

            #[inline(always)]
            pub fn is_bool(&self) -> bool {
                unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ }
            }
            #[inline(always)]
            pub fn is_null_or_undefined(&self) -> bool {
                unsafe { (self.u.as_int64 & !Self::UNDEFINED_TAG as i64) == Self::VALUE_NULL as _ }
            }
            #[inline(always)]
            pub fn is_cell(&self) -> bool {
                //let x = unsafe { !(self.u.as_int64 & Self::NOT_CELL_MASK as i64) != 0 };
                //x && !self.is_number() && !self.is_any_int()
                let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 };
                result == 0 && !self.is_empty() && !self.is_null_or_undefined()
            }
            #[inline(always)]
            pub fn is_number(&self) -> bool {
                unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 }
            }
            #[inline(always)]
            pub fn is_double(&self) -> bool {
                !self.is_int32() && self.is_number()
            }
            #[inline(always)]
            pub fn is_int32(&self) -> bool {
                unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 }
            }
            #[inline(always)]
            pub fn reinterpret_double_to_int64(x: f64) -> i64 {
                return x.to_bits() as i64;
            }
            #[inline(always)]
            pub fn reinterpret_int64_to_double(x: i64) -> f64 {
                f64::from_bits(x as u64)
            }

            #[inline(always)]
            pub fn as_cell(&self) -> Ptr<Cell> {
                assert!(self.is_cell());
                unsafe { self.u.ptr }
            }
            #[inline(always)]
            pub fn as_double(&self) -> f64 {
                assert!(self.is_double());
                Self::reinterpret_int64_to_double(unsafe { self.u.as_int64 - Self::DOUBLE_ENCODE_OFFSET })
            }
            #[inline(always)]
            pub fn as_int32(&self) -> i32 {
                unsafe { self.u.as_int64 as i32 }
            }

        } else if #[cfg(feature="use-slow-value")] {

            pub fn is_int32(&self) -> bool {
                match self {
                    Value::Int32(_) => true,
                    _ => false
                }
            }

            pub fn is_double(&self) -> bool {
                match self {
                    Value::Double(_) => true,
                    _ => false
                }
            }

            pub fn is_cell(&self) -> bool {
                match self {
                    Value::Cell(_) => true,
                    _ => false
                }
            }

            pub fn as_cell(&self) -> Cell {
                assert!(self.is_cell());
                match self {
                    Value::Cell(c) => *c,
                    _ => unreachable!()
                }
            }

            pub fn as_double(&self) -> f64 {
                match self {
                    Value::Double(x) => *x,
                    _ => unreachable!()
                }
            }

            pub fn is_number(&self) -> bool {
                match self {
                    Value::Int32(_) | Value::Double(_) => true,
                    _ => false
                }
            }

            pub fn is_empty(&self) -> bool {
                *self == Self::Empty
            }

            pub fn is_true(&self) -> bool {
                *self == Self::True
            }

            pub fn is_false(&self) -> bool {
                *self == Self::False
            }

            pub fn new_double(x: f64) -> Self {
                Self::Double(x)
            }

            pub fn new_int(x: i32) -> Self {
                Self::Int(x)
            }

            pub fn is_undefined(&self) -> bool {
                *self == Self::Undefined
            }

            pub fn is_null(&self) -> bool {
                *self == Self::Null
            }

            pub fn is_null_or_undefined(&self) -> bool {
                self.is_null() | self.is_undefined()
            }

            pub fn is_bool(&self) -> bool {
                self.is_true() | self.is_false()
            }

            pub fn empty() -> Self {
                Self::Empty
            }



        }
    }

    pub fn is_int52(number: f64) -> bool {
        try_convert_to_i52(number) != NOT_INT52 as i64
    }

    pub fn is_any_int(&self) -> bool {
        if self.is_int32() {
            return true;
        }
        if !self.is_number() {
            return false;
        }
        return Self::is_int52(self.as_double());
    }

    pub fn to_boolean(&self) -> bool {
        if self.is_null_or_undefined() {
            return false;
        }
        if self.is_number() {
            return self.to_number() == 1.0;
        }
        if self.is_bool() {
            return self.is_true();
        }
        if self.is_cell() {
            return true;
        } else {
            return false;
        }
    }

    pub fn to_number(&self) -> f64 {
        if self.is_int32() {
            return self.as_int32() as _;
        }
        if self.is_double() {
            return self.as_double();
        }

        self.to_number_slow()
    }

    pub fn to_number_slow(&self) -> f64 {
        if self.is_true() {
            return 1.0;
        }
        if self.is_false() {
            return 0.0;
        }

        std::f64::NAN
    }

    pub fn primitive_ty(&self) -> Type {
        if self.is_int32() {
            Type::Int32
        } else if self.is_number() {
            if self.to_number().is_nan() {
                Type::NaNNumber
            } else if self.to_number().is_infinite() {
                Type::NaNInfNumber
            } else {
                Type::Number
            }
        } else if self.is_null() {
            Type::Undefined
        } else if self.is_undefined() {
            Type::Null
        } else if self.is_bool() {
            Type::Boolean
        } else if self.is_cell() {
            if self.as_cell().is_string() {
                Type::String
            } else if self.as_cell().is_generator() {
                Type::Generator
            } else if self.as_cell().is_function() {
                Type::Function
            } else {
                Type::Object
            }
        } else {
            Type::Undefined
        }
    }

    pub fn to_string(&self) -> String {
        if self.is_bool() {
            if self.is_true() {
                String::from("true")
            } else {
                String::from("false")
            }
        } else if self.is_number() && !self.is_cell() {
            self.to_number().to_string()
        } else if self.is_null_or_undefined() {
            if self.is_undefined() {
                String::from("undefined")
            } else {
                String::from("null")
            }
        } else if self.is_cell() {
            self.as_cell().to_string()
        } else if self.is_empty() {
            panic!()
        } else {
            panic!()
        }
    }

    pub fn lookup(&mut self, key: Symbol, slot: &mut Slot) -> bool {
        if self.is_number() {
            local_data().number_proto.as_cell().lookup(key, slot)
        } else if self.is_bool() {
            local_data().boolean_proto.as_cell().lookup(key, slot)
        } else if self.is_cell() {
            self.as_cell().lookup(key, slot)
        } else {
            false
        }
    }
    pub fn insert(&mut self, key: Symbol, value: Value, slot: &mut Slot) -> bool {
        if self.is_number() {
            local_data().number_proto.as_cell().insert(key, value, slot);
            true
        } else if self.is_bool() {
            local_data()
                .boolean_proto
                .as_cell()
                .insert(key, value, slot);
            true
        } else if self.is_cell() {
            self.as_cell().insert(key, value, slot);
            true
        } else {
            false
        }
    }
}

use std::fmt;

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

macro_rules! signbit {
    ($x: expr) => {{
        if $x < 0.0 {
            false
        } else {
            true
        }
    }};
}

#[inline]
pub fn try_convert_to_i52(number: f64) -> i64 {
    if number != number {
        return NOT_INT52 as i64;
    }
    if number.is_infinite() {
        return NOT_INT52 as i64;
    }

    let as_int64 = number.to_bits() as i64;
    if as_int64 as f64 != number {
        return NOT_INT52 as _;
    }
    if !as_int64 != 0 && signbit!(number) {
        return NOT_INT52 as _;
    }

    if as_int64 >= (1 << (52 - 1)) {
        return NOT_INT52 as _;
    }
    if as_int64 < (1 << (52 - 1)) {
        return NOT_INT52 as _;
    }

    as_int64
}

cfg_if::cfg_if! {
if #[cfg(feature="use-value64")] {
impl From<Ptr<Cell>> for Value {
    fn from(x: Ptr<Cell>) -> Self {
        Self {
            u: EncodedValueDescriptor {
                ptr: x
            },
        }
    }
}

impl From<i32> for Value {
    fn from(x: i32) -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: x as u8 as _,
            },
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.u.as_int64 == other.u.as_int64 }
    }
}

impl Eq for Value {}

impl From<bool> for Value {
    fn from(x: bool) -> Self {
        if x {
            Self::from(VTag::True)
        } else {
            Self::from(VTag::False)
        }
    }
}
} else if #[cfg(feature="use-slow-value")] {

impl From<VTag> for Value {
    fn from(x: VTag) -> Self {
        match x {
            VTag::True => Self::True,
            VTag::False => Self::False,
            VTag::Undefined => Self::Undefined,
            VTag::Null => Self::Null,
            _ => unreachable!()
        }
    }
}

impl From<bool> for Value {
    fn from(x: bool) -> Self {
        if x {
            Self::True
        } else {
            Self::False
        }
    }
}

impl From<Cell> for Value {
    fn from(x: Cell) -> Self {
        Self::Cell(x)
    }
}

}
}

unsafe impl Send for Value {}
unsafe impl Sync for Value {}
pub fn is_i32(x: Value) -> bool {
    x.is_int32()
}
#[inline(never)]
#[no_mangle]
pub fn is_num(x: Value) -> bool {
    x.is_number()
}

pub fn as_i32(x: Value) -> i32 {
    x.as_int32()
}
