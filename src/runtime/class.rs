use super::cell_type::CellType;
use crate::prelude::*;
#[repr(C)]
pub struct Class {
    pub cell_type: CellType,
    pub name: Value,
    pub(crate) super_class: Option<Handle<Self>>,
    pub(crate) members: Handle<Map>,
    pub(crate) fields: Handle<Map>,
}

impl Class {
    pub fn super_class(&self) -> Option<&Handle<Self>> {
        self.super_class.as_ref()
    }
    pub fn new(
        isolate: &RCIsolate,
        name: Value,
        fields: &[&str],
        members: &[(&str, Value)],
        super_class: Option<Handle<Self>>,
    ) -> Local<Self> {
        let map = Map::new(
            isolate,
            compute_hash_default,
            iseq_default,
            fields.len() as u32
                + if let Some(cls) = super_class {
                    cls.fields.count() as u32
                } else {
                    0
                },
        );
        let mut this = isolate.new_local(Self {
            cell_type: CellType::Class,
            name,
            super_class,
            fields: map.to_heap(),
            members: Map::new(
                isolate,
                compute_hash_default,
                iseq_default,
                members.len() as u32
                    + if let Some(cls) = super_class {
                        cls.members.count() as u32
                    } else {
                        0
                    },
            )
            .to_heap(),
        });
        for field in fields {
            this.add_field(isolate, field);
        }
        if let Some(cls) = super_class {
            cls.fields.for_each(|node| {
                let key = node.key();
                assert!(key.is_sym());
                let val = node.val;
                assert!(val.is_int32());
                let interned = *key;
                if let Some(_val) = this.fields.lookup(isolate, interned) {
                    return;
                } else {
                    let c = this.fields.count();
                    this.fields
                        .insert(isolate, interned, Value::new_int(c as _));
                }
            });
        }
        this
    }

    pub fn member(&self, isolate: &RCIsolate, name: &str) -> Option<Value> {
        let interned = Value::new_sym(isolate.intern_str(name));
        self.members.lookup(isolate, interned)
    }

    pub fn member_mut(&mut self, isolate: &RCIsolate, name: &str) -> Option<&mut Value> {
        let interned = Value::new_sym(isolate.intern_str(name));
        self.members.lookup_mut(isolate, interned)
    }
    fn add_member(&mut self, isolate: &RCIsolate, name: &str, value: Value) {
        let interned = Value::new_sym(isolate.intern_str(name));
        if let Some(val) = self.members.lookup(isolate, interned) {
            return;
        } else {
            self.members.insert(isolate, interned, value);
        }
    }
    /// Get offset of field in class.
    /// return: field offset
    pub fn field(&self, isolate: &RCIsolate, name: &str) -> Option<u32> {
        let interned = Value::new_sym(isolate.intern_str(name));
        self.fields
            .lookup(isolate, interned)
            .map(|x| x.as_int32() as _)
    }
    /// Add field to class.
    /// return: field offset
    fn add_field(&mut self, isolate: &RCIsolate, name: &str) -> u32 {
        let interned = Value::new_sym(isolate.intern_str(name));
        if let Some(val) = self.fields.lookup(isolate, interned) {
            return val.as_int32() as _;
        } else {
            let c = self.fields.count();
            self.fields
                .insert(isolate, interned, Value::new_int(c as _));
            c as u32
        }
    }
}

/// Instance of class. This is dynamically sized object, this means
/// each instance of this object might use different amount of memory
/// based on field count.
#[repr(C)]
pub struct Instance {
    pub cell_type: CellType,
    pub class: Handle<Class>,
    fields: Value,
}

impl Instance {
    pub fn new_self(isolate: &RCIsolate, class: Handle<Class>) -> Local<Self> {
        let mut this = isolate.new_local(Self {
            cell_type: CellType::Instance,
            //super_: None,
            class,
            fields: Value::undefined(),
        });
        this.fields_mut()
            .iter_mut()
            .for_each(|x| *x = Value::undefined());
        this
    }

    pub fn new(isolate: &RCIsolate, class: Handle<Class>) -> Local<Self> {
        let mut obj = Self::new_self(isolate, class);
        /*let mut prev = obj.to_heap();
        let mut c = class.super_class;
        while let Some(x) = c {
            prev.super_ = Some(Self::new_self(isolate, x).to_heap());
            prev = prev.super_.unwrap();
            c = x.super_class;
        }*/
        obj
    }
    pub fn fields(&self) -> &[Value] {
        unsafe { std::slice::from_raw_parts(&self.fields, self.class.fields.count()) }
    }

    pub fn fields_mut(&mut self) -> &mut [Value] {
        unsafe { std::slice::from_raw_parts_mut(&mut self.fields, self.class.fields.count()) }
    }

    pub fn field(&self, isolate: &RCIsolate, name: &str) -> Option<Value> {
        if let Some(ix) = self.class.field(isolate, name) {
            return Some(self.fields()[ix as usize]);
        } else {
            None
        }
    }
    pub fn field_mut(&mut self, isolate: &RCIsolate, name: &str) -> Option<&mut Value> {
        if let Some(ix) = self.class.field(isolate, name) {
            return Some(&mut self.fields_mut()[ix as usize]);
        } else {
            None
        }
    }
}

impl Handle<Instance> {
    pub fn field_slot(&self, isolate: &RCIsolate, name: &str) -> Slot {
        if let Some(ix) = self.class.field(isolate, name) {
            return Slot {
                object: Some(*self),
                value: Some(self.fields()[ix as usize]),
                ix,
            };
        } else {
            Slot::not_found()
        }
    }
}

impl GcObject for Instance {
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        self.fields()
            .iter()
            .for_each(|val| val.visit_references(tracer));
        self.class.visit_references(tracer);
        //self.super_.visit_references(tracer);
    }

    fn size(&self) -> usize {
        core::mem::size_of::<Self>() + (self.class.fields.count() * core::mem::size_of::<Value>())
    }
}

pub struct Slot {
    pub value: Option<Value>,
    pub object: Option<Handle<Instance>>,
    pub ix: u32,
}

impl Slot {
    pub fn found(&self) -> bool {
        self.ix != u32::MAX
    }
    pub fn not_found() -> Self {
        Self {
            value: None,
            object: None,
            ix: 0,
        }
    }
}

impl GcObject for Class {
    fn visit_references(&self, tracer: &mut Tracer<'_>) {
        self.name.visit_references(tracer);
        self.super_class.visit_references(tracer);
        self.members.visit_references(tracer);
        self.fields.visit_references(tracer);
    }
}
#[macro_export]
macro_rules! define_class {
    ($isolate : expr, $ name : ident ($super_class: expr) begin $($rest:tt)*) => {{
        let mut name = Value::new_sym($isolate.intern_str(stringify!($name)));
        let mut members = vec![];
        let mut fields = vec![];
        define_class!(@parse &mut members,&mut fields,$isolate; $($rest)*);
        let cls = Class::new($isolate, name,&fields,&members,$super_class);
        cls
    }
    };
    (@parse $members: expr,$fields: expr, $isolate : expr; let $var : ident; $($rest:tt)*) => {
        $fields.push(stringify!($var));
        define_class!(@parse $members,$fields,$isolate;$($rest)*);
    };
    (@parse $members: expr,$fields: expr, $isolate : expr; static $var : ident = $x: expr; $($rest:tt)*) => {
        $members.push((stringify!($var),Value::from_lit($isolate,$x)));
        define_class!(@parse $members,$fields,$isolate;$($rest)*);
    };
    (@parse $members: expr, $fields: expr,$isolate : expr; fn $fname : ident($isolate_ : ident; $($arg: ident),* ...) $body: expr; $($rest:tt)*) => {
        {
            let mut argc = -1;
            $(
                argc-=1;
                let $arg = ();
            )*
            fn __clos($isolate_: &RCIsolate) -> Result<Value,Value> {
                let isolate: &RCIsolate = $isolate_;
                let frame = isolate.local_data().frame();

                $(

                    let $arg = frame.stack.pop().unwrap_or(Value::undefined());
                )*
                $body
            }
            let name = $isolate.intern_str(stringify!($fname));
            let mut clos = NativeClosure {
                argc,
                addr: __clos,
                cell_type: CellType::NativeFunction,
                name: Value::new_sym(name)
            };
            $members.push((stringify!($fname),Value::from($isolate.new_local(clos).to_heap())))
        }
    };
    (@parse $members: expr, $fields: expr,$isolate : expr; fn $fname : ident ($isolate_ : ident;$($arg : ident),*) $body : expr; $($rest : tt)*) => {
        {
            let mut argc = 0;
            $(
                argc+=1;
                let $arg = ();
            )*
            fn __clos($isolate_: &RCIsolate) -> Result<Value,Value> {
                let isolate: &RCIsolate = $isolate_;
                let frame = isolate.local_data().frame();

                $(

                    let $arg = frame.stack.pop().unwrap_or(Value::undefined());
                )*
                $body
            }
            let name = $isolate.intern_str(stringify!($fname));
            let mut clos = NativeClosure {
                argc,
                addr: __clos,
                cell_type: CellType::NativeFunction,
                name: Value::new_sym(name)
            };
            $members.push((stringify!($fname),Value::from($isolate.new_local(clos).to_heap())))
        }
    };
    (@parse $members: expr,$fields: expr,$isolate: expr; end ) => {

    }
}

impl CellTrait for Class {
    const TYPE: CellType = CellType::Class;
}

impl CellTrait for Instance {
    const TYPE: CellType = CellType::Instance;
}
