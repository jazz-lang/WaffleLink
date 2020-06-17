use crate::value::Value;
use ahash::AHasher;
pub extern "C" fn waffle_get_hash_of(value: Value) -> isize {
    if value.is_undefined_or_null() {
        return 0;
    } else if value.is_cell() {
        return value.as_cell().raw() as isize;
    } else if value.is_boolean() {
        if value.is_true() {
            return 1;
        } else {
            return 0;
        }
    } else if value.is_number() {
        return 0;
    } else {return 0}
}

pub extern "C" fn waffle_compute_hash(key: i64) -> u32 {
    let mut hash = 0;
    hash += (key >> 32) as u32;
    hash += hash << 10;
    hash ^= hash >> 6;

    hash += (key & 0xffffffff) as u32;
    hash += hash << 10;
    hash ^= hash >> 6;

    hash += hash << 3;
    hash ^= hash >> 1;
    hash += hash << 15;
    hash
}

pub extern "C" fn waffle_compute_hash_of_bytes(bytes: *const u8,length: u32) -> u32 {
    let mut hash = 0;
    for i in 0..length {
        let i = i as isize;
        let key = unsafe {*bytes.offset(i)};
        hash += key as u32;
        hash += hash << 10;
        hash ^= hash >> 6;
    }
    hash += hash << 3;
    hash ^= hash >> 11;
    hash += hash << 15;
    hash
}
