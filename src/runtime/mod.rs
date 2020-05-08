pub mod cell;
pub mod deref_ptr;

pub mod pure_nan;
pub mod value;
use crate::jit::*;
use cell::*;
use cgc::api::*;
use cgc::heap::Heap;
use osr::*;
use std::collections::HashMap;
use value::*;
pub struct Runtime {
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
    pub globals: HashMap<String, Value>,
    pub stack: crate::interpreter::callstack::CallStack,
}

impl Runtime {
    pub fn new() -> Self {
        let mut heap = Heap::new(32 * 1024, 64 * 1024, true);
        let object = heap.allocate(Cell::new(CellValue::None, None));
        let func = heap.allocate(Cell::new(CellValue::None, Some(object.to_heap())));

        Self {
            boolean_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            process_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            generator_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            array_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            byte_array_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            globals: HashMap::new(),
            stack: crate::interpreter::callstack::CallStack::new(999),
            module_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            file_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            string_prototype: heap.allocate(Cell::new(CellValue::None, Some(object.to_heap()))),
            object_prototype: object,
            function_prototype: func,
            number_prototype: heap.allocate(Cell::new(CellValue::None, None)),
            heap,
        }
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
                            if let Some(jit) = regular.code.jit_stub {
                                self.stack
                                    .push(unsafe { &mut *ptr }, val, regular.code, this)?;
                                // This variable is mutable because JIT might exit to interpreter and set bp/ip of instruction
                                // that will start interpreting.
                                let mut entry = OSREntry {
                                    to_bp: 0,
                                    to_ip: jit as usize, // if `to_ip` points to current func then skip all jump tables and start execution.
                                };
                                match jit(&mut entry, self, this, args) {
                                    JITResult::Err(e) => return Err(e),
                                    JITResult::Ok(x) => return Ok(x),
                                    JITResult::OSRExit => {
                                        self.stack.current_frame().ip = entry.to_ip;
                                        self.stack.current_frame().bp = entry.to_bp;
                                        match self.interpret() {
                                            Return::Return(val) => return Ok(val),
                                            Return::Error(e) => return Err(e),
                                            Return::Yield { .. } => {
                                                unimplemented!("TODO: Generators")
                                            }
                                        }
                                    }
                                }
                            } else {
                                regular.code.get_mut().hotness =
                                    regular.code.hotness.wrapping_add(1);
                                if regular.code.hotness >= 10000 {
                                    // TODO: FullCodegen
                                }
                                // unsafe code block is actually safe,we just access heap.
                                self.stack
                                    .push(unsafe { &mut *ptr }, val, regular.code, this)?;
                                match self.interpret() {
                                    Return::Return(val) => return Ok(val),
                                    Return::Error(e) => return Err(e),
                                    Return::Yield { .. } => unimplemented!("TODO: Generators"),
                                }
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
