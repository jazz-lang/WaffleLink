use super::token::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum PatternDecl {
    Ident(String),
    ConstInt(i64),
    ConstFloat(f64),
    ConstChar(char),
    ConstStr(String),
    Cons(Box<Pattern>, Box<Pattern>),
    EmptyList,
    Tuple(Vec<Box<Pattern>>),
    Record(Vec<(String, Option<Box<Pattern>>)>),
    Array(Vec<Box<Pattern>>),
    Pass,
    Rest,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    /// `function foo (x,y / *Ident*/ )`
    Ident(bool, String),
    /// `function foo ( {x,y} /* Record */ )`
    Record(Vec<String>),
    /// `function foo ( [x,y] /* Array */ )`
    Array(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pattern {
    pub decl: PatternDecl,
    pub pos: Position,
}

#[derive(Clone, PartialEq)]
pub struct Expr {
    pub pos: Position,
    pub expr: ExprKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Assign(Box<Expr>, Box<Expr>),
    BinOp(Box<Expr>, String, Box<Expr>),
    Unop(String, Box<Expr>),
    Access(Box<Expr>, String),
    Ident(String),
    Function(Option<String>, Vec<Arg>, Box<Expr>),
    Lambda(Vec<Arg>, Box<Expr>),
    Match(Box<Expr>, Vec<(Box<Pattern>, Option<Box<Expr>>, Box<Expr>)>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    ConstInt(i64),
    ConstChar(char),
    ConstStr(String),
    New(Box<Expr>),

    ConstFloat(f64),
    Object(Vec<(Box<Expr>, Box<Expr>)>),
    Var(bool, String, Option<Box<Expr>>),
    Let(bool, Box<Pattern>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    Block(Vec<Box<Expr>>),
    Return(Option<Box<Expr>>),
    Call(Box<Expr>, Vec<Box<Expr>>),
    Nil,
    Try(Box<Expr>,String,Box<Expr>),
    Throw(Box<Expr>),
    ConstBool(bool),
    NewObject(Vec<(String, Option<Box<Expr>>)>),
    Array(Vec<Box<Expr>>),
    ArrayIndex(Box<Expr>, Box<Expr>),
    Class(String, Option<Box<Expr>>, Vec<Box<Expr>>),
    Tuple(Vec<Box<Expr>>),
    This,
}

use std::fmt;

impl Expr {
    pub fn is_access(&self) -> bool {
        if let ExprKind::Access(_, _) = self.expr {
            return true;
        };
        false
    }

    pub fn is_binop(&self) -> bool {
        if let ExprKind::BinOp(_, _, _) = self.expr {
            return true;
        };
        false
    }

    pub fn is_binop_cmp(&self) -> bool {
        if let ExprKind::BinOp(_, ref op, _) = self.expr {
            let op: &str = op;
            match op {
                ">" | "<" | ">=" | "<=" | "==" | "!=" => return true,
                _ => return false,
            }
        }
        return false;
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.expr)
    }
}
