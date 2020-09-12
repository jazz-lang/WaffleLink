use crate::gc::*;
use object::*;
pub struct VirtualMachine {
    heap: TGC,
    stack: Root<Vec<u64>>,
}
