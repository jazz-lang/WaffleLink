use crate::common::rc::*;
use crate::frontend::ast::*;

pub enum HIRNodeKind {
    Undef,
    Null,
    This,
    True,
    False,
    ConstI {
        imm: i64,
    },
    ConstF {
        imm: i64,
    },
    ConstStr {
        id: usize,
    },
    Variable {
        id: usize,
    },
    Get {
        value: Rc<HIRNode>,
    },
    Set {
        src: Rc<HIRNode>,
        value: Rc<HIRNode>,
    },
    BinOp {
        op: BinOp,
        lhs: Rc<HIRNode>,
        rhs: Rc<HIRNode>,
    },
    Neg {
        value: Rc<HIRNode>,
    },
    Not {
        value: Rc<HIRNode>,
    },
    Throw {
        src: Rc<HIRNode>,
    },

    LoadProperty {
        base: Rc<HIRNode>,
        key: Rc<HIRNode>,
    },
    StoreProperty {
        base: Rc<HIRNode>,
        key: Rc<HIRNode>,
    },
    NewObject,
    New {
        ctor: Rc<HIRNode>,
        arguments: Rc<HIRNode>,
    },
    Call {
        this: Rc<HIRNode>,
        func: Rc<HIRNode>,
        arguments: Rc<HIRNode>,
    },
    Move {
        src: Rc<HIRNode>,
    },
    CloseEnvironment {
        value: Rc<HIRNode>,
        arguments: Rc<HIRNode>,
    },
    LoadUpvar {
        x: usize,
    },
}

pub struct HIRNode {
    pub kind: HIRNodeKind,
    pub uses: Vec<Rc<HIRNode>>,
}

pub enum BinOp {
    Add,
    Sub,
    Div,
    Mul,
    Rem,
    Shr,
    Shl,
    UShr,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Eq,
    BitAnd,
    BitOr,
    BitXor,
}
