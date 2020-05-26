use super::cell::*;
use super::transition_map::*;
use super::value::*;
use crate::common::DerefPointer;
use crate::common::*;
use crate::gc::*;
pub type Get = fn(this: Handle<Cell>, key: Value) -> Result<Option<Value>, Value>;
pub type Set = fn(this: Handle<Cell>, key: Value, val: Value) -> bool;
pub struct VTable {
    pub get: Option<Get>,
    pub set: Option<Set>,
    /// Used for inline caching.
    pub get_table: Option<fn(this: Handle<Cell>) -> DerefPointer<Table>>,
    pub get_class: Option<fn(this: Handle<Cell>) -> Option<Handle<Class>>>,

    pub parent: Option<&'static VTable>,
}
