/*
 *   Copyright (c) 2020 Adel Prokurov
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

pub type Type = u8;
pub const TYPE_INT32: Type = 1;
pub const TYPE_MAYBE_NUMBER: Type = 0x02;
pub const TYPE_MAYBE_STRING: Type = 0x04;
pub const TYPE_MAYBE_NULL: Type = 0x08;
pub const TYPE_MAYBE_BOOL: Type = 0x10;
pub const TYPE_MAYBE_BIGINT: Type = 0x20;
pub const TYPE_MAYBE_OTHER: Type = 0x40;
pub const TYPE_BITS: Type = TYPE_MAYBE_NUMBER
    | TYPE_MAYBE_STRING
    | TYPE_MAYBE_NULL
    | TYPE_MAYBE_BOOL
    | TYPE_MAYBE_BIGINT
    | TYPE_MAYBE_OTHER;

pub const NUM_BITS_NEEDED: u8 = 7;
use const_if::*;
#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Hash, Debug)]
pub struct ResultType {
    bits: Type,
}

impl ResultType {
    pub const fn new(ty: Type) -> Self {
        Self { bits: ty }
    }

    pub const fn is_int32(&self) -> bool {
        self.bits & TYPE_INT32 != 0
    }

    pub const fn definitely_is_number(&self) -> bool {
        (self.bits & TYPE_BITS) == TYPE_MAYBE_NUMBER
    }

    pub const fn definitely_is_string(&self) -> bool {
        (self.bits & TYPE_BITS) == TYPE_MAYBE_STRING
    }

    pub const fn definitely_is_boolean(&self) -> bool {
        (self.bits & TYPE_BITS) == TYPE_MAYBE_BOOL
    }

    pub const fn definitely_is_bigint(&self) -> bool {
        (self.bits & TYPE_BITS) == TYPE_MAYBE_BIGINT
    }
    pub const fn might_be_number(&self) -> bool {
        self.bits & TYPE_MAYBE_NUMBER != 0
    }

    pub const fn might_be_bigint(&self) -> bool {
        self.bits & TYPE_MAYBE_BIGINT != 0
    }

    pub const fn is_not_bigint(&self) -> bool {
        !self.might_be_bigint()
    }

    pub const fn null() -> Self {
        Self::new(TYPE_MAYBE_NULL)
    }

    pub const fn number() -> Self {
        Self::new(TYPE_MAYBE_NUMBER)
    }

    pub const fn number_is_int32() -> Self {
        Self::new(TYPE_INT32 | TYPE_MAYBE_NUMBER)
    }

    pub const fn string_or_number() -> Self {
        Self::new(TYPE_MAYBE_NUMBER | TYPE_MAYBE_STRING)
    }

    pub const fn add_result() -> Self {
        Self::new(TYPE_MAYBE_NUMBER | TYPE_MAYBE_STRING | TYPE_MAYBE_BIGINT)
    }

    pub const fn string() -> Self {
        Self::new(TYPE_MAYBE_STRING)
    }

    pub const fn bigint() -> Self {
        Self::new(TYPE_MAYBE_BIGINT)
    }

    pub const fn unknown() -> Self {
        Self::new(TYPE_BITS)
    }
    pub const fn boolean() -> Self {
        Self::new(TYPE_MAYBE_BOOL)
    }
    pub const fn for_add(op1: Self, op2: Self) -> Self {
        const_if!(
            op1.definitely_is_number() & op2.definitely_is_number() => Self::number();
            elif op1.definitely_is_string() | op2.definitely_is_string() => Self::string();
            elif op1.definitely_is_boolean() & op2.definitely_is_boolean() => Self::boolean();
            elif op1.definitely_is_bigint() & op2.definitely_is_bigint() => Self::bigint();
            else Self::add_result()
        )
    }

    pub const fn for_logical_op(op1: Self, op2: Self) -> Self {
        const_if!(
            op1.definitely_is_boolean() & op2.definitely_is_boolean() => Self::boolean();
            elif op1.definitely_is_number() & op2.definitely_is_number() => Self::number();
            elif op1.definitely_is_string() & op2.definitely_is_string() => Self::string();
            elif op1.definitely_is_bigint() & op2.definitely_is_bigint() => Self::bigint();
            else Self::unknown()
        )
    }

    pub const fn for_bit_op() -> Self {
        Self::number_is_int32()
    }
}

#[derive(Copy, Clone)]
struct Rds {
    first: Type,
    second: Type,
}

#[derive(Copy, Clone)]
union OperandUnion {
    rds: Rds,
    i: i32,
}

pub struct OperandTypes {
    u: OperandUnion,
}

impl OperandTypes {
    #[inline(always)]
    pub fn new(first: ResultType, second: ResultType) -> Self {
        let mut u = OperandUnion { i: 0 };
        u.rds = Rds {
            first: first.bits,
            second: second.bits,
        };
        Self { u }
    }
    #[inline(always)]
    pub fn first(&self) -> ResultType {
        unsafe { ResultType::new(self.u.rds.first) }
    }
    #[inline(always)]
    pub fn second(&self) -> ResultType {
        unsafe { ResultType::new(self.u.rds.second) }
    }
    #[inline(always)]
    pub fn to_int(&self) -> i32 {
        unsafe { self.u.i }
    }
    #[inline(always)]
    pub const fn from_int(i: i32) -> Self {
        Self {
            u: OperandUnion { i: i },
        }
    }
}
