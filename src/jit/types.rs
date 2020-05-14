#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Type {
    Int32,
    Float64,
    Undefined,
    Null,
    Array,
    String,
    /// Native function, this allows invoking it by just doing `call *(%rax + 8)`
    NativeFunction(usize),
    /// Regular function is bytecode function.
    RegularFunction(usize),
    /// Basically the same as Type::Value, but could be more optimized
    Object(usize),
    /// Any value
    Value,
}
use crate::common::mem;
use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MachineMode {
    Int8,
    Int32,
    Int64,
    IntPtr,
    Float32,
    Float64,
    Ptr,
}

impl MachineMode {
    pub fn size(self) -> i32 {
        match self {
            MachineMode::Int8 => 1,
            MachineMode::Int32 => 4,
            MachineMode::Int64 => 8,
            MachineMode::IntPtr | MachineMode::Ptr => mem::ptr_width(),
            MachineMode::Float32 => 4,
            MachineMode::Float64 => 8,
        }
    }

    pub fn is_int8(self) -> bool {
        match self {
            MachineMode::Int8 => true,
            _ => false,
        }
    }

    pub fn is_float(self) -> bool {
        match self {
            MachineMode::Float32 | MachineMode::Float64 => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct TypeListId(u32);

impl TypeListId {
    pub fn idx(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for TypeListId {
    fn from(data: usize) -> TypeListId {
        assert!(data < u32::max_value() as usize);
        TypeListId(data as u32)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TypeList {
    Empty,
    List(Vec<Type>),
}

impl TypeList {
    pub fn empty() -> TypeList {
        TypeList::Empty
    }

    pub fn single(ty: Type) -> TypeList {
        TypeList::List(vec![ty])
    }

    pub fn with(type_params: Vec<Type>) -> TypeList {
        if type_params.len() == 0 {
            TypeList::Empty
        } else {
            TypeList::List(type_params)
        }
    }

    pub fn len(&self) -> usize {
        match self {
            &TypeList::Empty => 0,
            &TypeList::List(ref params) => params.len(),
        }
    }

    pub fn iter(&self) -> TypeListIter {
        TypeListIter {
            params: self,
            idx: 0,
        }
    }
}

impl Index<usize> for TypeList {
    type Output = Type;

    fn index(&self, idx: usize) -> &Type {
        match self {
            &TypeList::Empty => panic!("out-of-bounds"),
            &TypeList::List(ref params) => &params[idx],
        }
    }
}

pub struct TypeListIter<'a> {
    params: &'a TypeList,
    idx: usize,
}

impl<'a> Iterator for TypeListIter<'a> {
    type Item = Type;

    fn next(&mut self) -> Option<Type> {
        match self.params {
            &TypeList::Empty => None,

            &TypeList::List(ref params) => {
                if self.idx < params.len() {
                    let ret = params[self.idx];
                    self.idx += 1;

                    Some(ret)
                } else {
                    None
                }
            }
        }
    }
}

pub struct TypeLists {
    lists: HashMap<TypeList, TypeListId>,
    values: Vec<TypeList>,
    next_id: usize,
}

impl TypeLists {
    pub fn new() -> TypeLists {
        TypeLists {
            lists: HashMap::new(),
            values: Vec::new(),
            next_id: 0,
        }
    }

    pub fn insert(&mut self, list: TypeList) -> TypeListId {
        if let Some(&val) = self.lists.get(&list) {
            return val;
        }

        let id: TypeListId = self.next_id.into();
        self.lists.insert(list.clone(), id);

        self.values.push(list);

        self.next_id += 1;

        id
    }

    pub fn get(&self, id: TypeListId) -> TypeList {
        self.values[id.idx()].clone()
    }
}

pub enum FeedBack {
    TypeList(TypeList),
    Loop {
        osr_enter: Option<usize>,
        hotness: usize,
    },
    None,
}
