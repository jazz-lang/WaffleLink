#[derive(Copy, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Number(f64),
    Boolean(bool),
    Nil,
    Cell,
}

pub struct Cell {}
