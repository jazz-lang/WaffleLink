use crate::object::*;
use crate::pure_nan::*;

#[derive(Copy, Clone)]
#[repr(C, align(8))]
pub union EncodedValueDescriptor {
    pub as_int64: i64,
    #[cfg(feature = "value32-64")]
    pub as_double: f64,
    pub cell: Ref<Obj>,
    pub as_bits: AsBits,
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
impl From<Ref<Obj>> for Value {
    fn from(x: Ref<Obj>) -> Self {
        Self {
            u: EncodedValueDescriptor { cell: x },
        }
    }
}
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
#[repr(transparent)]
pub struct Value {
    pub u: EncodedValueDescriptor,
}

#[cfg(all(feature = "value32-64", not(feature = "value64")))]
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
    pub fn as_cell(self) -> Ref<Obj> {
        unsafe { std::mem::transmute(self.payload()) }
    }
    pub fn as_cell_ref<'a>(&'a self) -> &'a Ref<Obj> {
        unsafe { std::mem::transmue(&self.u.as_bits.payload) }
    }
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
    /// These special values are never visible to code; Empty is used to represent
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
                as_int64: Self::VALUE_FALSE,
            },
        }
    }
    pub fn true_() -> Self {
        Self {
            u: EncodedValueDescriptor {
                as_int64: Self::VALUE_TRUE,
            },
        }
    }
    pub fn as_cell(self) -> Ref<Obj> {
        unsafe { self.u.cell }
    }
    pub fn as_cell_ref<'a>(&'a self) -> &'a Ref<Obj> {
        unsafe { &self.u.cell }
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
        unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE }
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
    pub fn is_double(self) -> bool {
        !self.is_int32() && self.is_number()
    }
    pub fn as_double(&self) -> f64 {
        assert!(self.is_double());
        unsafe { f64::from_bits((self.u.as_int64 - Self::DOUBLE_ENCODE_OFFSET) as u64) }
    }
    #[inline]
    pub fn is_any_int(&self) -> bool {
        if self.is_int32() {
            true
        } else if !self.is_number() {
            false
        } else {
            try_convert_to_i52(self.as_double()) != NOT_INT52 as i64
        }
    }
    #[inline(always)]
    pub fn as_int32(&self) -> i32 {
        debug_assert!(self.is_int32());
        unsafe { self.u.as_int64 as i32 }
    }
    #[inline]
    pub fn as_any_int(&self) -> i64 {
        assert!(self.is_any_int());
        if self.is_int32() {
            return self.as_int32() as i64;
        }
        return self.as_double() as i64;
    }
    #[inline]
    pub fn is_int32_as_any_int(&self) -> bool {
        if !self.is_any_int() {
            return false;
        }
        let value = self.as_any_int();
        return value >= i32::min_value() as i64 && value <= i32::max_value() as i64;
    }
    #[inline]
    pub fn as_int32_as_any_int(&self) -> i32 {
        assert!(self.is_int32_as_any_int());
        if self.is_int32() {
            return self.as_int32();
        }
        self.as_double() as i32
    }
    #[inline]
    pub fn is_uint32_as_any_int(&self) -> bool {
        if !self.is_any_int() {
            return false;
        }
        let value = self.as_any_int();
        return value >= 0 as i64 && value <= u32::max_value() as i64;
    }
    #[inline]
    pub fn as_uint32_as_any_int(&self) -> u32 {
        assert!(self.is_int32_as_any_int());
        if self.is_int32() {
            return self.as_int32() as u32;
        }
        self.as_double() as u32
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.u.as_int64 == other.u.as_int64 }
    }
}

impl Eq for Value {}
impl Value {
    #[inline]
    pub fn is_uint32(&self) -> bool {
        self.is_int32() && self.as_int32() >= 0
    }

    #[inline]
    pub fn to_int32(&self) -> i32 {
        let d = self.to_number();
        d as i32
    }
    pub fn to_uint32(&self) -> u32 {
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
    #[inline]
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
            true
        } else {
            false
        }
    }
    #[inline]
    pub fn new_bool(x: bool) -> Self {
        if x {
            Self::true_()
        } else {
            Self::false_()
        }
    }
    #[inline]
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

pub const NOT_INT52: u64 = 1 << 52;

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

/*
impl Hash for Value {
    fn hash<H: Hasher>(&self, s: &mut H) {
        unsafe {
            if self.is_cell() {
                let cell = self.as_cell();
                match cell.type_of() {
                    WaffleType::String => {
                        let string = cell.as_string();
                        string.value().string.hash(s);
                        return;
                    }
                    WaffleType::Array => {
                        let array = cell.as_array();
                        for ix in 0..array.value().len() {
                            array.value().at(ix).hash(s);
                        }
                        array.value().len().hash(s);
                    }
                    _ => (),
                }
            }
            self.u.as_int64.hash(s);
        }
    }}
}*/
use std::hash::{Hash, Hasher};
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.is_number() {
            if self.is_int32() {
                state.write_i32(self.as_int32());
            } else {
                state.write_u64(self.as_number().to_bits())
            }
        } else if self.is_boolean() {
            state.write_u8(self.to_boolean() as u8);
        } else if self.is_undefined_or_null() {
            state.write_u8(0);
        } else if self.is_cell() {
            let cell = self.as_cell();
            if cell.is_string() {
                let s = cell.cast::<crate::object::WaffleString>();
                state.write_usize(s.len());
                for i in 0..s.len() {
                    let c = s.get_at(i);
                    state.write_u32(c as u32);
                }
            } else {
                state.write_usize(cell.ptr as usize);
            }
        } else {
            unreachable!()
        }
    }
}
