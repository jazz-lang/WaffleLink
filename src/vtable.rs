//! Virtual method table for Waffle objects.
//!
//!
//! We use vtable's for fast invocation of some run-time functions like `set` and `lookup`
//! on objects, but this might be slower sometimes so virtual table is not used always and it is behind feature flag.

pub(self) use super::object::*;
pub(self) use super::value::*;

pub type SetFn = fn(this: Value, key: Value, field: Value) -> Result<bool, Value>;
pub type LookupFn = fn(this: Value, key: Value) -> Result<Value, Value>;
pub type InsertFn = fn(this: Value, key: Value, field: Value) -> Result<bool, Value>;
pub type HasFn = fn(this: Value, key: Value) -> Result<bool, Value>;
pub type IsFn = fn(this: Value, object: Value) -> bool;
pub type ToStringFn = fn(this: Value, key: Value, field: Value) -> Result<String, Value>;
pub type CallFn = fn(this: Value, this_var: Value, args: &[Value]) -> Result<Value, Value>;
#[cfg(feature = "use-vtable")]
pub struct VTable {
    pub set: Option<SetFn>,
    pub lookup: Option<LookupFn>,
    pub insert: Option<InsertFn>,
    pub to_string: Option<ToStringFn>,
    pub call: Option<CallFn>,
    pub is: Option<IsFn>,
    pub has: Option<HasFn>,
    pub parent: Option<&'static VTable>,
}

#[cfg(feature = "use-vtable")]
pub mod __vtable_impl {
    use super::*;
    use crate::object::*;
    impl VTable {
        pub fn set_fn(&self) -> Option<SetFn> {
            match self.set {
                Some(f) => Some(f),
                _ => match self.parent {
                    Some(p) => p.set_fn(),
                    _ => None,
                },
            }
        }
        pub fn insert_fn(&self) -> Option<InsertFn> {
            match self.insert {
                Some(f) => Some(f),
                _ => match self.parent {
                    Some(p) => p.insert_fn(),
                    _ => None,
                },
            }
        }
        pub fn lookup_fn(&self) -> Option<LookupFn> {
            match self.lookup {
                Some(f) => Some(f),
                _ => match self.parent {
                    Some(p) => p.lookup_fn(),
                    _ => None,
                },
            }
        }

        pub fn call_fn(&self) -> Option<CallFn> {
            match self.call {
                Some(f) => Some(f),
                _ => match self.parent {
                    Some(p) => p.call_fn(),
                    _ => None,
                },
            }
        }
    }

    /// Insert field to an object.
    pub fn object_set_insert_fn(this: Value, key: Value, field: Value) -> Result<bool, Value> {
        Ok(true)
    }

    impl<T: WaffleCellTrait> WaffleCellPointer<T> {
        pub fn set(&self, key: Value, field: Value) -> Result<bool, Value> {
            match self.value().header().vtable.set_fn() {
                Some(fun) => fun(Value::from(self.to_cell()), key, field),
                _ => Ok(false),
            }
        }

        pub fn insert(&self, key: Value, field: Value) -> Result<bool, Value> {
            match self.value().header().vtable.insert_fn() {
                Some(fun) => fun(Value::from(self.to_cell()), key, field),
                _ => Ok(false),
            }
        }

        pub fn lookup(&self, key: Value) -> Result<Value, Value> {
            match self.value().header().vtable.lookup_fn() {
                Some(fun) => fun(Value::from(self.to_cell()), key),
                _ => Ok(Value::undefined()),
            }
        }

        pub fn call(&self, args: &[Value], this: Value) -> Result<Value, Value> {
            match self.value().header().vtable.call_fn() {
                Some(fun) => fun(Value::from(self.to_cell()), this, args),
                _ => todo!("TODO: Throw error"),
            }
        }
    }
}

#[cfg(not(feature = "use-vtable"))]
pub mod __rt_dispatch_impl {
    use super::*;

    impl<T: WaffleCellTrait> WaffleCellPointer<T> {
        pub fn set(&self, key: Value, field: Value) -> Result<bool, Value> {
            match self.type_of() {
                WaffleType::Object => {
                    let object = self.try_as_object().unwrap();
                    //object.value_mut().map.insert(key, field);
                    Ok(true)
                }
                WaffleType::Array => {
                    if key.is_number() {
                        let mut ix = key.to_number().floor() as isize;
                        let array = self.try_as_array().unwrap();
                        if ix < 0 {
                            ix = array.value().len() as isize + ix;
                        }
                        if ix >= array.value().len() as isize {
                            return Ok(false);
                        }
                        array.value_mut()[ix as usize] = field;
                        return Ok(true);
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }

        pub fn lookup(&self, key: Value) -> Result<Option<Value>, Value> {
            match self.type_of() {
                WaffleType::Object => {
                    let object = self.try_as_object().unwrap();
                    object.value().lookup(key)
                }
                WaffleType::Array => {
                    if key.is_number() {
                        let mut ix = key.to_number().floor() as isize;
                        let array = self.try_as_array().unwrap();
                        if ix < 0 {
                            ix = array.value().len() as isize + ix;
                        }
                        if ix >= array.value().len() as isize {
                            return Ok(None);
                        }
                        return Ok(Some(array.value()[ix as usize]));
                    }
                    Ok(None)
                }
                _ => unimplemented!(),
            }
        }
    }
}

#[cfg(feature = "use-vtable")]
pub use __vtable_impl::*;

#[cfg(not(feature = "use-vtable"))]
pub use __rt_dispatch_impl::*;
