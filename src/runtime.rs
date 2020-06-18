pub mod hash;
use crate::object::*;
use crate::value::*;
use crate::*;
pub fn rt_lookup_property(obj: Value, key: Value, insert: bool) -> isize {
    if !obj.is_cell() {
        return -1;
    }

    let cell = obj.as_cell();
    match cell.type_of() {
        WaffleType::Array => {
            let arr = cell.try_as_array().unwrap();
            let key = key.to_number();
            if key.is_infinite() || key.is_nan() {
                return -1;
            }
            let mut key = key.floor() as isize;
            if key < 0 {
                key = key + arr.value().len() as isize;
            }
            if arr.value().len() as isize > key {
                return key;
            } else {
                return -1;
            }
        }
        WaffleType::Object => {
            unimplemented!();
        }
        _ => -1,
    }
}

pub fn rt_strictcmp(lhs: Value, rhs: Value) -> std::cmp::Ordering {
    if lhs.is_cell() && !rhs.is_cell() {
        return std::cmp::Ordering::Less;
    }
    if !lhs.is_cell() && rhs.is_cell() {
        return std::cmp::Ordering::Less;
    }

    if !lhs.is_cell() && !rhs.is_cell() {
        if lhs.is_number() && rhs.is_number() {
            return lhs
                .to_number()
                .partial_cmp(&rhs.to_number())
                .unwrap_or(std::cmp::Ordering::Less);
        } else {
            return if lhs != rhs {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            };
        }
    } else {
        let lhs = lhs.as_cell();
        let rhs = rhs.as_cell();
        match (lhs.type_of(), rhs.type_of()) {
            (WaffleType::String, WaffleType::String) => lhs
                .try_as_string()
                .unwrap()
                .value()
                .string
                .cmp(&rhs.try_as_string().unwrap().value().string),
            _ => std::cmp::Ordering::Less,
        }
    }
}
