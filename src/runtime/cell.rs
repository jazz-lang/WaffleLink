/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

use super::module::Module;
use super::process::Process;
use super::scheduler::process_worker::ProcessWorker;
use super::state::*;
use super::value::*;
use super::*;
use crate::bytecode;
use crate::bytecode::cfg::FunctionCFG;
use crate::heap::space::Space;
use crate::interpreter::context::Context;
use crate::util::arc::Arc;
use crate::util::mem::Address;
use crate::util::ptr::*;
use crate::util::tagged::*;
use bytecode::basicblock::BasicBlock;
use regex::Regex;
use std::fs::File;
use std::string::String;
use std::vec::Vec;
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

#[derive(Clone, Copy)]
#[repr(C)]
pub enum ReturnValue {
    Value(Value),
    YieldProcess,
    SuspendProcess,
}

pub type NativeFn = extern "C" fn(
    &mut ProcessWorker,
    &RcState,
    &Arc<Process>,
    Value,
    &[Value],
) -> Result<ReturnValue, Value>;

#[derive(Clone)]
#[repr(C)]
pub struct FunctionMetadata {
    pub stack_size: usize,
    pub can_jit: bool,
    pub hotness: usize,
    pub cfg: Option<FunctionCFG>,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct AttributesMapTable {
    pub values: Vec<(Value, Value)>,
}

impl AttributesMapTable {
    pub fn new() -> Self {
        Self { values: vec![] }
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn values(&self) -> Vec<&Value> {
        self.values.iter().map(|x| &x.1).collect()
    }

    pub fn remove(&mut self, k: &Value) -> Option<Value> {
        let mut pos = None;
        for (i, (key, _)) in self.values.iter().enumerate() {
            if key.to_string() == k.to_string() {
                pos = Some(i);
                break;
            }
        }

        if let Some(p) = pos {
            Some(self.values.remove(p).1)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, k: &Value) -> Option<&mut Value> {
        for (key, val) in self.values.iter_mut() {
            if key.to_string() == k.to_string() {
                return Some(val);
            }
        }
        None
    }
    pub fn get(&self, k: &Value) -> Option<&Value> {
        for (key, val) in self.values.iter() {
            //log::trace!("cmp {} == {} is {}", key, k, key == k);
            if key.to_string() == k.to_string() {
                return Some(val);
            }
        }
        None
    }

    pub fn insert(&mut self, k: Value, v: Value) {
        if let Some(field) = self.get_mut(&k) {
            *field = v;
        } else {
            self.values.push((k, v));
        }
    }

    pub fn contains(&self, k: Value) -> bool {
        self.get(&k).is_some()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (Value, Value)> {
        self.values.iter()
    }
}

impl Default for FunctionMetadata {
    fn default() -> Self {
        Self {
            stack_size: 0,
            can_jit: true,
            hotness: 0,
            cfg: None,
        }
    }
}

pub struct Function {
    pub name: Value,
    pub upvalues: Vec<Value>,
    pub argc: i32,
    pub native: Option<NativeFn>,
    pub module: Arc<Module>,
    pub code: Arc<Vec<BasicBlock>>,
    pub md: FunctionMetadata,
}

pub struct Generator {
    pub function: Value,
    pub last_ip: usize,
    pub last_bp: usize,
    pub last_this: Value,
    pub registers: [Value; 32],
    pub stack: Vec<Value>,
    pub upvalues: Vec<Value>,
    pub complete: bool,
    pub dest: u16,
}

pub enum GeneratorList {
    NotStarted,
    Next(Value, Option<Generator>),
}

pub enum CellValue {
    None,
    /// Heap allocated number.
    ///
    /// Not all numbers allocated in heap, only ones that doesn't fit into NaN-boxed value.
    Number(f64),
    String(Arc<String>),
    InternedString(crate::runtime::interner::Name),
    Array(Box<Vec<Value>>),
    ByteArray(Box<Vec<u8>>),
    Function(Arc<Function>),
    Module(Arc<Module>),
    Process(Arc<Process>),
    Regex(Arc<Regex>),
    Duration(std::time::Duration),
    GeneratorFunction(Box<Generator>),
    File(File),
}

#[repr(C)]
pub struct Cell {
    pub value: CellValue,
    pub prototype: Option<CellPointer>,
    pub attributes: TaggedPointer<AttributesMap>,
    pub generation: u8,
    pub color: u8,
    pub forward: crate::util::mem::Address,
}

pub type AttributesMap = hashlink::LinkedHashMap<Arc<String>, Value, fxhash::FxBuildHasher>;

pub const MARK_BIT: usize = 0;

impl Cell {
    pub fn copy_to_addr(&self, obj: Address) {
        unsafe {
            std::ptr::copy(self as *const Cell, obj.to_mut_ptr(), 1);
        }
    }
    pub fn with_prototype(value: CellValue, prototype: CellPointer) -> Self {
        Self {
            value,
            prototype: Some(prototype),
            attributes: TaggedPointer::null(),
            generation: 0,
            color: CELL_WHITE_A,
            forward: crate::util::mem::Address::null(),
        }
    }

    pub fn new(value: CellValue) -> Self {
        Self {
            value,
            prototype: None,
            attributes: TaggedPointer::null(),
            generation: 0,
            color: CELL_WHITE_A,
            forward: crate::util::mem::Address::null(),
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
            self.set_attributes_map(AttributesMap::default());
        }
    }

    /// Returns true if an attributes map has been allocated.
    pub fn has_attributes(&self) -> bool {
        !self.attributes.untagged().is_null()
    }

    pub fn drop_attributes(&mut self) {
        if !self.has_attributes() {
            return;
        }

        drop(unsafe { Box::from_raw(self.attributes.untagged()) });

        self.attributes = TaggedPointer::null();
    }

    /// Adds a new attribute to the current object.
    pub fn add_attribute(&mut self, name: Arc<String>, object: Value) {
        self.allocate_attributes_map();
        assert!(name.references() != 0);
        self.attributes_map_mut().unwrap().insert(name, object);
    }

    pub fn set_attributes_map(&mut self, attrs: AttributesMap) {
        self.attributes = TaggedPointer::new(Box::into_raw(Box::new(attrs)));
    }
    pub fn trace<F>(&self, mut cb: F)
    where
        F: FnMut(*const CellPointer),
    {
        unsafe {
            if let Some(ref prototype) = &self.prototype {
                cb(prototype)
            }
            if self.attributes.is_null() == false {
                for (_, attribute) in self.attributes.as_ref().unwrap().iter() {
                    if attribute.is_cell() {
                        cb(&attribute.u.ptr);
                    }
                }
            }

            match self.value {
                CellValue::Array(ref array) => {
                    for value in array.iter() {
                        if value.is_cell() {
                            cb(&value.u.ptr);
                        }
                    }
                }
                CellValue::Function(ref f) => {
                    for value in f.upvalues.iter() {
                        if value.is_cell() {
                            cb(&value.u.ptr);
                        }
                    }
                    if f.name.is_cell() {
                        cb(&f.name.u.ptr);
                    }
                }
                CellValue::GeneratorFunction(ref generator) => {
                    if generator.function.is_cell() {
                        cb(&generator.function.u.ptr);
                    }
                    for value in generator.stack.iter() {
                        if value.is_cell() {
                            cb(&value.u.ptr);
                        }
                    }

                    for value in generator.registers.iter() {
                        if value.is_cell() {
                            cb(&value.u.ptr);
                        }
                    }
                    for value in generator.upvalues.iter() {
                        if value.is_cell() {
                            cb(&value.u.ptr);
                        }
                    }

                    if generator.last_this.is_cell() {
                        cb(&generator.last_this.u.ptr);
                    }
                }
                _ => (),
            }
        }
    }

    /// Sets the prototype of this object.
    pub fn set_prototype(&mut self, prototype: CellPointer) {
        self.prototype = Some(prototype);
    }

    /// Returns the prototype of this object.
    pub fn prototype(&self) -> Option<CellPointer> {
        self.prototype
    }

    /// Returns and removes the prototype of this object.
    pub fn take_prototype(&mut self) -> Option<CellPointer> {
        self.prototype.take()
    }

    /// Removes an attribute and returns it.
    pub fn remove_attribute(&mut self, name: &Arc<String>) -> Option<Value> {
        if let Some(map) = self.attributes_map_mut() {
            map.remove(name)
        } else {
            None
        }
    }

    /// Returns all the attributes available to this object.
    pub fn attributes(&self) -> Vec<Value> {
        let mut attributes = Vec::new();

        if let Some(map) = self.attributes_map() {
            push_collection!(map, values, attributes);
        }

        attributes
    }

    /// Returns all the attribute names available to this object.
    pub fn attribute_names(&self) -> Vec<&Arc<String>> {
        let mut attributes = Vec::new();

        if let Some(map) = self.attributes_map() {
            for (key, _) in map.iter() {
                attributes.push(key);
            }
            //push_collection!(map, keys, attributes);
        }

        attributes
    }
    /// Looks up an attribute without walking the prototype chain.
    pub fn lookup_attribute_in_self(&self, name: &Arc<String>) -> Option<Value> {
        if let Some(map) = self.attributes_map() {
            map.get(name).map(|x| *x)
        } else {
            None
        }
    }
    /// Looks up an attribute in either the current object or a parent object.
    pub fn lookup_attribute(&self, name: &Arc<String>) -> Option<Value> {
        let got = self.lookup_attribute_in_self(&name);

        if got.is_some() {
            return got;
        }

        // Method defined somewhere in the object hierarchy
        if self.prototype().is_some() {
            let mut opt_parent = self.prototype();

            while let Some(parent_ptr) = opt_parent {
                if parent_ptr.is_tagged_number() || parent_ptr.raw.is_null() {
                    break;
                }
                let parent = parent_ptr;
                let got = parent.get().lookup_attribute_in_self(name);

                if got.is_some() {
                    return got;
                }

                opt_parent = parent.get().prototype();
            }
        }

        None
    }
}
#[repr(C)]
pub struct CellPointer {
    /// 'raw' points to permanent or GC heap.
    ///
    /// If cell is in permanent heap then one bit is set to 1.
    pub raw: TaggedPointer<Cell>,
}

impl CellPointer {
    pub fn function_value(&self) -> Result<&Arc<Function>, String> {
        match &self.get().value {
            CellValue::Function(func) => Ok(func),
            _ => Err("Not a function".to_owned()),
        }
    }
    pub fn copy_to(
        &self,
        old_space: &mut Space,
        new_space: &mut Space,
        needs_gc: &mut bool,
    ) -> CellPointer {
        self.increment_generation();
        let result;
        if self.get().generation >= 5 {
            result = old_space.allocate(std::mem::size_of::<Cell>(), needs_gc);
        } else {
            result = new_space.allocate(std::mem::size_of::<Cell>(), needs_gc);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                self as *const Self as *const u8,
                result.to_mut_ptr::<u8>(),
                std::mem::size_of::<Self>(),
            );
        }
        CellPointer {
            raw: TaggedPointer::new(result.to_mut_ptr()),
        }
    }

    pub fn increment_generation(&self) {
        let cell = self.get_mut();
        if cell.generation < MIN_OLD_SPACE_GENERATION {
            cell.generation += 1;
        }
    }

    pub fn is_marked(&self) -> bool {
        self.get().attributes.bit_is_set(0)
    }

    pub fn get_color(&self) -> u8 {
        self.get().color
    }
    #[allow(unused)]
    fn is_color(&self, color: u8) -> bool {
        let c = self.get().color;
        if color == CELL_BLACK {
            return (c & CELL_BLACK) != 0;
        } else if color == CELL_GREY {
            return c == CELL_GREY;
        } else if color == CELL_WHITES || color == CELL_WHITE_A || color == CELL_WHITE_B {
            return (c & CELL_WHITES) != 0;
        } else {
            c == color
        }
    }

    pub fn set_color(&self, mut color: u8) -> u8 {
        std::mem::swap(&mut self.get_mut().color, &mut color);
        color
    }

    pub fn mark(&self, value: bool) {
        if value {
            self.get_mut().attributes.set_bit(0);
        } else {
            self.get_mut().attributes.unset_bit(0);
        }
    }

    pub fn is_soft_marked(&self) -> bool {
        self.get().attributes.bit_is_set(1)
    }

    pub fn soft_mark(&self, value: bool) {
        if value {
            self.get_mut().attributes.set_bit(1);
        } else {
            self.get_mut().attributes.unset_bit(1);
        }
    }
    pub fn array_value(&self) -> Option<&Box<Vec<Value>>> {
        match &self.get().value {
            CellValue::Array(a) => Some(a),
            _ => None,
        }
    }
    pub fn array_value_mut(&self) -> Option<&mut Box<Vec<Value>>> {
        match &mut self.get_mut().value {
            CellValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn get(&self) -> &Cell {
        self.raw.as_ref().unwrap()
    }

    pub fn get_mut(&self) -> &mut Cell {
        self.raw.as_mut().unwrap()
    }
    /// Returns true if this cell is from permanent heap.
    ///
    /// Multiple threads may invoke this method and no races will occure since we just check one bit from pointer
    /// and do not update pointer.
    pub fn is_permanent(&self) -> bool {
        self.raw.bit_is_set(0)
    }
    /// Set permanent bit in object. This method is *unsafe* because races may occure if you will try to use this method.
    pub unsafe fn set_permanent(&mut self) {
        self.raw.set_bit(0)
    }

    pub fn prototype(&self, _: &State) -> Option<CellPointer> {
        self.get().prototype()
    }
    pub fn set_prototype(&self, proto: CellPointer) {
        self.get_mut().set_prototype(proto);
    }

    pub fn is_kind_of(&self, state: &RcState, other: CellPointer) -> bool {
        let mut prototype = self.prototype(state);

        while let Some(proto) = prototype {
            if other.is_function() {
                if let Some(other) =
                    other.lookup_attribute_in_self(state, &Arc::new("prototype".to_owned()))
                {
                    if other.as_cell() == proto {
                        return true;
                    }
                }
            }
            if proto == other {
                return true;
            }

            prototype = proto.prototype(state);
        }

        false
    }

    /// Adds an attribute to the object this pointer points to.
    pub fn add_attribute(&self, proc: &Arc<Process>, name: &Arc<String>, attr: Value) {
        proc.local_data_mut().heap.field_write_barrier(*self, attr);
        self.get_mut().add_attribute(name.clone(), attr);
    }

    pub fn add_attribute_without_barrier(&self, name: &Arc<String>, attr: Value) {
        self.get_mut().add_attribute(name.clone(), attr);
    }

    /// Looks up an attribute.
    pub fn lookup_attribute(&self, state: &RcState, name: &Arc<String>) -> Option<Value> {
        if self.is_tagged_number() {
            state
                .number_prototype
                .as_cell()
                .get()
                .lookup_attribute(name)
        } else {
            self.get().lookup_attribute(name)
        }
    }

    /// Looks up an attribute without walking the prototype chain.
    pub fn lookup_attribute_in_self(&self, state: &RcState, name: &Arc<String>) -> Option<Value> {
        self.get().lookup_attribute_in_self(name)
    }
    pub fn is_false(&self) -> bool {
        if self.raw.is_null() {
            return true;
        }
        match self.get().value {
            CellValue::GeneratorFunction(ref gen) => gen.complete,
            CellValue::String(ref s) => s.len() == 0,
            CellValue::Number(ref n) => *n == 0.0,
            _ => false,
        }
    }

    pub fn attributes(&self) -> Vec<Value> {
        if self.is_tagged_number() {
            vec![]
        } else {
            self.get().attributes()
        }
    }
    pub fn is_tagged_number(&self) -> bool {
        //self.raw.bit_is_set(0)
        false
    }

    pub fn attribute_names(&self) -> Vec<&Arc<String>> {
        if self.is_tagged_number() {
            vec![]
        } else {
            self.get().attribute_names()
        }
    }

    pub fn is_function(&self) -> bool {
        match self.get().value {
            CellValue::Function(_) => true,
            _ => false,
        }
    }

    pub fn is_process(&self) -> bool {
        match self.get().value {
            CellValue::Process(_) => true,
            _ => false,
        }
    }

    pub fn is_module(&self) -> bool {
        match self.get().value {
            CellValue::Module(_) => true,
            _ => false,
        }
    }
    pub fn is_file(&self) -> bool {
        match self.get().value {
            CellValue::File(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self.get().value {
            CellValue::String(_) | CellValue::InternedString(_) => true,
            _ => false,
        }
    }

    pub fn is_interned_str(&self) -> bool {
        match self.get().value {
            CellValue::InternedString(_) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self.get().value {
            CellValue::Array(_) => true,
            _ => false,
        }
    }

    pub fn is_byte_array(&self) -> bool {
        match self.get().value {
            CellValue::ByteArray(_) => true,
            _ => false,
        }
    }
    pub fn to_array(&self) -> Option<&Vec<Value>> {
        match self.get().value {
            CellValue::Array(ref a) => Some(a),
            _ => None,
        }
    }

    pub fn module_value(&self) -> Option<&Arc<Module>> {
        match self.get().value {
            CellValue::Module(ref m) => Some(m),
            _ => None,
        }
    }

    pub fn module_value_mut(&self) -> Option<&mut Arc<Module>> {
        match self.get_mut().value {
            CellValue::Module(ref mut m) => Some(m),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        match self.get().value {
            CellValue::Regex(ref s) => format!("{}", s),
            CellValue::String(ref s) => (**s).clone(),
            CellValue::Array(ref array) => {
                use std::fmt::Write;
                let mut fmt_buf = String::new();
                for (i, object) in array.iter().enumerate() {
                    write!(fmt_buf, "{}", object.to_string()).unwrap();
                    if i != array.len() - 1 {
                        write!(fmt_buf, ",").unwrap();
                    }
                }

                fmt_buf
            }
            CellValue::GeneratorFunction(_) => "GeneratorFunction".to_owned(),
            CellValue::InternedString(ref s) => crate::runtime::interner::str(*s).to_string(),
            CellValue::Duration(d) => format!("Duration({})", d.as_millis()),
            CellValue::Process(_) => String::from("Process"),
            CellValue::File(_) => String::from("File"),
            CellValue::ByteArray(ref array) => format!("ByteArray({:?})", array),
            CellValue::Function(ref f) => format!("function {}(...) {{...}}", f.name.to_string()),
            CellValue::Number(n) => n.to_string(),
            CellValue::Module(_) => String::from("Module"),
            CellValue::None => {
                if self.get().has_attributes() {
                    use std::fmt::Write;
                    let mut fmt_buf = String::new();
                    write!(fmt_buf, "{{\n").unwrap();
                    for (_, (key, value)) in
                        self.get().attributes.as_ref().unwrap().iter().enumerate()
                    {
                        write!(fmt_buf, "  {}: {}\n", key, value.to_string()).unwrap();
                    }
                    write!(fmt_buf, "}}").unwrap();

                    fmt_buf
                } else {
                    String::from("{}")
                }
            }
        }
    }
}

impl Copy for CellPointer {}
impl Clone for CellPointer {
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq for CellPointer {
    fn eq(&self, other: &Self) -> bool {
        self.raw.untagged() == other.raw.untagged()
    }
}

impl From<*const Cell> for CellPointer {
    fn from(x: *const Cell) -> Self {
        Self {
            raw: TaggedPointer::new(x as *mut _),
        }
    }
}

use std::fmt;

impl fmt::Debug for CellPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

impl fmt::Display for CellPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[no_mangle]
pub extern "C" fn cell_add_attribute_wo_barrier(cell: *const Cell, key: Value, value: Value) {
    let key = key.to_string();
    let key_ptr = Arc::new(key);
    let pointer = CellPointer::from(cell);
    pointer.add_attribute_without_barrier(&key_ptr, value);
}

pub extern "C" fn cell_add_attribute_barriered(
    proc: *const Process,
    cell: *const Cell,
    key: Value,
    value: Value,
) {
    let proc = unsafe { Arc::from_raw(proc as *mut Process) };
    let key = key.to_string();
    let key_ptr = Arc::new(key);
    let pointer = CellPointer::from(cell);
    pointer.add_attribute(&proc, &key_ptr, value);
}

#[no_mangle]
pub extern "C" fn cell_lookup_attribute(cell: *const Cell, key: Value) -> Value {
    let key = key.to_string();
    let key_ptr = Arc::new(key);
    let pointer = CellPointer::from(cell);
    if let Some(value) = pointer.lookup_attribute(&RUNTIME.state, &key_ptr) {
        return value;
    } else {
        Value::empty()
    }
}

#[no_mangle]
pub extern "C" fn cell_set_prototype(cell: *const Cell, prototype: *const Cell) {
    let pointer = CellPointer::from(cell);
    pointer.set_prototype(CellPointer::from(prototype));
}

impl Drop for Cell {
    fn drop(&mut self) {
        std::mem::replace(&mut self.value, CellValue::None);
        self.drop_attributes();
        self.generation = 127;
    }
}

unsafe impl Send for CellPointer {}

use std::hash::{Hash, Hasher};
impl Hash for CellPointer {
    fn hash<H: Hasher>(&self, h: &mut H) {
        (self.raw.raw as usize).hash(h);
    }
}

impl Eq for CellPointer {}
