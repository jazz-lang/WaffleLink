use crate::value::Value;
use murmur3::murmur3_32;
use std::hash::{Hash, Hasher};
pub extern "C" fn waffle_get_hash_of(value: Value) -> u32 {
    let mut hasher = WaffleHash::default();
    value.hash(&mut hasher);
    hasher.finish() as u32
}

#[derive(Default)]
pub struct WaffleHash(u32);

macro_rules! write_impl_for {
    ($($t: ty : $name: ident)*) => {
        $(
        fn $name(&mut self, key: $t) {
            self.0 = waffle_compute_hash(self.0, key as _);
        }
    )*
    };
}

impl Hasher for WaffleHash {
    write_impl_for!(
        u8: write_u8
        u16: write_u16
        u32: write_u32
        u64: write_u64
        i8: write_i8
        i16: write_i16
        i32: write_i32
        i64: write_i64
    );
    fn write(&mut self, bytes: &[u8]) {
        self.0 = waffle_compute_hash_of_bytes(self.0, bytes.as_ptr(), bytes.len() as _);
    }
    fn write_usize(&mut self, i: usize) {
        self.0 = waffle_compute_hash(self.0, i as i64);
    }

    fn write_isize(&mut self, i: isize) {
        self.0 = waffle_compute_hash(self.0, i as _);
    }

    fn write_i128(&mut self, i: i128) {
        let bytes: [u8; 16] = unsafe { std::mem::transmute(i) };
        self.write(&bytes);
    }

    fn write_u128(&mut self, i: u128) {
        let bytes: [u8; 16] = unsafe { std::mem::transmute(i) };
        self.write(&bytes);
    }
    fn finish(&self) -> u64 {
        self.0 as u64
    }
}

pub extern "C" fn waffle_compute_hash(hash: u32, key: i64) -> u32 {
    let bytes = key.to_le_bytes();
    murmur3_32(&mut std::io::Cursor::new(bytes), hash).unwrap()
}

pub extern "C" fn waffle_compute_hash_of_bytes(hash: u32, bytes: *const u8, length: u32) -> u32 {
    murmur3_32(
        &mut std::io::Cursor::new(unsafe { std::slice::from_raw_parts(bytes, length as _) }),
        hash,
    )
    .unwrap()
}
