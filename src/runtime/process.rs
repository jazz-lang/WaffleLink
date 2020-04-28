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
    pub array_proto: Value,
    pub function_proto: Value,
    pub number_proto: Value,
    pub boolean_proto: Value,
}

impl LocalData {
    pub fn new() -> Self {
        let mut heap = Heap::new();
        let obj = heap.allocate_cell(Cell::new(None));
        let str = heap.allocate_cell(Cell::new(Some(obj)));
        Self {
            frames: vec![],
            process: Value::empty(), // initialized later.
            heap: Heap::new(),
            string_proto: Value::from(str),
            object_proto: Value::from(obj),
            function_proto: Value::new_int(0),
            number_proto: Value::new_int(0),
            boolean_proto: Value::new_int(0),
            array_proto: Value::new_int(0),
            globals: HashMap::new(),
        }
    }
}

impl LocalData {
    pub fn stacktrace<W: std::fmt::Write>(&self, buffer: &mut W) -> std::fmt::Result {
        for (i, frame) in self.frames.iter().enumerate() {
            writeln!(
                buffer,
                "{}: {}",
                i,
                frame.func.func_value_unchecked().name.to_string()
            )?;
        }
        Ok(())
    }
    pub fn allocate_cell(&mut self, cell: Cell) -> Value {
        Value::from(self.heap.allocate_cell(cell))
    }
    pub fn allocate(&mut self, cell: Cell, frame: &mut Frame) -> Value {
        Value::from(self.heap.allocate(frame, cell))
    }
    pub fn allocate_string(&mut self, s: impl AsRef<str>, f: &mut Frame) -> Value {
        let cell = Cell {
            prototype: Some(self.string_proto.as_cell()),
            color: CELL_WHITE_A,
            value: CellValue::String(Box::new(s.as_ref().to_string())),
            slots: TaggedPointer::null(),
            attributes: Arc::new(super::map::Map::new()),
            map: Arc::new(super::structure::Map::new_unique(Ptr::null(), false)),
        };

        Value::from(self.heap.allocate(f, cell))
    }

    pub fn allocate_array(&mut self, values: Vec<Value>, f: &mut Frame) -> Value {
        //let length = Value::new_int(values.len() as _);
        let cell = Cell {
            prototype: Some(self.object_proto.as_cell()),
            color: CELL_WHITE_A,
            value: CellValue::Array(Box::new(values)),
            slots: TaggedPointer::null(),
            attributes: Arc::new(super::map::Map::new()),
            map: Arc::new(super::structure::Map::new_unique(Ptr::null(), false)),
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
