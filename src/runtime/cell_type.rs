/// Various builtin types
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum CellType {
    Function,
    String,
    Class,
    Instance,
    Proto,
    Array,
    Map,
    Module,
    ComObj,
    NativeFunction,
    Closure,
    NativeClosure,
}
