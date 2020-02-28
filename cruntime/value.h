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

#pragma once
#include <stdbool.h>
#include <inttypes.h>
#include <stddef.h>

typedef struct ReturnValue ReturnValue;
typedef ReturnValue *ReturnValuePtr;
typedef void *ResultPtr;

typedef union {
    int64_t as_int64;
    void *ptr;
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
    ((Value){.as_int64 = x})
#define value_is_undefined(x) x.as_int64 == value_from_tag(VALUE_UNDEFINED).as_int64
#define value_is_null(x) (x.as_int64 == value_from_tag(VALUE_NULL).as_int64)
#define value_is_true(x) (x.as_int64 == value_from_tag(VALUE_TRUE).as_int64)
#define value_is_false(x) (x.as_int64 == value_from_tag(VALUE_FALSE).as_int64)
#define value_as_bool(x) !value_is_false(x)
#define value_is_bool(x) ((x.as_int64 & !1) == VALUE_FALSE)
#define value_is_null_or_undefined(x) (x.as_int64 & !UNDEFINED_TAG) == VALUE_NULL
#define value_is_cell(x) x.as_int64 &NOT_CELL_MASK
#define value_binop(s) Value value_##s(Value x, Value y);
#define value_as_int32(x) ((int32_t)x.as_int64)

value_binop(add);
value_binop(sub);
value_binop(mul);
value_binop(div);
value_binop(rsh);
value_binop(lsh);
value_binop(mod);
value_binop(gt);
value_binop(lt);
value_binop(lte);
value_binop(gte);
value_binop(eq);
value_binop(neq);

#define value_unop(s) Value value_##s(Value);

value_unop(not);
value_unop(neg);

#define value_as_double(x) ((f64_to_bits){.bits = x.as_int64 - DOUBLE_ENCODE_OFFSET}).value

double value_to_number(Value self);

void cell_add_attribute_wo_barrier(const void *cell, Value key, Value value);

Value cell_lookup_attribute(const void *cell, Value key);

void cell_set_prototype(const void *cell, const void *prototype);
void cell_add_attribute(const void *proc, const void *cell, Value key, Value value);
void store_by_id_impl(const void *proc, Value object, Value value, Value id);
Value load_by_id_impl(const void *proc, Value object, Value id);
Value load_by_value_impl(const void *proc, Value object, Value field);
void store_by_value_impl(const void *proc, Value object, Value value, Value field);
ReturnValuePtr create_ret(Value x);
typedef void *Stack;

Value stack_pop(void *);
void stack_push(void *, Value *);
/***** DEFINITIONS *****/

#define VECTOR_MINIMUM_CAPACITY 2
#define VECTOR_GROWTH_FACTOR 2
#define VECTOR_SHRINK_THRESHOLD (1 / 4)

#define VECTOR_ERROR -1
#define VECTOR_SUCCESS 0

#define VECTOR_UNINITIALIZED NULL
#define VECTOR_INITIALIZER            \
    {                                 \
        0, 0, 0, VECTOR_UNINITIALIZED \
    }

/***** STRUCTURES *****/

typedef struct Vector
{
    size_t size;
    size_t capacity;
    size_t element_size;

    void *data;
} Vector;

typedef struct Iterator
{
    void *pointer;
    size_t element_size;
} Iterator;

/***** METHODS *****/

/* Constructor */
int vector_setup(Vector *vector, size_t capacity, size_t element_size);

/* Copy Constructor */
int vector_copy(Vector *destination, Vector *source);

/* Copy Assignment */
int vector_copy_assign(Vector *destination, Vector *source);

/* Move Constructor */
int vector_move(Vector *destination, Vector *source);

/* Move Assignment */
int vector_move_assign(Vector *destination, Vector *source);

int vector_swap(Vector *destination, Vector *source);

/* Destructor */
int vector_destroy(Vector *vector);

/* Insertion */
int vector_push_back(Vector *vector, void *element);
int vector_push_front(Vector *vector, void *element);
int vector_insert(Vector *vector, size_t index, void *element);
int vector_assign(Vector *vector, size_t index, void *element);

/* Deletion */
int vector_pop_back(Vector *vector);
int vector_pop_front(Vector *vector);
int vector_erase(Vector *vector, size_t index);
int vector_clear(Vector *vector);

/* Lookup */
void *vector_get(Vector *vector, size_t index);
const void *vector_const_get(const Vector *vector, size_t index);
void *vector_front(Vector *vector);
void *vector_back(Vector *vector);
#define VECTOR_GET_AS(type, vector_pointer, index) \
    *((type *)vector_get((vector_pointer), (index)))

/* Information */
bool vector_is_initialized(const Vector *vector);
size_t vector_byte_size(const Vector *vector);
size_t vector_free_space(const Vector *vector);
bool vector_is_empty(const Vector *vector);

/* Memory management */
int vector_resize(Vector *vector, size_t new_size);
int vector_reserve(Vector *vector, size_t minimum_capacity);
int vector_shrink_to_fit(Vector *vector);

/* Iterators */
Iterator vector_begin(Vector *vector);
Iterator vector_end(Vector *vector);
Iterator vector_iterator(Vector *vector, size_t index);

void *iterator_get(Iterator *iterator);
#define ITERATOR_GET_AS(type, iterator) *((type *)iterator_get((iterator)))

int iterator_erase(Vector *vector, Iterator *iterator);

void iterator_increment(Iterator *iterator);
void iterator_decrement(Iterator *iterator);

void *iterator_next(Iterator *iterator);
void *iterator_previous(Iterator *iterator);

bool iterator_equals(Iterator *first, Iterator *second);
bool iterator_is_before(Iterator *first, Iterator *second);
bool iterator_is_after(Iterator *first, Iterator *second);

size_t iterator_index(Vector *vector, Iterator *iterator);

#define VECTOR_FOR_EACH(vector_pointer, iterator_name)             \
    for (Iterator(iterator_name) = vector_begin((vector_pointer)), \
        end = vector_end((vector_pointer));                        \
         !iterator_equals(&(iterator_name), &end);                 \
         iterator_increment(&(iterator_name)))

/***** PRIVATE *****/

#define MAX(a, b) ((a) > (b) ? (a) : (b))

bool _vector_should_grow(Vector *vector);
bool _vector_should_shrink(Vector *vector);

size_t _vector_free_bytes(const Vector *vector);
void *_vector_offset(Vector *vector, size_t index);
const void *_vector_const_offset(const Vector *vector, size_t index);

void _vector_assign(Vector *vector, size_t index, void *element);

int _vector_move_right(Vector *vector, size_t index);
void _vector_move_left(Vector *vector, size_t index);

int _vector_adjust_capacity(Vector *vector);
int _vector_reallocate(Vector *vector, size_t new_capacity);

void _vector_swap(size_t *first, size_t *second);