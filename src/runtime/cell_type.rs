/// Various builtin types
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum CellType {
    /// An regular object: prototype + table of fields
    Object = 0,
    /// Array type
    Array,
    /// UTF-8 String
    String,
    /// Big integer
    BigInt,
    /// Regular expression
    Regex,
    /// Executable function
    Function,
    Property,
}
