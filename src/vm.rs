use crate::gc::*;

pub struct VirtualMachine {
    heap: Heap,
    stack: Root<Vec<u64>>,
}
