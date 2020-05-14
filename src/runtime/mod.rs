pub mod builtins;
pub mod cell;
pub mod deref_ptr;
pub mod perf;
pub mod pure_nan;
pub mod tld;
pub mod value;
use crate::common::*;
use crate::fullcodegen::FullCodegen;
use crate::heap::api::*;
use crate::heap::heap::Heap;
use crate::interpreter::callstack::CallFrame;
use crate::jit::*;
use cell::*;
use osr::*;
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

use crate::jit::osr::*;
use std::cell::RefCell;
thread_local! {
    static OSR_STUB: RefCell<OSRStub> = RefCell::new(OSRStub::new());
}

pub struct Runtime {
    pub configs: Configs,
    #[cfg(feature = "perf")]
    pub perf: perf::Perf,
    pub heap: Heap,
    pub string_prototype: Rooted<Cell>,
    pub object_prototype: Rooted<Cell>,
    pub array_prototype: Rooted<Cell>,
    pub number_prototype: Rooted<Cell>,
    pub function_prototype: Rooted<Cell>,
    pub generator_prototype: Rooted<Cell>,
    pub process_prototype: Rooted<Cell>,
    pub file_prototype: Rooted<Cell>,
    pub module_prototype: Rooted<Cell>,
    pub boolean_prototype: Rooted<Cell>,
    pub byte_array_prototype: Rooted<Cell>,
    pub globals: Rooted<HashMap<String, Value>>,
    pub stack: Rooted<crate::interpreter::callstack::CallStack>,
    pub code_space: CodeAllocator,
    pub strings: Rooted<HashMap<String, Value>>,
}

impl Runtime {
    pub fn get_osr(
        &mut self,
    ) -> extern "C" fn(&mut Runtime, &mut CallFrame, usize) -> super::jit::JITResult {
        OSR_STUB.with(|x| {
            if x.borrow().code.is_some() {
                return x.borrow().get();
            }
            x.borrow_mut().generate(self);
            x.borrow().get()
        })
    }
    pub fn new(c: Configs) -> Self {
        let mut heap = Heap::new();
        let object = heap.allocate(Cell::new(CellValue::None, None));
        let func = heap.allocate(Cell::new(CellValue::None, Some(object.to_heap())));

        let mut this = Self {
            configs: c,
            #[cfg(feature = "perf")]
            perf: perf::Perf::new(),
            boolean_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            process_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            generator_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            array_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            byte_array_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            globals: heap.allocate(HashMap::new()),
            stack: heap.allocate(crate::interpreter::callstack::CallStack::new(999 * 2)),
            module_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            file_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            string_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            object_prototype: object,
            function_prototype: func,
            strings: heap.allocate(HashMap::new()),
            number_prototype: heap.allocate(Cell::new(CellValue::None, None)),
            heap,
            code_space: CodeAllocator::new(),
        };

        builtins::initialize(&mut this);
        this
    }
    #[inline]
    pub fn allocate_cell(&mut self, cell: Cell) -> Rooted<Cell> {
        self.heap.allocate(cell)
    }
    #[inline]
    /// Make some value rooted.
    pub fn make_rooted<T: Traceable + 'static>(&mut self, value: Handle<T>) -> Rooted<T> {
        self.heap.root(value)
    }
    #[inline]
    pub fn allocate<T: Traceable + 'static>(&mut self, val: T) -> Rooted<T> {
        self.heap.allocate(val)
    }
    pub fn intern(&mut self, str: impl AsRef<str>) -> Value {
        if let Some(x) = self.strings.get().get(str.as_ref()) {
            return *x;
        }
        let val = Value::from(self.allocate_string(str.as_ref()));
        self.strings.insert(str.as_ref().to_string(), val);
        return val;
    }
    pub fn allocate_string(&mut self, string: impl AsRef<str>) -> Rooted<Cell> {
        let s = string.as_ref().to_string();
        let proto = self.string_prototype.to_heap();
        let cell = Cell::new(CellValue::String(Box::new(s)), Some(proto));

        self.allocate_cell(cell)
    }

    pub extern "C" fn call(
        &mut self,
        func: Value,
        this: Value,
        args: &[Value],
    ) -> Result<Value, Value> {
        let ptr = self as *mut Self;
        if func.is_empty() {
            panic!();
        }
        if func.is_cell() == false {
            return Err(Value::from(self.allocate_string("not a function")));
        }
        let val = func;
        use crate::interpreter::Return;

        if let CellValue::Function(ref mut func) = func.as_cell().value {
            match func {
                Function::Native { name: _, native } => match native(self, this, args) {
                    Return::Error(e) => return Err(e),
                    Return::Return(x) => return Ok(x),
                    _ => unimplemented!("TODO: Generators"),
                },
                Function::Regular(ref mut regular) => {
                    let regular: &mut RegularFunction = regular;
                    match regular.kind {
                        RegularFunctionKind::Generator => {
                            unimplemented!("TODO: Instantiat generator");
                        }
                        _ => {
                            if self.configs.enable_jit {
                                if let Some(ref jit) = regular.code.jit_code {
                                    self.stack.push(
                                        unsafe { &mut *ptr },
                                        val,
                                        regular.code,
                                        this,
                                    )?;

                                    let mut cur = self.stack.current_frame();
                                    let func: extern "C" fn(
                                        &mut Runtime,
                                        &mut CallFrame,
                                        usize,
                                    )
                                        -> JITResult =
                                        unsafe { std::mem::transmute(jit.instruction_start()) };
                                    for (i, arg) in args.iter().enumerate() {
                                        if i >= self.stack.current_frame().entries.len() {
                                            break;
                                        }
                                        self.stack.current_frame().entries[i] = *arg;
                                    }
                                    assert!(regular.code.jit_enter == 0);
                                    let _r = self.stack.clone();
                                    let res = func(self, cur.get_mut(), jit.osr_table.labels[0]);
                                    self.stack.pop();
                                    match res {
                                        JITResult::Err(e) => return Err(e),
                                        JITResult::Ok(x) => return Ok(x),
                                        JITResult::OSRExit => {
                                            /*self.stack.current_frame().ip = entry.to_ip;
                                            self.stack.current_frame().bp = entry.to_bp;
                                            match self.interpret() {
                                                Return::Return(val) => return Ok(val),
                                                Return::Error(e) => return Err(e),
                                                Return::Yield { .. } => {
                                                    unimplemented!("TODO: Generators")
                                                }
                                            }*/
                                            unimplemented!();
                                        }
                                    }
                                } else {
                                    if regular.code.hotness >= 1000 {
                                        if let RegularFunctionKind::Ordinal = regular.kind {
                                            let mut gen = FullCodegen::new(regular.code);
                                            gen.compile(false);
                                            log::trace!(
                                                "Disassembly for '{}'",
                                                unwrap!(regular.name.to_string(self))
                                            );
                                            let code = gen.finish(self, true);
                                            let func: extern "C" fn(
                                                &mut Runtime,
                                                &mut CallFrame,
                                                usize,
                                            )
                                                -> JITResult = unsafe {
                                                std::mem::transmute(code.instruction_start())
                                            };
                                            let x = unsafe { &mut *ptr };
                                            let _ = self.stack.push(x, val, regular.code, this);
                                            for (i, arg) in args.iter().enumerate() {
                                                if i >= self.stack.current_frame().entries.len() {
                                                    break;
                                                }
                                                self.stack.current_frame().entries[i] = *arg;
                                            }
                                            let _r = self.stack.clone();
                                            let mut cur = self.stack.current_frame();
                                            drop(_r);
                                            match func(
                                                self,
                                                cur.get_mut(),
                                                code.osr_table.labels[0],
                                            ) {
                                                JITResult::Ok(val) => {
                                                    assert!(
                                                        self.stack.get() as *const _ as *const u8
                                                            as usize
                                                            != 0x19
                                                    );
                                                    self.stack.pop();
                                                    regular.code.jit_code = Some(code);
                                                    return Ok(val);
                                                }
                                                JITResult::Err(e) => {
                                                    self.stack.pop();
                                                    regular.code.jit_code = Some(code);
                                                    return Err(e);
                                                }
                                                _ => unimplemented!(),
                                            }
                                        }
                                    } else {
                                        regular.code.get_mut().hotness =
                                            regular.code.hotness.wrapping_add(50);
                                    }
                                }
                            }
                            // unsafe code block is actually safe,we just access heap.
                            self.stack
                                .push(unsafe { &mut *ptr }, val, regular.code, this)?;
                            for (i, arg) in args.iter().enumerate() {
                                if i >= self.stack.current_frame().entries.len() {
                                    break;
                                }
                                self.stack.current_frame().entries[i] = *arg;
                            }
                            self.stack.current_frame().exit_on_return = true;
                            match self.interpret() {
                                Return::Return(val) => return Ok(val),
                                Return::Error(e) => return Err(e),
                                Return::Yield { .. } => unimplemented!("TODO: Generators"),
                            }
                        }
                    }
                }
                _ => unimplemented!("TODO: Async"),
            }
        }
        let key = self.allocate_string("call");
        if let Some(call) = func.lookup(self, Value::from(key.to_heap()))? {
            return self.call(call, this, args);
        }
        return Err(Value::from(self.allocate_string("not a function")));
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
