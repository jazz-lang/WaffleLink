use super::function::*;
use super::map::*;
use super::process::*;
use super::structure::Map as Structure;
use super::symbol::*;
use super::value::*;
use crate::arc::ArcWithoutWeak as Arc;
use crate::common::ptr::*;
use std::collections::HashMap;
pub enum CellValue {
    None,
    Array(Box<Vec<Value>>),
    String(Box<String>),
    Function(Box<Function>),
}

pub const MIN_OLD_SPACE_GENERATION: u8 = 5;

macro_rules! push_collection {
    ($map:expr, $what:ident, $vec:expr) => {{
        $vec.reserve($map.len());

        for thing in $map.$what() {
            $vec.push(thing.clone());
        }
    }};
}

pub const CELL_WHITE_A: u8 = 1;
pub const CELL_WHITE_B: u8 = 1 << 1;
pub const CELL_GREY: u8 = 0;
pub const CELL_BLACK: u8 = 1 << 2;
pub const CELL_WHITES: u8 = CELL_WHITE_A | CELL_WHITE_B;

pub type AttributesMap = Map;

pub struct Cell {
    pub value: CellValue,
    pub(crate) color: u8,
    pub(crate) prototype: Option<Ptr<Cell>>,
    pub(crate) slots: TaggedPointer<Vec<Value>>,
    pub(crate) map: Arc<Structure>,
    pub(crate) attributes: TaggedPointer<Map>,
}

impl Cell {
    pub fn new(proto: Option<Ptr<Cell>>) -> Self {
        Self {
            value: CellValue::None,
            color: 0,
            prototype: proto,
            slots: TaggedPointer::null(),
            map: Arc::new(Structure::new_unique(Ptr::null(), true)),
            attributes: TaggedPointer::null(),
        }
    }
    /// Returns an immutable reference to the attributes.
    pub fn attributes_map(&self) -> Option<&AttributesMap> {
        self.attributes.as_ref()
    }

    pub fn attributes_map_mut(&self) -> Option<&mut AttributesMap> {
        self.attributes.as_mut()
    }
    /// Allocates an attribute map if needed.
    fn allocate_attributes_map(&mut self) {
        if !self.has_attributes() {
            self.set_attributes_map(AttributesMap::new());
        }
    }

    /// Returns true if an attributes map has been allocated.
    pub fn has_attributes(&self) -> bool {
        !self.attributes.untagged().is_null()
    }
    pub fn set_attributes_map(&mut self, attrs: AttributesMap) {
        self.attributes = TaggedPointer::new(Box::into_raw(Box::new(attrs)));
    }

    pub fn drop_attributes(&mut self) {
        if !self.has_attributes() {
            return;
        }

        drop(unsafe { Box::from_raw(self.attributes.untagged()) });

        self.attributes = TaggedPointer::null();
    }

    pub fn direct(&self, offset: u32) -> Value {
        if self.slots.is_null() {
            return Value::from(VTag::Undefined);
        }
        if offset >= self.slots.as_ref().unwrap().len() as u32 {
            return Value::from(VTag::Undefined);
        }
        unsafe { *self.slots.as_ref().unwrap().get_unchecked(offset as usize) }
    }
    pub fn direct_ref(&self, offset: u32) -> DerefPointer<Value> {
        if self.slots.is_null() {
            return DerefPointer::null();
        }
        if offset >= self.slots.as_ref().unwrap().len() as u32 {
            return DerefPointer::null();
        }
        unsafe { DerefPointer::new(self.slots.as_ref().unwrap().get_unchecked(offset as usize)) }
    }

    pub fn to_string(&self) -> String {
        match self.value {
            CellValue::String(ref s) => (*s).to_string(),
            _ => String::new(),
        }
    }

    pub fn is_false(&self) -> bool {
        false
    }

    pub fn trace(&self, stack: &mut std::collections::VecDeque<*const Ptr<Cell>>) {
        if self.has_attributes() {
            for entry in self.attributes_map().unwrap().storage.iter() {
                if entry.key.0.is_cell() {
                    stack.push_back(entry.key.0.cell_ref());
                }
            }
        }
        //self.map.trace(stack);
        if let Some(slots) = self.slots.as_ref() {
            for value in slots.iter() {
                if value.is_cell() {
                    stack.push_back(value.cell_ref());
                }
            }
        }

        let mut proto = self.prototype.as_ref();
        while let Some(p) = proto {
            stack.push_back(p as *const _);
            p.trace(stack);
            proto = p.prototype.as_ref();
        }
    }

    pub fn lookup_in_self(&mut self, sym: Symbol, slot: &mut Slot) -> bool {
        if !sym.is_index() && sym.name() == "length" {
            match self.value {
                CellValue::Array(ref a) => {
                    slot.value_c = Value::new_int(a.len() as _);
                    return true;
                }
                CellValue::String(ref a) => {
                    slot.value_c = Value::new_int(a.len() as _);

                    return true;
                }
                _ => (),
            }
        }
        /*if sym.is_index() {
            if let CellValue::Array(ref array) = &self.value {
                if let Some(value) = array.get(sym.index() as usize) {
                    slot.base = Ptr {
                        raw: self as *const Self as *mut Self,
                    };
                    slot.value = DerefPointer::new(value);
                    slot.offset = Map::NOT_FOUND;
                    return true;
                }
            }
        }
        let off = self.map.get(sym);
        if off.is_not_found() {
            slot.base = Ptr {
                raw: self as *const Self as *mut Self,
            };
            slot.value = DerefPointer::null();
            slot.offset = Map::NOT_FOUND;

            return false;
        } else {
            slot.value = self.direct_ref(off.offset);
            slot.base = Ptr {
                raw: self as *const Self as *mut Self,
            };
            true
        }*/
        if sym.is_index() {
            if let CellValue::Array(ref array) = &self.value {
                if let Some(value) = array.get(sym.index() as usize) {
                    slot.base = Ptr {
                        raw: self as *const Self as *mut Self,
                    };
                    slot.value = DerefPointer::new(value);
                    slot.offset = Map::NOT_FOUND;
                    return false; // we do not cache access to array
                } else {
                    return false;
                }
            } else {
                if self.has_attributes() {
                    let off = self.attributes_map_mut().unwrap().get(sym);
                    if off != Map::NOT_FOUND {
                        slot.value = self.direct_ref(off);
                        slot.base = Ptr {
                            raw: self as *const Self as *mut Self,
                        };
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        } else {
            if self.has_attributes() {
                let off = self.attributes_map_mut().unwrap().get(sym);
                if off != Map::NOT_FOUND {
                    slot.value = self.direct_ref(off);
                    slot.base = Ptr {
                        raw: self as *const Self as *mut Self,
                    };
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
    pub fn is_string(&self) -> bool {
        match &self.value {
            CellValue::String(_) => true,
            _ => false,
        }
    }
    pub fn is_function(&self) -> bool {
        match &self.value {
            CellValue::Function(_) => true,
            _ => false,
        }
    }
    pub fn is_generator(&self) -> bool {
        match &self.value {
            _ => false,
        }
    }

    pub fn insert(&mut self, sym: Symbol, slot: &mut Slot) {
        if self.slots.is_null() {
            self.slots = TaggedPointer::new(Box::into_raw(Box::new(Vec::with_capacity(4))));
        }
        self.allocate_attributes_map();
        let off = self.attributes_map_mut().unwrap().insert(sym);
        if off as usize == self.slots.as_ref().unwrap().len() {
            self.slots.as_mut().unwrap().push(Value::empty());
        }
        slot.value = self.direct_ref(off);
        slot.base = Ptr {
            raw: self as *mut Self,
        };
        slot.offset = off;

        /*if !self.lookup_in_self(sym, slot) {
            let mut offset = 422;
            let map = self.map.add_property_transition(sym, &mut offset, 0);
            self.map = map;
            self.slots
                .as_mut()
                .unwrap()
                .resize(self.map.get_slots_size(), Value::empty());
            println!("new offset: {}", offset);
            slot.offset = offset;
            println!("shit");
            slot.value = self.direct_ref(offset);
        } else {
            println!("new offset: {}", slot.offset);
            let r = self.direct_ref(slot.offset);
            slot.value = r;
        }*/
    }
    pub fn lookup(&mut self, sym: Symbol, slot: &mut Slot) -> bool {
        let mut object = Some(DerefPointer::new(self));
        while let Some(mut obj) = object {
            if obj.lookup_in_self(sym, slot) {
                return true;
            }

            object = obj.prototype.map(|x| DerefPointer::new(x.get()));
        }
        false
    }

    pub fn func_value_unchecked_mut(&mut self) -> &mut Function {
        match &mut self.value {
            CellValue::Function(f) => f,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    pub fn func_value_unchecked(&self) -> &Function {
        match &self.value {
            CellValue::Function(f) => f,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    pub fn inline(&mut self, value: Value) -> u32 {
        self.slots.as_mut().unwrap().push(value);
        self.slots.as_mut().unwrap().len() as u32 - 1
    }

    pub fn store_direct(&mut self, offset: u32, value: Value) -> bool {
        if self.slots.is_null() {
            return false;
        } else {
            unsafe {
                *self
                    .slots
                    .as_mut()
                    .unwrap()
                    .get_unchecked_mut(offset as usize) = value;

                true
            }
        }
    }
}

impl Drop for Cell {
    fn drop(&mut self) {
        if self.color == CELL_WHITE_B {
            return;
        }
        self.drop_attributes();
        self.value = CellValue::None;
        self.color = CELL_WHITE_B;
    }
}

pub struct Slot {
    pub base: Ptr<Cell>,
    pub value: DerefPointer<Value>,
    pub value_c: Value,
    pub offset: u32,
}

impl Slot {
    pub fn new() -> Self {
        Self {
            base: Ptr::null(),
            offset: Map::NOT_FOUND,
            value: DerefPointer::null(),
            value_c: Value::empty(),
        }
    }
    pub fn store(&mut self, val: Value) -> bool {
        if self.value.is_null() {
            return false;
        }
        *self.value = val;
        true
    }
    pub fn value(&self) -> Value {
        if self.value.is_null() {
            if self.value_c.is_empty() {
                Value::from(VTag::Undefined)
            } else {
                self.value_c
            }
        } else {
            *self.value
        }
    }
}
