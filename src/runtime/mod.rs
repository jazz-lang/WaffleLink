pub mod cell;
pub mod deref_ptr;

pub mod pure_nan;
pub mod value;
use cell::*;
use cgc::api::*;
use cgc::heap::Heap;
use std::collections::HashMap;
use value::*;

pub struct Runtime {
    pub heap: Heap,
    pub string_prototype: Value,
    pub object_prototype: Value,
    pub array_prototype: Value,
    pub number_prototype: Value,
    pub function_prototype: Value,
    pub generator_prototype: Value,
    pub process_prototype: Value,
    pub file_prototype: Value,
    pub module_prototype: Value,
    pub boolean_prototype: Value,
    pub byte_array_prototype: Value,
    pub globals: HashMap<String, Value>,
    pub stack: crate::interpreter::callstack::CallStack,
}

impl Runtime {
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
        let proto = self.string_prototype.as_cell();
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
                                return jit(self, this, args);
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
        return Err(Value::from(self.allocate_string("not a function")));
    }
}
