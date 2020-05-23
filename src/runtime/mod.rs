pub mod cell;
pub mod deref_ptr;
pub mod perf;
pub mod pure_nan;
pub mod tld;
pub mod value;
use crate::common::*;
use crate::heap::api::*;
use crate::heap::heap::{Collection, Heap};
use cell::*;
use std::collections::HashMap;
use std::collections::{HashSet, VecDeque};
use value::*;

pub struct CodeAllocator {
    allocated: HashSet<Address>,
    free: VecDeque<(Address, usize)>,
    free_bytes: usize,
}
pub const K: usize = 1024;
pub const M: usize = K * K;
impl CodeAllocator {
    pub fn new() -> Self {
        Self {
            allocated: Default::default(),
            free: Default::default(),
            free_bytes: 0,
        }
    }
    pub fn alloc(&mut self, size: usize) -> (Address, usize) {
        if let Some((mem, s)) = self.free.iter().find(|(_mem, size_)| *size_ >= size) {
            os::commit_at(*mem, *s, true);
            return (*mem, *s);
        }
        let mem = os::reserve(mem::page_align(size));
        os::commit_at(mem, mem::page_align(size), true);
        self.allocated.insert(mem);
        (mem, mem::page_align(size))
    }

    pub fn free(&mut self, mem: Address, size: usize) {
        self.free_bytes += mem::page_align(size);
        self.free.push_back((mem, mem::page_align(size)));
        os::uncommit(mem, mem::page_align(size));
        if self.free_bytes >= 512 * K {
            while let Some((ptr, size)) = self.free.pop_front() {
                os::free(ptr, size);
                self.free_bytes -= size;
            }
        }
    }
}

use smallvec::SmallVec;
use std::cell::RefCell;

pub struct Runtime {
    pub configs: Configs,
    #[cfg(feature = "perf")]
    pub perf: perf::Perf,
    pub heap: Heap,
    pub string_prototype: CellPointer,
    pub object_prototype: CellPointer,
    pub array_prototype: CellPointer,
    pub number_prototype: CellPointer,
    pub function_prototype: CellPointer,
    pub generator_prototype: CellPointer,
    pub process_prototype: CellPointer,
    pub file_prototype: CellPointer,
    pub module_prototype: CellPointer,
    pub boolean_prototype: CellPointer,
    pub byte_array_prototype: CellPointer,
    pub globals: HashMap<String, Value>,
    pub code_space: CodeAllocator,
    pub strings: HashMap<String, Value>,
}

impl Runtime {
    pub fn new(c: Configs) -> Self {
        let mut heap = Heap::new();
        let object = heap.allocate(Cell::new(CellValue::None, None));
        let func = heap.allocate(Cell::new(CellValue::None, Some(object)));

        let mut this = Self {
            configs: c,
            #[cfg(feature = "perf")]
            perf: perf::Perf::new(),
            boolean_prototype: heap.allocate(Cell::new(CellValue::None, Some(object))),
            process_prototype: heap.allocate(Cell::new(CellValue::None, Some(object))),
            generator_prototype: heap.allocate(Cell::new(CellValue::None, Some(object))),
            array_prototype: heap.allocate(Cell::new(CellValue::None, Some(object))),
            byte_array_prototype: heap.allocate(Cell::new(CellValue::None, Some(object))),
            globals: (HashMap::new()),
            module_prototype: heap.allocate(Cell::new(CellValue::None, Some(object))),
            file_prototype: heap.allocate(Cell::new(CellValue::None, Some(object))),
            string_prototype: heap.allocate(Cell::new(CellValue::None, Some(object))),
            object_prototype: object,
            function_prototype: func,
            strings: HashMap::new(),
            number_prototype: heap.allocate(Cell::new(CellValue::None, None)),
            heap,
            code_space: CodeAllocator::new(),
        };

        this
    }
    pub fn safepoint(&mut self) {
        if self.heap.safepoint() {
            log::debug!(
                "{} bytes allocated when threshold is {}, running GC cycle.",
                self.heap.allocated(),
                self.heap.threshold()
            );
            Collection::run(self);
        }
    }

    pub fn each_pointer(&mut self, stack: &mut std::collections::VecDeque<*const CellPointer>) {
        stack.push_back(&self.array_prototype);
        stack.push_back(&self.object_prototype);
        stack.push_back(&self.number_prototype);
        stack.push_back(&self.boolean_prototype);
        stack.push_back(&self.generator_prototype);
        stack.push_back(&self.boolean_prototype);
        stack.push_back(&self.byte_array_prototype);
        stack.push_back(&self.function_prototype);
        stack.push_back(&self.file_prototype);
        stack.push_back(&self.string_prototype);
        stack.push_back(&self.module_prototype);
        for (_, val) in self.globals.iter() {
            val.each_pointer(stack);
        }
        for (_, val) in self.strings.iter() {
            val.each_pointer(stack);
        }

        /*for frame in self.stack.stack.iter() {
            match frame {
                StackEntry::Frame(ref f) => {
                    f.registers.iter().for_each(|item| item.each_pointer(stack));
                    f.entries.iter().for_each(|item| item.each_pointer(stack));
                    f.func.each_pointer(stack);
                    f.code
                        .constants_
                        .iter()
                        .for_each(|item| item.each_pointer(stack));
                    f.this.each_pointer(stack);
                }
                _ => (),
            }
        }*/
    }

    #[inline]
    pub fn allocate_cell(&mut self, cell: Cell) -> CellPointer {
        self.heap.allocate(cell)
    }

    #[inline]
    pub fn allocate(&mut self, cell: Cell) -> CellPointer {
        self.heap.allocate(cell)
    }
    pub fn intern(&mut self, str: impl AsRef<str>) -> Value {
        if let Some(x) = self.strings.get(str.as_ref()) {
            return *x;
        }
        let val = Value::from(self.allocate_string(str.as_ref()));
        self.strings.insert(str.as_ref().to_string(), val);
        return val;
    }
    pub fn allocate_string(&mut self, string: impl AsRef<str>) -> CellPointer {
        let s = string.as_ref().to_string();
        let proto = self.string_prototype;
        let cell = Cell::new(CellValue::String(Box::new(s)), Some(proto));

        self.allocate_cell(cell)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trap {
    DIV0,
    ASSERT,
    INDEX_OUT_OF_BOUNDS,
    NIL,
    CAST,
    OOM,
    STACK_OVERFLOW,
}

impl Trap {
    pub fn int(self) -> u32 {
        match self {
            Trap::DIV0 => 1,
            Trap::ASSERT => 2,
            Trap::INDEX_OUT_OF_BOUNDS => 3,
            Trap::NIL => 4,
            Trap::CAST => 5,
            Trap::OOM => 6,
            Trap::STACK_OVERFLOW => 7,
        }
    }

    pub fn from(value: u32) -> Option<Trap> {
        match value {
            1 => Some(Trap::DIV0),
            2 => Some(Trap::ASSERT),
            3 => Some(Trap::INDEX_OUT_OF_BOUNDS),
            4 => Some(Trap::NIL),
            5 => Some(Trap::CAST),
            6 => Some(Trap::OOM),
            7 => Some(Trap::STACK_OVERFLOW),
            _ => None,
        }
    }
}

pub struct Configs {
    pub enable_osr: bool,
    pub enable_jit: bool,
}

impl Configs {
    pub fn no_jit(mut self) -> Self {
        self.enable_osr = false;
        self.enable_jit = false;
        self
    }
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            enable_jit: true,
            enable_osr: true,
        }
    }
}
