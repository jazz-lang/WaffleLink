use super::*;
use cgc::api::*;
use cgc::heap::*;
use fancy_regex::Regex;
use hashlink::*;
use value::*;

pub enum CellValue {
    None,
    String(Box<String>),
    Array(Box<Vec<Value>>),
    ByteArray(Box<Vec<u8>>),
    RegEx(Box<Regex>),
    File(Box<std::fs::File>),
}

pub struct Cell {
    pub value: CellValue,
    pub prototype: Option<Handle<Cell>>,
    pub properties: Box<LinkedHashMap<String, Value>>,
}

impl Cell {
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
    pub fn put(&mut self, value: Value, name: &str) -> bool {
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

    pub fn lookup(&self, name: &str) -> Option<Value> {
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
