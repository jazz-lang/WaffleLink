//! Value implementation is exactly the same as in JSC and uses NaN-boxing.
use super::cell::*;
use super::pure_nan::*;
use super::*;
use cgc::api::Handle;
#[cfg(all(target_pointer_width = "64", feature = "value32-64"))]
compile_error!("Cannot use value32-64 feature on 64 target");

#[derive(Copy, Clone)]
#[repr(C, align(8))]
union EncodedValueDescriptor {
    as_int64: i64,
    #[cfg(feature = "value32-64")]
    as_double: f64,
    cell: Handle<Cell>,
    as_bits: AsBits,
}
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
#[cfg(target_endian = "big")]
pub struct AsBits {
    pub tag: i32,
    pub payload: i32,
}
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
#[cfg(target_endian = "little")]
pub struct AsBits {
    pub payload: i32,
    pub tag: i32,
}

pub fn tag_offset() -> usize {
    offset_of!(AsBits, tag)
}
pub fn payload_offset() -> usize {
    offset_of!(AsBits, payload)
}

pub fn cell_payload_offset() -> usize {
    #[cfg(feature = "value64")]
    {
        0
    }
    #[cfg(not(feature = "value64"))]
    {
        payload_offset()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WhichValueWord {
    Tag = 0,
    Payload,
}
#[cfg(feature = "value32-64")]
pub const INT32_TAG: i32 = 0xffffffffu32 as i32;
#[cfg(feature = "value32-64")]
pub const BOOL_TAG: i32 = 0xfffffffeu32 as i32;
#[cfg(feature = "value32-64")]
pub const UNDEFINED_TAG: i32 = 0xfffffffdu32 as i32;
#[cfg(feature = "value32-64")]
pub const NULL_TAG: i32 = 0xfffffffcu32 as i32;
#[cfg(feature = "value32-64")]
pub const CELL_TAG: i32 = 0xfffffffbu32 as i32;
#[cfg(feature = "value32-64")]
pub const EMPTY_TAG: i32 = 0xfffffffau32 as i32;
#[cfg(feature = "value32-64")]
pub const DELETED_TAG: i32 = 0xfffffff9u32 as i32;
#[cfg(feature = "value32-64")]
pub const LOWEST_TAG: i32 = DELETED_TAG;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum JSTag {
    Null,
    Undefined,
    True,
    False,
    Cell,
    AsDouble,
}

#[derive(Copy, Clone)]
pub struct Value {
    u: EncodedValueDescriptor,
}

#[cfg(feature = "value32-64")]
impl Value {
    /*
     * On 32-bit platforms `value32-64` feature should be enabled, and we use a NaN-encoded
     * form for immediates.
     *
     * The encoding makes use of unused NaN space in the IEEE754 representation.  Any value
     * with the top 13 bits set represents a QNaN (with the sign bit set).  QNaN values
     * can encode a 51-bit payload.  Hardware produced and C-library payloads typically
     * have a payload of zero.  We assume that non-zero payloads are available to encode
     * pointer and integer values.  Since any 64-bit bit pattern where the top 15 bits are
     * all set represents a NaN with a non-zero payload, we can use this space in the NaN
     * ranges to encode other values (however there are also other ranges of NaN space that
     * could have been selected).
     *
     * For Values that do not contain a double value, the high 32 bits contain the tag
     * values listed in the enums below, which all correspond to NaN-space. In the case of
     * cell, integer and bool values the lower 32 bits (the 'payload') contain the pointer
     * integer or boolean value; in the case of all other tags the payload is 0.
     */
    pub fn tag(self) -> u32 {
        unsafe { self.u.as_bits.tag }
    }
    pub fn payload(self) -> i32 {
        unsafe { self.u.as_bits.payload }
    }

    pub(crate) fn with_tag_payload(tag: i32, payload: i32) -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_bits: AsBits { tag, payload },
            },
        }
    }

    pub fn default() -> Self {
        Self::with_tag_payload(EMPTY_TAG, 0)
    }

    pub fn null() -> Self {
        Self::with_tag_payload(NULL_TAG, 0)
    }

    pub fn undefined() -> Self {
        Self::with_tag_payload(UNDEFINED_TAG, 0)
    }
    pub fn true_() -> Self {
        Self::with_tag_payload(BOOL_TAG, 1)
    }
    pub fn false_() -> Self {
        Self::with_tag_payload(BOOL_TAG, 0)
    }

    pub fn cell(cell: Handle<Cell>) -> Self {
        Self::with_tag_payload(CELL_TAG, unsafe {
            std::mem::transmute::<i32>(cell) /* sizeof(void*) == sizeof(i32) on 32 bit machines,this cast is safe*/
        })
    }

    pub fn is_empty(&self) -> bool {
        self.tag() == EMPTY_TAG
    }

    pub fn is_null(&self) -> bool {
        self.tag() == NULL_TAG
    }

    pub fn is_undefined(&self) -> bool {
        self.tag() == UNDEFINED_TAG
    }

    pub fn is_undefined_or_null(&self) -> bool {
        self.is_undefined() || self.is_null()
    }
    pub fn is_cell(&self) -> bool {
        self.tag() == CELL_TAG
    }

    pub fn is_int32(&self) -> bool {
        self.tag() == INT32_TAG
    }

    pub fn is_double(&self) -> bool {
        self.tag() < LOWEST_TAG
    }

    pub fn is_true(&self) -> bool {
        self.tag() == BOOL_TAG && self.payload() != 0
    }

    pub fn is_false(&self) -> bool {
        self.tag() == BOOL_TAG && self.payload() == 0
    }

    pub fn as_int32(&self) -> bool {
        self.payload()
    }

    pub fn as_double(&self) -> bool {
        unsafe { self.u.as_double }
    }

    pub fn as_cell(&self) -> Handle<Cell> {
        //assert!(self.is_cell(), "Value payload is not a cell!");
        unsafe { std::mem::transmute(self.payload()) }
    }
    pub fn as_cell_ref(&self) -> &Handle<Cell> {
        assert!(self.is_cell(), "Value payload is not a cell!");
        unsafe { std::mem::transmute(&self.payload()) }
    }

    pub fn new_double(f: f64) -> Self {
        assert!(!is_impure_nan(f));
        Self {
            u: EncodedValueDescriptor { as_double: f },
        }
    }

    pub fn new_int(x: i32) -> Self {
        Self::with_tag_payload(INT32_TAG, x)
    }

    pub fn is_number(&self) -> bool {
        self.is_int32() || self.is_double()
    }

    pub fn is_boolean(&self) -> bool {
        self.tag() == BOOL_TAG
    }

    pub fn as_boolean(&self) -> bool {
        assert!(self.is_boolean());
        self.payload() != 0
    }

    const _XX: () = {
        impl PartialEq for Value {
            fn eq(&self, other: &Self) -> bool {
                unsafe { self.u.as_int64 == other.u.as_int64 }
            }
        }

        impl Eq for Value {}
    };
}

#[cfg(feature = "value64")]
impl Value {
    /*
     * On 64-bit platforms `value64` feature should be enabled, and we use a NaN-encoded
     * form for immediates.
     *
     * The encoding makes use of unused NaN space in the IEEE754 representation.  Any value
     * with the top 13 bits set represents a QNaN (with the sign bit set).  QNaN values
     * can encode a 51-bit payload.  Hardware produced and C-library payloads typically
     * have a payload of zero.  We assume that non-zero payloads are available to encode
     * pointer and integer values.  Since any 64-bit bit pattern where the top 15 bits are
     * all set represents a NaN with a non-zero payload, we can use this space in the NaN
     * ranges to encode other values (however there are also other ranges of NaN space that
     * could have been selected).
     *
     * This range of NaN space is represented by 64-bit numbers begining with the 15-bit
     * hex patterns 0xFFFC and 0xFFFE - we rely on the fact that no valid double-precision
     * numbers will fall in these ranges.
     *
     * The top 15-bits denote the type of the encoded Value:
     *
     *     Pointer {  0000:PPPP:PPPP:PPPP
     *              / 0002:****:****:****
     *     Double  {         ...
     *              \ FFFC:****:****:****
     *     Integer {  FFFE:0000:IIII:IIII
     *
     * The scheme we have implemented encodes double precision values by performing a
     * 64-bit integer addition of the value 2^49 to the number. After this manipulation
     * no encoded double-precision value will begin with the pattern 0x0000 or 0xFFFE.
     * Values must be decoded by reversing this operation before subsequent floating point
     * operations may be peformed.
     *
     * 32-bit signed integers are marked with the 16-bit tag 0xFFFE.
     *
     * The tag 0x0000 denotes a pointer, or another form of tagged immediate. Boolean,
     * null and undefined values are represented by specific, invalid pointer values:
     *
     *     False:     0x06
     *     True:      0x07
     *     Undefined: 0x0a
     *     Null:      0x02
     *
     * These values have the following properties:
     * - Bit 1 (OtherTag) is set for all four values, allowing real pointers to be
     *   quickly distinguished from all immediate values, including these invalid pointers.
     * - With bit 3 masked out (UndefinedTag), Undefined and Null share the
     *   same value, allowing null & undefined to be quickly detected.
     *
     * No valid Value will have the bit pattern 0x0, this is used to represent array
     * holes, and as a C++ 'no value' result (e.g. Value() has an internal value of 0).
     *
     * This representation works because of the following things:
     * - It cannot be confused with a Double or Integer thanks to the top bits
     * - It cannot be confused with a pointer to a Cell, thanks to bit 1 which is set to true
     * - It cannot be confused with a pointer to wasm thanks to bit 0 which is set to false
     * - It cannot be confused with true/false because bit 2 is set to false
     * - It cannot be confused for null/undefined because bit 4 is set to true
     */

    /// This value is 2^49, used to encode doubles such that the encoded value will begin
    /// with a 15-bit pattern within the range 0x0002..0xFFFC.
    pub const DOUBLE_ENCODE_OFFSET_BIT: i64 = 49;
    pub const DOUBLE_ENCODE_OFFSET: i64 = 1 << Self::DOUBLE_ENCODE_OFFSET_BIT;
    /// If all bits in the mask are set, this indicates an integer number,
    /// if any but not all are set this value is a double precision number.
    pub const NUMBER_TAG: i64 = 0xfffe000000000000u64 as i64;
    /// The following constant is used for a trick in the implementation of strictEq, to detect if either of the arguments is a double
    pub const LOWEST_OF_HIGH_BITS: i64 = 1 << 49;
    /// All non-numeric (bool, null, undefined) immediates have bit 2 set.
    pub const OTHER_TAG: i64 = 0x2;
    pub const BOOL_TAG: i64 = 0x4;
    pub const UNDEFINED_TAG: i64 = 0x8;
    pub const VALUE_FALSE: i64 = Self::OTHER_TAG | Self::BOOL_TAG | 0; // `0` stands for `false`.
    pub const VALUE_TRUE: i64 = Self::OTHER_TAG | Self::BOOL_TAG | 1; // `1` stands for `true`.
    pub const VALUE_UNDEFINED: i64 = Self::OTHER_TAG | Self::UNDEFINED_TAG;
    pub const VALUE_NULL: i64 = Self::OTHER_TAG;
    pub const MISC_TAG: i64 = Self::OTHER_TAG | Self::BOOL_TAG | Self::UNDEFINED_TAG;
    /// NOT_CELL_MASK is used to check for all types of immediate values (either number or 'other').
    pub const NOT_CELL_MASK: i64 = (Self::NUMBER_TAG as u64 | Self::OTHER_TAG as u64) as i64;
    /// These special values are never visible to JavaScript code; Empty is used to represent
    /// Array holes, and for uninitialized Values. Deleted is used in hash table code.
    /// These values would map to cell types in the Value encoding, but not valid GC cell
    /// pointer should have either of these values (Empty is null, deleted is at an invalid
    /// alignment for a GC cell, and in the zero page).
    pub const VALUE_EMPTY: i64 = 0x0;
    pub const VALUE_DELETED: i64 = 0x4;
    // 0x0 can never occur naturally because it has a tag of 00, indicating a pointer value, but a payload of 0x0, which is in the (invalid) zero page.
    pub fn default() -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: Self::VALUE_EMPTY,
            },
        }
    }

    pub fn cell(x: Handle<Cell>) -> Self {
        Self {
            u: EncodedValueDescriptor { cell: x },
        }
    }

    pub fn is_empty(&self) -> bool {
        unsafe { self.u.as_int64 == Self::VALUE_EMPTY }
    }

    pub fn undefined() -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: Self::VALUE_UNDEFINED,
            },
        }
    }

    pub fn null() -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: Self::VALUE_NULL,
            },
        }
    }
    pub fn false_() -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: Self::VALUE_TRUE,
            },
        }
    }
    pub fn true_() -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: Self::VALUE_FALSE,
            },
        }
    }

    pub fn is_number(&self) -> bool {
        unsafe { self.u.as_int64 & Self::NUMBER_TAG != 0 }
    }

    pub fn is_int32(&self) -> bool {
        unsafe { (self.u.as_int64 & Self::NUMBER_TAG) == Self::NUMBER_TAG }
    }

    pub fn is_undefined(&self) -> bool {
        *self == Self::undefined()
    }

    pub fn is_null(&self) -> bool {
        *self == Self::null()
    }

    pub fn is_true(&self) -> bool {
        *self == Self::true_()
    }

    pub fn is_false(&self) -> bool {
        *self == Self::false_()
    }

    pub fn is_undefined_or_null(&self) -> bool {
        unsafe { (self.u.as_int64 & !Self::UNDEFINED_TAG) == Self::VALUE_NULL }
    }

    pub fn is_boolean(&self) -> bool {
        unsafe { self.u.as_int64 & !Self::UNDEFINED_TAG == Self::VALUE_FALSE }
    }
    pub fn is_cell(&self) -> bool {
        unsafe { (self.u.as_int64 & Self::NOT_CELL_MASK) == 0 }
    }

    pub fn new_double(x: f64) -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: x.to_bits() as i64 + Self::DOUBLE_ENCODE_OFFSET,
            },
        }
    }

    pub fn new_int(x: i32) -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: Value::NUMBER_TAG | (x as u32 as i64),
            },
        }
    }
    pub fn is_double(&self) -> bool {
        !self.is_int32() && self.is_number()
    }
    pub fn as_double(&self) -> f64 {
        assert!(self.is_double());
        unsafe { f64::from_bits((self.u.as_int64 - Self::DOUBLE_ENCODE_OFFSET) as u64) }
    }

    pub fn as_cell(&self) -> Handle<Cell> {
        //assert!(self.is_cell());
        unsafe { self.u.cell }
    }
    pub fn as_cell_ref(&self) -> &Handle<Cell> {
        assert!(self.is_cell());
        unsafe { &self.u.cell }
    }

    pub fn is_any_int(&self) -> bool {
        if self.is_int32() {
            true
        } else if !self.is_number() {
            false
        } else {
            try_convert_to_i52(self.as_double()) != NOT_INT52 as i64
        }
    }
    pub fn as_int32(&self) -> i32 {
        assert!(self.is_int32());
        unsafe { self.u.as_int64 as i32 }
    }
    pub fn as_any_int(&self) -> i64 {
        assert!(self.is_any_int());
        if self.is_int32() {
            return self.as_int32() as i64;
        }
        return self.as_double().trunc() as i64;
    }

    pub fn is_int32_as_any_int(&self) -> bool {
        if !self.is_any_int() {
            return false;
        }
        let value = self.as_any_int();
        return value >= i32::min_value() as i64 && value <= i32::max_value() as i64;
    }

    pub fn as_int32_as_any_int(&self) -> i32 {
        assert!(self.is_int32_as_any_int());
        if self.is_int32() {
            return self.as_int32();
        }
        self.as_double().trunc() as i32
    }

    pub fn is_uint32_as_any_int(&self) -> bool {
        if !self.is_any_int() {
            return false;
        }
        let value = self.as_any_int();
        return value >= 0 as i64 && value <= u32::max_value() as i64;
    }

    pub fn as_uint32_as_any_int(&self) -> u32 {
        assert!(self.is_int32_as_any_int());
        if self.is_int32() {
            return self.as_int32() as u32;
        }
        self.as_double().trunc() as u32
    }

    const _XX: () = {
        impl PartialEq for Value {
            fn eq(&self, other: &Self) -> bool {
                unsafe { self.u.as_int64 == other.u.as_int64 }
            }
        }

        impl Eq for Value {}
    };
}

impl Value {
    pub fn is_uint32(&self) -> bool {
        self.is_int32() && self.as_int32() >= 0
    }

    pub fn to_int32(&self) -> i32 {
        let d = self.to_number();
        d as i32
    }
    pub fn to_uint32(&self) -> u32 {
        // The only difference between to_int32 and to_uint32 is that to_uint32 reinterprets resulted i32 value as u32.
        // https://tc39.es/ecma262/#sec-touint32
        self.to_int32() as u32
    }
    #[inline(always)]
    pub fn to_number(&self) -> f64 {
        if self.is_int32() {
            return self.as_int32() as f64;
        }
        if self.is_double() {
            return self.as_double();
        }
        self.to_number_slow_case()
    }
    pub fn to_number_slow_case(&self) -> f64 {
        assert!(!self.is_int32() && !self.is_double());
        if self.is_cell() {
            unimplemented!()
        }
        if self.is_true() {
            return 1.0;
        }
        if self.is_undefined() {
            return pure_nan();
        } else {
            0.0 // null and false both convert to 0.
        }
    }

    pub fn as_number(&self) -> f64 {
        assert!(self.is_number());
        if self.is_int32() {
            self.as_int32() as f64
        } else {
            self.as_double()
        }
    }
    pub fn to_boolean(&self) -> bool {
        if self.is_number() {
            if self.is_int32() {
                self.as_int32() != 0
            } else {
                self.to_number() != 0.0
            }
        } else if self.is_boolean() {
            self.is_true()
        } else if self.is_undefined_or_null() {
            false
        } else if self.is_cell() {
            match self.as_cell().value {
                CellValue::String(ref x) => x.len() != 0,
                CellValue::Array(ref x) => x.len() != 0,
                CellValue::ByteArray(ref x) => x.len() != 0,
                _ => true,
            }
        } else {
            false
        }
    }
    pub fn to_string(&self, rt: &mut Runtime) -> Result<String, Self> {
        if self.is_number() {
            Ok(self.to_number().to_string())
        } else if self.is_true() || self.is_false() {
            if self.is_true() {
                Ok("true".to_string())
            } else {
                Ok("false".to_string())
            }
        } else if self.is_undefined_or_null() {
            if self.is_null() {
                Ok("null".to_string())
            } else {
                Ok("undefined".to_string())
            }
        } else {
            if self.is_cell() {
                if let CellValue::String(ref s) = self.as_cell().value {
                    return Ok((**s).clone());
                }
                if let CellValue::Function(ref f) = self.as_cell().value {
                    match f {
                        Function::Native { name, .. } => {
                            let name = name.to_string(rt)?;
                            return Ok(format!("function {}(...)", name));
                        }
                        Function::Regular(regular) => {
                            let name = regular.name.to_string(rt)?;
                            return Ok(format!("function {}(...)", name));
                        }
                        _ => unimplemented!(),
                    }
                }
                let key = rt.allocate_string("toString");
                let x = self.lookup(rt, Value::from(key.to_heap()))?;
                if let Some(to_string) = x {
                    if to_string.is_cell() {
                        if let CellValue::Function(_) = to_string.as_cell().value {
                            return match rt.call(to_string, *self, &[]) {
                                Ok(x) => x.to_string(rt),
                                Err(e) => Err(e),
                            };
                        } else {
                            return Err(Value::from(
                                rt.allocate_string("toString is not a function,expected function"),
                            ));
                        }
                    }
                }
            }
            unsafe {
                println!("0{:x}", self.u.as_int64);
            }
            unimplemented!()
        }
    }

    pub fn lookup(&self, rt: &mut Runtime, key: Value) -> Result<Option<Value>, Value> {
        let r: &mut Runtime = unsafe { &mut *(rt as *mut Runtime) };
        if self.is_cell() {
            self.as_cell().lookup(r, key)
        } else if self.is_number() {
            rt.number_prototype.lookup(r, key)
        } else if self.is_boolean() {
            rt.boolean_prototype.lookup(r, key)
        } else {
            Ok(None)
        }
    }
    pub fn put(&self, rt: &mut Runtime, key: Value, val: Value) -> Result<(), Value> {
        let r: &mut Runtime = unsafe { &mut *(rt as *mut Runtime) };
        if self.is_cell() {
            self.as_cell().put(rt, key, val)
        } else if self.is_number() {
            rt.number_prototype.put(r, key, val)
        } else if self.is_boolean() {
            rt.boolean_prototype.put(r, key, val)
        } else {
            Ok(())
        }
    }
    pub fn number(x: f64) -> Self {
        if x as i32 as f64 == x {
            Self::new_int(x as i32)
        } else {
            Self::new_double(x)
        }
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

pub const NOT_INT52: usize = 1 << 52;

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

use cgc::api::{Finalizer, Traceable, Tracer};

impl Traceable for Value {
    fn trace_with(&self, tracer: &mut Tracer) {
        if self.is_cell() {
            tracer.trace(self.as_cell_ref());
        }
    }
}

impl Finalizer for Value {}

impl From<Handle<Cell>> for Value {
    fn from(x: Handle<Cell>) -> Self {
        Self {
            u: EncodedValueDescriptor { cell: x },
        }
    }
}

use cgc::api::*;

impl From<Rooted<Cell>> for Value {
    fn from(x: Rooted<Cell>) -> Self {
        Self {
            u: EncodedValueDescriptor { cell: x.to_heap() },
        }
    }
}
impl From<&Rooted<Cell>> for Value {
    fn from(x: &Rooted<Cell>) -> Self {
        Self {
            u: EncodedValueDescriptor { cell: x.to_heap() },
        }
    }
}
use std::hash::{Hash, Hasher};
impl Hash for Value {
    fn hash<H: Hasher>(&self, s: &mut H) {
        unsafe {
            self.u.as_int64.hash(s);
        }
    }
}
