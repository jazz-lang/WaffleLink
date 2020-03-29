#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
#[repr(u8)]
pub enum CellType {
    Cell,
    String,
    Symbol,
    BigInt,

    Object,
    Function,
    InternalFunction,
    Number,
    Array,
    Int8Array,
    UInt8Array,
    Int16Array,
    UInt16Array,
    Int32Array,
    UInt32Array,
    Float32Array,
    Float64Array,
    RegExp,

    Map,
    Set,
}
