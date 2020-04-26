use super::cell::*;
use super::frame::*;
use super::symbol::Symbol;
use super::value::Value;
use crate::arc::ArcWithoutWeak as Arc;
use crate::common::ptr::*;
use crate::heap::Heap;
use may::coroutine::JoinHandle;
use may::coroutine_local;
use may::sync::Mutex;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::collections::VecDeque;
pub struct LocalData {
    pub frames: Vec<Frame>,
    pub process: Value,
    pub heap: Heap,
    pub globals: HashMap<Symbol, Value>,
    pub string_proto: Value,
    pub object_proto: Value,
    pub function_proto: Value,
    pub number_proto: Value,
    pub boolean_proto: Value,
}

impl LocalData {
    pub fn new() -> Self {
        Self {
            frames: vec![],
            process: Value::empty(), // initialized later.
            heap: Heap::new(),
            string_proto: Value::new_int(0),
            object_proto: Value::new_int(0),
            function_proto: Value::new_int(0),
            number_proto: Value::new_int(0),
            boolean_proto: Value::new_int(0),
            globals: HashMap::new(),
        }
    }
}

impl LocalData {
    pub fn allocate_string(&mut self, s: impl AsRef<str>, f: &mut Frame) -> Value {
        let cell = Cell {
            prototype: Some(self.string_proto.as_cell()),
            attributes: TaggedPointer::null(),
            color: CELL_WHITE_A,
            value: CellValue::String(Box::new(s.as_ref().to_string())),
            slots: TaggedPointer::null(),
        };

        Value::from(self.heap.allocate(f, cell))
    }
}

pub struct Process {
    pub handle: JoinHandle<()>,
    pub mailbox: Mutex<VecDeque<Value>>,
}

coroutine_local!(static LOCAL_DATA: UnsafeCell<LocalData> = UnsafeCell::new(LocalData::new()));

pub fn local_data<'a>() -> &'a mut LocalData {
    unsafe { &mut *LOCAL_DATA.with(|x| x.get()) }
}
