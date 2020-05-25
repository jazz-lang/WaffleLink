pub enum Ins {
    Int {dst: u8,i: i32},
    Undef {dst: u8},
    Null {dst: u8},
    True {dst: u8},
    False {dst: u8},
    Mov {
        dst: u8,
        src: u8 
    },
    /// Push R(src) to stack
    Push {src: u8},
    /// Pop value from stack to R(dst)
    Pop {dst: u8},
    /// Pop N elements from stack
    PopN {cnt: u32},
    /// Pop N elements from stack to array R(dst)
    PopNArray {cnt: u32,dst: u8},
    /// Clear stack
    ClearStack,
    /// Pop all values from stack to array R(dst)
    PopAllToArray{dst: u8},
    /// Pop all values from stack to array R(dst) and reverse it
    PopAllToArrayRev {dst: u8},
    /// Load upvariable to R(dst) from U(id)
    GetUpvar {dst: u8,id: u32},
    /// Store upvariable
    PutUpvar {id: u32,src: u8},
    /// Load constant C(src) to R(dst)
    GetConst {
        dst: u8,
        src: u32 
    },
    /// Load global variable at G[C(id)] to R(dst)
    GetGlobalById {
        dst: u8,
        id: u32 
    },
    /// Put value from R(src) to G[C(id)]
    PutGlobalById {
        src: u8 ,
        id: u32 
    },
    /// R(dst) = R(function)(stack.popn(argc))
    Call {
        dst: u8,
        function: u8, 
        argc: u32,
    },
    CallThis {
        dst: u8,
        function: u8,
        this: u8,
        argc: u32
    },
    VirtCallById {
        dst: u8,
        this: u8,
        method: u32,
        argc: u32
    },
    VirtCallByVal {
        dst: u8,
        this: u8,
        methodo: u8,
        argc: u32
    },
    
    Return {src: u8},
    ReturnUndef,
    Branch {
        dst: u16,
    },
    BranchConditional {
        val: u8,
        if_true: u16,
        if_false: u16,
    },

    TryCatch {
        try_block: u16,
        catch_block: u16
    },
    PopCatch,
    Throw {value: u8},
    
    GetById {
        dst: u8,
        base: u8,
        id: u32,
        fdbk: u32 
    },
    PutById {
        src: u8,
        base: u8,
        id: u32,
        fdbk: u32 
    },
    GetByVal {
        dst: u8,
        base: u8,
        val: u8 
    },
    PutByVal {
        src: u8, 
        base: u8,
        val: u8 
    },
    GetByIdx {
        dst: u8,
        base: u8,
        idx: u32,
        fdbk: u32,
    },
    PutByIdx {
        src: u8,
        base: u8,
        idx: u8,
        fdbk: u32,
    },
    
    Add {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },


    Sub {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },

    Div {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },

    Mul {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },

    Rem {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },

    Shr {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },

    Shl {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },
    UShr {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },
 
    Eq {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    }, 
    NEq {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    }, 
    Greater {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    }, 
    
 
 
    GreaterEq {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },   

 
    LessEq {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },   
    Less {
        dst: u8,
        lhs: u8,
        rhs: u8,
        fdbk: u32
    },  
    /// Hint to interpreter that we're entering loop header. 
    ///
    /// 
    /// After 100 iterations this will trigger OSR.
    LoopHint {fdbk: u32},
    Safepoint,
    IC_GetByIdOwn {
        dst: u8,
        base: u8, 
        id: u32, 
        fdbk: u32,
    },
    IC_GetByIdProto {
        dst: u8,
        base: u8,
        id: u32,
        fdbk: u32 
    },
    IC_GetByIdChain {
        dst: u8,
        base: u8,
        id: u32,
        fdbk: u32 
    },
    IC_PutByIdOwn {
        src: u8,
        base: u8,
        id: u32,
        fdbk: u32 
    },
    IC_GetByIdxOwn {
        dst: u8,
        base: u8,
        idx: u32,
        fdbk: u32 
    },
    IC_GetByIdxProto {
        dst: u8,
        base: u8,
        idx: u32,
        fdbk: u32,
    },
    IC_GetByIdxChain {
        dst: u8,
        base: u8,
        idx: u32,
        fdbk: u32 
    },
    IC_PutByIdxOwn {
        src: u8,
        base: u8,
        idx: u32,
        fdbk: u32
    },

    IC_GetByIdGeneric {
        dst: u8,
        base: u8,
        id: u32,
        fdbk: u32 
    },
    IC_PutByIdGeneric {
        src: u8,
        base: u8,
        id: u32,
        fdbk: u32 
    },
    IC_PutByIdxGeneric {
        src: u8,
        base: u8,
        idx: u32,
        fdbk: u32 
    },
    IC_GetByIdxGeneric {
        dst: u8,
        base: u8,
        idx: u32,
        fdbk: u32
    },
}   
