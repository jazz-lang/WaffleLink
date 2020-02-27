#include "value.h"
#include <math.h>
#define slow_binop(s) extern Value value_slow_##s (Value x,Value y);

slow_binop (add)
slow_binop (sub)
slow_binop (mul)
slow_binop (div)
slow_binop (rsh)
slow_binop (lsh)
slow_binop (mod)
slow_binop (gt)
slow_binop (lt)
slow_binop (lte)
slow_binop (gte)
slow_binop (eq)
slow_binop (neq)

/// This function is defined in Rust code.
extern double value_to_double_slow(Value self);

double value_to_double(Value self) {
  if (value_is_int32 (self)) {
    return value_as_int32(self);
  } else if (value_is_double (self)) {
    return value_as_double (self);
  } else {
    if (value_is_bool (self)) {
      if (value_is_true(self)) {
        return 1.0;
      } else {
        return 0.0;
      }
    } else if (value_is_null_or_undefined (self)) {
      return 1.0 * 0.0;
    } else {
      return value_to_double_slow (self);
    }

  }
}

Value value_add(Value x,Value y) {
  if (value_is_number (x) && value_is_number (y)) {
    return value_double(value_to_double(x) + value_to_double (y));
  } else {
    return value_slow_add(x,y);
  }
}

Value value_sub(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y)) {
    return value_double(value_to_double (x) - value_to_double(y));
  } else {
    return value_slow_sub(x,y);
  }
}
Value value_mul(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y)) {
    return value_double(value_to_double (x) * value_to_double(y));
  } else {
    return value_slow_mul(x,y);
  }
}
Value value_div(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y)) {
    return value_double(value_to_double (x) / value_to_double(y));
  } else {
    return value_slow_div(x,y);
  }
}
Value value_mod(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y)) {
    return value_double(fmod(value_to_double (x), value_to_double(y)));
  } else {
    return value_slow_mod(x,y);
  }
}

Value value_lsh(Value x,Value y) {
  return value_slow_lsh(x,y);
}

Value value_rsh(Value x,Value y) {
  return value_slow_rsh(x,y);
}

Value value_eq(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y))
      return value_from_tag ( value_to_double(x) == value_to_double (y) ? VALUE_TRUE : VALUE_FALSE);
  else if (value_is_bool (x) && value_is_bool (y))
      return value_from_tag( value_to_double(x) == value_to_double(y) ? VALUE_TRUE : VALUE_FALSE );
  else
      return value_slow_eq(x,y);
}

Value value_gt(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y))
      return value_from_tag ( value_to_double(x) > value_to_double (y) ? VALUE_TRUE : VALUE_FALSE );
  else
      return value_slow_gt(x,y);
}

Value value_lt(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y))
      return value_from_tag(value_to_double(x) < value_to_double(y) ? VALUE_TRUE : VALUE_FALSE);
  else
      return value_slow_lt(x,y);
}

Value value_lte(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y))
      return value_from_tag(value_to_double(x) <= value_to_double(y) ? VALUE_TRUE : VALUE_FALSE);
  else
      return value_slow_lte(x,y);
}
Value value_gte(Value x,Value y) {
  if (value_is_number(x) && value_is_number(y))
      return value_from_tag(value_to_double(x) >= value_to_double(y) ? VALUE_TRUE : VALUE_FALSE);
  else
      return value_slow_gte(x,y);
}
