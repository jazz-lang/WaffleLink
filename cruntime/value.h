#pragma once
#include <stdbool.h>
#include <inttypes.h>
#include <stddef.h>
typedef struct CellPointer
{
} CellPointer;

typedef union {
    int64_t as_int64;
    CellPointer ptr;
} Value;


#define DOUBLE_ENCODE_OFFSET_BIT 49
#define DOUBLE_ENCODE_OFFSET ((int64_t)1 << (int64_t)49)
#define NUMBER_TAG 0xfffe000000000000ll
#define BOOL_TAG 0x4
#define UNDEFINED_TAG 0x8
#define OTHER_TAG 0x2
#define VALUE_FALSE (OTHER_TAG | BOOL_TAG | false)
#define VALUE_TRUE (OTHER_TAG | BOOL_TAG | true)
#define VALUE_UNDEFINED (OTHER_TAG | UNDEFINED_TAG)
#define VALUE_NULL (OTHER_TAG)
#define MISC_TAG (OTHER_TAG | BOOL_TAG | UNDEFINED_TAG)
#define NOT_CELL_MASK (NUMBER_TAG | OTHER_TAG)
#define VALUE_EMPTY 0x0
#define VALUE_DELETED 0x4

#define value_empty() \
    (Value) { .as_int64 = VALUE_EMPTY }

typedef union {
    uint64_t bits : sizeof(double);
    double value;
    uint8_t bytes[sizeof(double)];
} f64_to_bits;

#define value_is_int32(x) ((x.as_int64 & NUMBER_TAG) == NUMBER_TAG)
#define value_is_double(x) ((!value_is_int32(x)) && value_is_number(x))
#define value_is_number(x) (x.as_int64 & NUMBER_TAG) != 0

#define value_double(x) \
    (Value) { .as_int64 = (f64_to_bits){.value = x}.bits }
#define value_int(x) \
    (Value) { .as_int64 = NUMBER_TAG | (int64_t)((uint32_t)x) }

#define value_is_empty(x) x.as_int64 == VALUE_EMPTY
#define value_from_tag(x) \
    ((Value) { .as_int64 = x })
#define value_is_undefined(x) x.as_int64 == value_from_tag(VALUE_UNDEFINED).as_int64
#define value_is_null(x) (x.as_int64 == value_from_tag(VALUE_NULL).as_int64)
#define value_is_true(x) (x.as_int64 == value_from_tag(VALUE_TRUE).as_int64)
#define value_is_false(x) (x.as_int64 == value_from_tag(VALUE_FALSE).as_int64)
#define value_as_bool(x) !value_is_false(x)
#define value_is_bool(x) ((x.as_int64 & !1) == VALUE_FALSE)
#define value_is_null_or_undefined(x) (x.as_int64 & !UNDEFINED_TAG) == VALUE_NULL
#define value_is_cell(x) x.as_int64 &NOT_CELL_MASK
#define value_binop(s) Value value_##s(Value x,Value y);
#define value_as_int32(x) ((int32_t)x.as_int64)

value_binop (add)
value_binop (sub)
value_binop (mul)
value_binop (div)
value_binop (rsh)
value_binop (lsh)
value_binop (mod)
value_binop (gt)
value_binop (lt)
value_binop (lte)
value_binop (gte)
value_binop (eq)
value_binop (neq)

#define value_unop(s) Value value_##s(Value);

value_unop (not)
value_unop (neg)

#define value_as_double(x) ((f64_to_bits){.bits = x.as_int64 - DOUBLE_ENCODE_OFFSET}).value

double value_to_number(Value self);
