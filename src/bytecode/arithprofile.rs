use crate::runtime::value::Value;
use crate::types::*;
#[derive(Clone, Copy)]
pub struct ObservedType {
    pub bits: u8,
}

impl ObservedType {
    pub const EMPTY: u8 = 0x0;
    pub const INT32: u8 = 0x1;
    pub const NUMBER: u8 = 0x02;
    pub const NON_NUMBER: u8 = 0x04;
    pub const NUM_BITS_NEEDED: u8 = 3;
    pub const fn new(bits: u8) -> Self {
        Self { bits }
    }

    pub const fn empty() -> Self {
        Self { bits: Self::EMPTY }
    }

    pub const fn saw_int32(&self) -> bool {
        self.bits & Self::INT32 != 0
    }

    pub const fn is_only_int32(&self) -> bool {
        self.bits == Self::INT32
    }

    pub const fn saw_number(&self) -> bool {
        self.bits & Self::NUMBER != 0
    }

    pub const fn is_only_number(&self) -> bool {
        self.bits == Self::NUMBER
    }

    pub const fn is_empty(&self) -> bool {
        !self.bits == 0
    }

    pub const fn with_int32(&self) -> Self {
        Self::new(self.bits | Self::INT32)
    }

    pub const fn with_number(&self) -> Self {
        Self::new(self.bits | Self::NUMBER)
    }

    pub const fn with_non_number(&self) -> Self {
        Self::new(self.bits | Self::NON_NUMBER)
    }

    pub const fn without_non_number(&self) -> Self {
        Self::new(self.bits | !Self::NON_NUMBER)
    }
}
#[derive(Clone, Copy, Default)]
pub struct ObservedResults {
    pub bits: u8,
}

impl ObservedResults {
    pub const NON_NEG_ZERO_DOUBLE: u8 = 1 << 0;
    pub const NEG_ZERO_DOUBLE: u8 = 1 << 1;
    pub const NON_NUMERIC: u8 = 1 << 2;
    pub const INT32_OVERFLOW: u8 = 1 << 3;
    pub const INT52_OVERFLOW: u8 = 1 << 4;
    pub const BIGINT: u8 = 1 << 5;
    pub const NUM_BITS_NEEDED: u16 = 6;
    pub const fn new(x: u8) -> Self {
        Self { bits: x }
    }

    pub const fn did_observe_non_int32(&self) -> bool {
        (self.bits
            & (Self::NON_NEG_ZERO_DOUBLE
                | Self::NEG_ZERO_DOUBLE
                | Self::NON_NUMERIC
                | Self::BIGINT))
            != 0
    }

    pub const fn did_observe_double(&self) -> bool {
        (self.bits & (Self::NON_NEG_ZERO_DOUBLE | Self::NEG_ZERO_DOUBLE)) != 0
    }

    pub const fn did_observe_non_neg_zero_double(&self) -> bool {
        (self.bits & (Self::NON_NEG_ZERO_DOUBLE)) != 0
    }

    pub const fn did_observe_neg_zero_double(&self) -> bool {
        (self.bits & (Self::NEG_ZERO_DOUBLE)) != 0
    }

    pub const fn did_observe_non_numeric(&self) -> bool {
        (self.bits & Self::NON_NUMERIC) != 0
    }
    pub const fn did_observe_bigint(&self) -> bool {
        (self.bits & Self::BIGINT) != 0
    }

    pub const fn did_observe_int32_overflow(&self) -> bool {
        (self.bits & Self::INT32_OVERFLOW) != 0
    }

    pub const fn did_observe_int52_overflow(&self) -> bool {
        (self.bits & Self::INT52_OVERFLOW) != 0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum ArithProfileType {
    Binary,
    Unary,
}

pub struct ArithProfile {
    pub ty: ArithProfileType,
    pub bits: u16,
}

const RHS_OBSERVED_TYPE_SHIFT: u16 = ObservedResults::NUM_BITS_NEEDED;
const LHS_OBSERVED_TYPE_SHIFT: u16 = RHS_OBSERVED_TYPE_SHIFT + ObservedResults::NUM_BITS_NEEDED;

impl ArithProfile {
    #[inline]
    pub const fn observed_results(&self) -> ObservedResults {
        return ObservedResults::new(
            (self.bits & ((1 << ObservedResults::NUM_BITS_NEEDED) - 1)) as u8,
        );
    }
    #[inline]
    pub const fn has_bits(&self, mask: u16) -> bool {
        self.bits & mask != 0
    }
    #[inline]
    pub fn set_bit(&mut self, mask: u16) {
        self.bits |= mask;
    }

    #[inline]
    pub const fn did_observe_non_int32(&self) -> bool {
        self.observed_results().did_observe_non_int32()
    }

    #[inline]
    pub const fn did_observe_double(&self) -> bool {
        self.observed_results().did_observe_double()
    }

    #[inline]
    pub const fn did_observe_non_neg_zero_double(&self) -> bool {
        self.observed_results().did_observe_non_neg_zero_double()
    }

    #[inline]
    pub const fn did_observe_neg_zero_double(&self) -> bool {
        self.observed_results().did_observe_neg_zero_double()
    }

    #[inline]
    pub const fn did_observe_non_numeric(&self) -> bool {
        self.observed_results().did_observe_non_numeric()
    }

    #[inline]
    pub const fn did_observe_bigint(&self) -> bool {
        self.observed_results().did_observe_bigint()
    }
    #[inline]
    pub const fn did_observe_int32_overflow(&self) -> bool {
        self.observed_results().did_observe_int32_overflow()
    }

    #[inline]
    pub const fn did_observe_int52_overflow(&self) -> bool {
        self.observed_results().did_observe_int52_overflow()
    }

    pub fn observe_result(&mut self, value: &Value) {
        if value.is_int32() {
            return;
        }
        if value.is_number() {
            self.bits |= ObservedResults::INT32_OVERFLOW as u16
                | ObservedResults::INT52_OVERFLOW as u16
                | ObservedResults::NON_NEG_ZERO_DOUBLE as u16
                | ObservedResults::NEG_ZERO_DOUBLE as u16;
            return;
        }
        self.bits |= ObservedResults::NON_NUMERIC as u16;
    }
}
