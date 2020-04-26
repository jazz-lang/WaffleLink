#[repr(u32)]
pub enum Attribute {
    None = 0,
    Writable = 1,
    Enumerable = 2,
    Configurable = 4,
    Data = 8,
    Accessor = 16,
    Empty = 32,
    UndefWritable = 64,
    UndefEnumerable = 128,
    UndefConfigurable = 256,
    UndefValue = 512,
    UndefGetter = 1024,
    UndefSetter = 2048,
}

use Attribute::*;

pub struct Attributes;
impl Attributes {
    pub const TYPE_MASK: u32 = Attribute::Data as u32 | Attribute::Accessor as u32;
    pub const DATA_ATTR_MASK: u32 = Attribute::Data as u32
        | Attribute::Writable as u32
        | Attribute::Enumerable as u32
        | Attribute::Configurable as u32;
    pub const ACCESSOR_ATTR_MASK: u32 =
        Attribute::Accessor as u32 | Attribute::Enumerable as u32 | Attribute::Configurable as u32;
    pub const DEFAULT: u32 = UndefWritable as u32
        | UndefEnumerable as u32
        | UndefConfigurable as u32
        | UndefValue as u32
        | UndefGetter as u32
        | UndefSetter as u32;

    pub const UNDEFS: u32 = Empty as u32 | Self::DEFAULT;
    pub const BOTH: u32 = Configurable as u32 | Enumerable as u32;

    pub fn is_stored(attrs: u32) -> bool {
        if (attrs & Self::UNDEFS) != 0 {
            return false;
        }
        if (attrs & Data as u32) != 0 {
            return (attrs & Accessor as u32) == 0;
        }

        if (attrs & Accessor as u32) != 0 {
            return (attrs & Writable as u32) == 0;
        }
        false
    }

    pub const fn remove_undefs(x: u32) -> u32 {
        return x & !(Self::UNDEFS);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AttrsExternal {
    attributes: u32,
}

impl AttrsExternal {
    pub const fn new(x: u32) -> Self {
        Self { attributes: x }
    }

    pub const fn empty() -> Self {
        Self {
            attributes: None as u32,
        }
    }

    pub const fn ty(&self) -> u32 {
        self.attributes & Attributes::TYPE_MASK
    }

    pub const fn is_enumerable(&self) -> bool {
        self.attributes & Enumerable as u32 != 0
    }

    pub const fn is_enumerable_absent(&self) -> bool {
        self.attributes & UndefEnumerable as u32 != 0
    }

    pub fn set_enumerable(&mut self, val: bool) {
        if val {
            self.attributes = (self.attributes & !(UndefEnumerable as u32)) & Enumerable as u32;
        } else {
            self.attributes = (self.attributes & !(UndefEnumerable as u32)) & !(Enumerable as u32);
        }
    }

    pub const fn is_configurable(&self) -> bool {
        self.attributes & Configurable as u32 != 0
    }

    pub const fn is_configurable_absent(&self) -> bool {
        self.attributes & UndefConfigurable as u32 != 0
    }

    pub fn set_configurable(&mut self, val: bool) {
        if val {
            self.attributes = (self.attributes & !(UndefConfigurable as u32)) & Configurable as u32;
        } else {
            self.attributes =
                (self.attributes & !(UndefConfigurable as u32)) & !(Configurable as u32);
        }
    }

    pub const fn is_data(&self) -> bool {
        self.attributes & Data as u32 != 0
    }
    pub fn set_data(&mut self) {
        self.attributes &= !(Accessor as u32);
        self.attributes |= Data as u32;
    }
    pub const fn is_accessor(&self) -> bool {
        self.attributes & Accessor as u32 != 0
    }

    pub fn set_accessor(&mut self) {
        self.attributes &= !(Data as u32 | Writable as u32);
        self.attributes |= Accessor as u32;
    }

    pub const fn is_generic(&self) -> bool {
        !(self.attributes & (Data as u32 | Accessor as u32 | Empty as u32)) != 0
    }

    pub const fn is_empty(&self) -> bool {
        self.attributes & Empty as u32 != 0
    }

    pub const fn is_writable(&self) -> bool {
        self.attributes & Writable as u32 != 0
    }

    pub const fn is_writable_absent(&self) -> bool {
        self.attributes & UndefWritable as u32 != 0
    }

    pub fn set_writable(&mut self, val: bool) {
        if val {
            self.attributes = (self.attributes & !(UndefWritable as u32)) & Writable as u32;
        } else {
            self.attributes = (self.attributes & !(UndefWritable as u32)) & !(Writable as u32);
        }
    }

    pub const fn is_value_absent(&self) -> bool {
        self.attributes & UndefValue as u32 != 0
    }

    pub const fn is_getter_absent(&self) -> bool {
        self.attributes & UndefGetter as u32 != 0
    }

    pub const fn is_setter_absent(&self) -> bool {
        self.attributes & UndefSetter as u32 != 0
    }

    pub fn is_absent(&self) -> bool {
        self.is_configurable_absent() && self.is_enumerable_absent() && self.is_generic()
    }

    pub const fn is_default(&self) -> bool {
        let def = Configurable as u32 | Enumerable as u32 | Data as u32 | Writable as u32;
        self.attributes & def == def
    }

    pub fn raw(&self) -> u32 {
        self.attributes
    }

    fn fill_enumerable_and_configurable(&mut self) {
        if self.is_configurable_absent() {
            self.attributes &= !(UndefConfigurable as u32);
        }
        if self.is_enumerable_absent() {
            self.attributes &= !(UndefEnumerable as u32);
        }
    }
}

pub struct Safe(pub AttrsExternal);

impl Safe {
    pub fn not_found() -> Self {
        Self(AttrsExternal::new(None as u32))
    }
    pub fn empty() -> Self {
        Self(AttrsExternal::empty())
    }
    pub fn is_not_found(&self) -> bool {
        self.0.raw() == None as u32
    }
    pub fn new(attr: u32) -> Self {
        Self(AttrsExternal::new(Attributes::remove_undefs(attr)))
    }
    pub fn from_ext(x: AttrsExternal) -> Self {
        Self(AttrsExternal::new(Attributes::remove_undefs(x.raw())))
    }

    pub fn create_data(mut x: AttrsExternal) -> Self {
        x.fill_enumerable_and_configurable();
        x.set_data();
        if x.is_writable_absent() {
            x.set_writable(false);
        }
        Self::from_ext(x)
    }

    pub fn create_accessor(mut x: AttrsExternal) -> Self {
        x.fill_enumerable_and_configurable();
        x.set_accessor();
        Self::from_ext(x)
    }
}

use std::ops::{Deref, DerefMut};

impl Deref for Safe {
    type Target = AttrsExternal;
    fn deref(&self) -> &AttrsExternal {
        &self.0
    }
}

impl DerefMut for Safe {
    fn deref_mut(&mut self) -> &mut AttrsExternal {
        &mut self.0
    }
}

pub struct AttrObject;

impl AttrObject {
    pub fn data() -> Safe {
        Safe::create_data(AttrsExternal::new(
            Writable as u32 | Enumerable as u32 | Configurable as u32,
        ))
    }

    pub fn accessor() -> Safe {
        Safe::create_data(AttrsExternal::new(Enumerable as u32 | Configurable as u32))
    }
}

pub struct AttrString;

impl AttrString {
    pub fn length() -> Safe {
        Safe::create_data(AttrsExternal::new(None as u32))
    }

    pub fn indexed() -> Safe {
        Safe::create_data(AttrsExternal::new(Enumerable as u32))
    }
}
