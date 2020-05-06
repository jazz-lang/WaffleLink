use super::*;
use cgc::api::*;
use cgc::heap::*;
use fancy_regex::Regex;
use hashlink::*;
use value::*;

pub enum Function {
    AsyncNative {
        name: Value,
        native: fn(
            &mut Runtime,
            Value,
            &[Value],
        ) -> Box<dyn std::future::Future<Output = crate::interpreter::Return>>,
    },
    Native {
        name: Value,
        native: fn(&mut Runtime, Value, &[Value]) -> crate::interpreter::Return,
    },
    Regular(RegularFunction),
}

pub struct RegularFunction {
    pub name: Value,
    pub source: String,
    pub arguments: Vec<String>,
    pub env: Value,
    pub code: Handle<crate::bytecode::CodeBlock>,
}

pub enum CellValue {
    None,
    String(Box<String>),
    Array(Vec<Value>),
    ByteArray(Box<Vec<u8>>),
    RegEx(Regex),
    File(std::fs::File),
    Future(std::pin::Pin<Box<dyn std::future::Future<Output = crate::interpreter::Return>>>),
    Function(Function),
}

pub struct Cell {
    pub value: CellValue,
    pub prototype: Option<Handle<Cell>>,
    pub properties: Box<LinkedHashMap<String, Value>>,
}

impl Cell {
    pub fn take_value(&mut self) -> CellValue {
        std::mem::replace(&mut self.value, CellValue::None)
    }
    pub fn new(val: CellValue, proto: Option<Handle<Cell>>) -> Self {
        Self {
            value: val,
            prototype: proto,
            properties: Box::new(LinkedHashMap::new()),
        }
    }
    pub fn is_string(&self) -> bool {
        if let CellValue::String(_) = self.value {
            true
        } else {
            false
        }
    }
    pub fn is_array(&self) -> bool {
        if let CellValue::Array(_) = self.value {
            true
        } else {
            false
        }
    }
    pub fn is_any_array(&self) -> bool {
        match self.value {
            CellValue::Array(_) => true,
            CellValue::ByteArray(_) => true,
            _ => false,
        }
    }
    fn _get_len(&self) -> Option<usize> {
        match self.value {
            CellValue::Array(ref arr) => Some(arr.len()),
            CellValue::ByteArray(ref arr) => Some(arr.len()),
            CellValue::String(ref string) => Some(string.len()),
            _ => None,
        }
    }
    pub fn put(&mut self, rt: &mut Runtime, key: Value, value: Value) {
        if key.is_number() {
            let idx = key.to_number().floor() as usize;
            if let CellValue::Array(ref mut arr) = self.value {
                if idx >= arr.len() {
                    for _ in 0..=idx {
                        arr.push(Value::default());
                    }
                    arr[idx] = value;
                }
            } else if let CellValue::ByteArray(ref mut arr) = self.value {
                if idx < arr.len() {
                    arr[idx] = value.to_int32() as u8;
                    return;
                }
            } else if let CellValue::String(ref mut s) = self.value {
                if idx < s.len() {
                    unimplemented!()
                }
            }
        }

        self.put_named(value, &key.to_string(rt));
    }
    pub fn put_named(&mut self, value: Value, name: &str) -> bool {
        if self.properties.contains_key(name) {
            self.properties[name] = value;
            false
        } else {
            self.properties.insert(name.to_owned(), value);
            true
        }
    }

    pub fn lookup_in_self(&self, name: &str) -> Option<Value> {
        if name == "length" {
            if let Some(len) = self._get_len() {
                return Some(Value::new_int(len as i32));
            }
        }
        if let Some(val) = self.properties.get(name) {
            return Some(*val);
        }
        None
    }

    pub fn lookup_named(&self, name: &str) -> Option<Value> {
        use super::deref_ptr::DerefPointer;
        let mut object = Some(DerefPointer::new(self));
        while let Some(obj) = object {
            if let Some(prop) = obj.lookup_in_self(name) {
                return Some(prop);
            }
            object = obj.prototype.map(|x| DerefPointer::new(x.get()));
        }
        None
    }
    fn _try_index(&mut self, rt: &mut Runtime, x: i32) -> Option<Value> {
        match self.value {
            CellValue::Array(ref mut arr) => {
                if x >= arr.len() as i32 {
                    arr.push(Value::default());
                    arr[x as usize] = Value::undefined();
                }
                return Some(arr[x as usize]);
            }
            CellValue::ByteArray(ref mut arr) => {
                if x >= arr.len() as i32 {
                    return None;
                }
                return Some(Value::new_int(arr[x as usize] as i32));
            }
            CellValue::String(ref s) => {
                let character = s.chars().nth(x as usize);
                if let Some(character) = character {
                    return Some(Value::from(
                        rt.allocate_string(character.to_string()).to_heap(),
                    ));
                } else {
                    return None;
                }
            }
            _ => None,
        }
    }

    pub fn lookup(&mut self, rt: &mut Runtime, value: Value) -> Option<Value> {
        let try_index = value.is_number();
        use super::deref_ptr::DerefPointer;
        let mut object = Some(DerefPointer::new(self));
        let name = value.to_string(rt);
        while let Some(obj) = object {
            if try_index {
                if let Some(val) = self._try_index(rt, value.to_int32()) {
                    return Some(val);
                }
            }
            if let Some(prop) = obj.lookup_in_self(&name) {
                return Some(prop);
            }
            object = obj.prototype.map(|x| DerefPointer::new(x.get()));
        }
        None
    }
}

impl Traceable for Cell {
    fn trace_with(&self, tracer: &mut Tracer) {
        for (_, prop) in self.properties.iter() {
            prop.trace_with(tracer);
        }
        self.prototype.trace_with(tracer);
        match &self.value {
            CellValue::Array(arr) => arr.trace_with(tracer),
            _ => (),
        }
    }
}
impl Finalizer for Cell {}
