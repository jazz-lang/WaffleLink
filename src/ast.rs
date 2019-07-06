#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(name: &str, line: usize, column: usize) -> Location {
        Location {
            file: name.to_owned(),
            line,
            column,
        }
    }
}

use std::fmt;

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = if self.file.starts_with("./")
            || self.file.starts_with(".")
            || self.file.starts_with("/")
        {
            self.file.clone()
        } else {
            format!("./{}", self.file)
        };

        write!(f, "{}:{}:{}", name, self.line, self.column)
    }
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub pos: Location,
    pub kind: ExprKind,
    pub id: usize,
}

#[derive(Clone, Debug, Hash, Eq)]
pub struct Type {
    pub pos: Location,
    pub kind: TypeKind,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Type {
    pub const fn new(pos: Location, kind: TypeKind) -> Type {
        Type { pos, kind }
    }

    pub fn is_array_wo_len(&self) -> bool {
        match &self.kind {
            TypeKind::Array(_, None) => true,
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match &self.kind {
            TypeKind::Void => true,
            _ => false,
        }
    }

    pub fn is_basic(&self) -> bool {
        match &self.kind {
            TypeKind::Basic(_) => true,
            _ => false,
        }
    }

    pub fn is_basic_name(&self, name: &str) -> bool {
        match &self.kind {
            TypeKind::Basic(name_) => name == name_,
            _ => false,
        }
    }

    pub fn is_option(&self) -> bool {
        match &self.kind {
            TypeKind::Optional(_) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match &self.kind {
            TypeKind::Array(_, _) => true,
            _ => false,
        }
    }

    pub fn get_subty(&self) -> Option<&Type> {
        match &self.kind {
            TypeKind::Array(subty, _) => Some(subty),
            TypeKind::Pointer(subty) => Some(subty),
            TypeKind::Optional(subty) => Some(subty),
            _ => None,
        }
    }

    pub fn is_basic_names(&self, names: &[&str]) -> bool {
        match &self.kind {
            TypeKind::Basic(name_) => {
                for name in names.iter() {
                    if name == name_ {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn is_pointee(&self, to: &TypeKind) -> bool {
        match &self.kind {
            TypeKind::Pointer(ptr) => ptr.kind == *to,
            _ => false,
        }
    }

    pub fn is_interface(&self) -> bool {
        match &self.kind {
            TypeKind::Interface(_, _) => true,
            _ => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        match &self.kind {
            TypeKind::Structure(_, _) => true,
            _ => false,
        }
    }

    pub fn get_pointee(&self) -> Option<&Type> {
        match &self.kind {
            TypeKind::Pointer(pointee) => Some(pointee),
            _ => None,
        }
    }

    pub fn is_pointer(&self) -> bool {
        match &self.kind {
            TypeKind::Pointer(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Clone, Debug, Hash)]
pub enum TypeKind {
    Void,
    Basic(String),
    Pointer(Box<Type>),
    Array(Box<Type>, Option<i32>),
    Function(Box<Type>, Vec<Box<Type>>),
    Structure(String, Vec<(String, Box<Type>)>),
    Interface(String, Vec<(String, Vec<(String, Box<Type>)>, Box<Type>)>),
    Optional(Box<Type>),
}
impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Void => write!(f, "void"),
            TypeKind::Basic(name) => write!(f, "{}", name),
            TypeKind::Pointer(ptr) => write!(f, "*{}", ptr),
            TypeKind::Array(subty, len) => {
                if len.is_none() {
                    write!(f, "{}[]", subty)
                } else {
                    write!(f, "{}[{}]", subty, len.unwrap())
                }
            }
            TypeKind::Structure(name, _) => write!(f, "{}", name),
            TypeKind::Interface(name, _) => write!(f, "{}", name),
            TypeKind::Function(returns, params) => {
                let mut string = String::new();
                string.push('(');
                for (i, x) in params.iter().enumerate() {
                    string.push_str(&format!("{}", x));
                    if i != params.len() - 1 {
                        string.push(',');
                    }
                }
                string.push(')');

                string.push_str(&format!(" {}", returns));
                write!(f, "{}", string)
            }
            TypeKind::Optional(opt) => write!(f, "{}?", opt),
        }
    }
}

impl Eq for TypeKind {}

impl PartialEq for TypeKind {
    fn eq(&self, other: &TypeKind) -> bool {
        match (self, other) {
            (TypeKind::Void, TypeKind::Void) => true,
            (TypeKind::Basic(name), TypeKind::Basic(name2)) => name == name2,
            (TypeKind::Pointer(pointee1), TypeKind::Pointer(pointee2)) => {
                pointee1.kind == pointee2.kind
            }
            (TypeKind::Array(array1, len1), TypeKind::Array(array2, len2)) => {
                array1.kind == array2.kind && len1 == len2
            }
            (TypeKind::Function(returns, params), TypeKind::Function(returns2, params2)) => {
                returns == returns2 && params == params2
            }
            (TypeKind::Structure(name, fields), TypeKind::Structure(name2, fields2)) => {
                name == name2 && fields == fields2
            }
            (TypeKind::Interface(name, _), TypeKind::Interface(name2, _)) => name == name2,
            (TypeKind::Optional(opt1), TypeKind::Optional(opt2)) => opt1 == opt2,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Element {
    Const(Constant),
    Var(ExternVar),
    Import(Import),
    Struct(Struct),
    Func(Function),
    Interface(Interface),
    Module(String, Vec<Element>),
}

#[derive(Clone, Debug)]
pub struct ExternVar {
    pub name: String,
    pub ty: Box<Type>,
    pub pos: Location,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub name: String,
    pub pos: Location,
}

#[derive(Clone, Debug)]
pub struct Constant {
    pub id: usize,
    pub name: String,
    pub ty: Option<Box<Type>>,
    pub value: Box<Expr>,
    pub pos: Location,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub id: usize,
    pub name: String,
    pub loc: Location,
    pub fields: Vec<(Location, String, Box<Type>)>,
}

#[derive(Clone, Debug)]
pub struct Interface {
    pub id: usize,
    pub name: String,
    pub pos: Location,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub id: usize,
    pub name: String,
    pub pos: Location,
    pub this: Option<(String, Box<Type>)>,
    pub parameters: Vec<(String, Box<Type>)>,
    pub returns: Box<Type>,
    pub body: Option<Box<StmtKind>>,
    pub external: bool,
    pub internal: bool,
    pub variadic: bool,
}

impl Function {
    pub fn mangle_name(&self) -> String {
        if self.external || self.internal  || (&self.name == "main" && self.this.is_none() ) {
            return self.name.clone();
        } else {
            let mut name = String::new();
            name.push_str("waffle_");
            if self.this.is_some() {
                name.push_str(&format!(
                    "{}_",
                    if self.this.as_ref().unwrap().1.is_pointer() { self.this.as_ref().unwrap().1.get_subty().unwrap() }
                    else { &self.this.as_ref().unwrap().1 }
                ));
            }

            name.push_str(&self.name);

            return name;
        }
    }
}

use crate::lexer::{FloatSuffix, IntSuffix};

#[derive(Clone, Debug)]
pub enum ExprKind {
    Paren(Box<Expr>),
    Integer(u64, IntSuffix),
    UInt(u64),
    Float(f64, FloatSuffix),
    String(String),
    Bool(bool),
    Character(char),
    Identifier(String),
    Deref(Box<Expr>),
    AddrOf(Box<Expr>),
    Null,
    Conv(Box<Expr>, Box<Type>),
    Member(Box<Expr>, String),
    /// Call expression:
    /// ```go
    /// factorial(5)
    ///
    /// foo.getPoint().getX()
    ///
    /// ```
    ///
    ///
    Call(String, Option<Box<Expr>>, Vec<Box<Expr>>),
    Unary(String, Box<Expr>),
    /// Binary operation: `x <op> y
    Binary(String, Box<Expr>, Box<Expr>),
    Array(Vec<Box<Expr>>),
    /// Subscript: `array[index]`
    Subscript(Box<Expr>, Box<Expr>),
    SizeOf(Box<Type>),
    CString(String),
    /// Struct creation:
    /// ```rust
    /// x: Foo = Foo {bar: 0,baz: 42}
    /// ```
    StructConstruct(String, Vec<(String, Box<Expr>)>),
    /// Assign value to variable or create new variable:
    /// ```go
    ///  x := 9
    ///  x = x + 1  // assign new value to x
    /// ```
    Assign(Box<Expr>, Box<Expr>),
    Undefined,
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    /// Declare variable with type:
    /// ```go
    /// foo: int = 42
    /// bar := 10 * 10
    /// ```
    VarDecl(String, Option<Box<Type>>, Option<Box<Expr>>),
    /// If statement:
    /// ```go
    /// if condition {
    ///     ...
    /// } else if condition_2 {
    ///     ...
    /// } else {
    ///     ...
    /// }
    ///
    /// ```
    If(Box<Expr>, Box<StmtKind>, Option<Box<StmtKind>>),
    /// While statement:
    /// ```go
    /// while condition {
    ///     ...
    /// }
    /// ```
    While(Box<Expr>, Box<StmtKind>),
    /// For statement:
    /// ```go
    /// for i := 0, i < 100, i++ {
    ///     println(i.toStr())
    /// }
    ///
    /// ```
    ///
    For(Box<StmtKind>, Box<Expr>, Box<Expr>, Box<StmtKind>),
    /// Switch statement:
    /// ```go
    /// switch x {
    ///     1 -> {
    ///         print("one")
    ///     }
    ///     2 -> {
    ///         print("two")
    ///     }
    ///     default:
    ///         print("unknown")
    /// }
    /// ```
    Switch(Box<Expr>, Vec<(Box<Expr>, Box<StmtKind>)>),
    Expr(Box<Expr>),
    Block(Vec<Box<StmtKind>>),
    Return(Option<Box<Expr>>),
    Break,
    Continue,
}
