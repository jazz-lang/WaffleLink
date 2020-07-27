pub mod register_id;
use crate::bytecode::virtual_register::*;
use crate::bytecode::*;
use crate::object::*;
use crate::value::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ptr::NonNull;
#[derive(Default)]
pub struct Scope {
    locals: HashMap<String, VirtualRegister>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn try_get(&self, l: &str) -> Option<VirtualRegister> {
        if let Some(loc) = self.locals.get(l) {
            return Some(*loc);
        } else if let Some(parent) = &self.parent {
            parent.try_get(l)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct ByteCompiler {
    pub parent: Option<std::ptr::NonNull<Self>>,
    pub used_upvars: indexmap::IndexMap<String, i32>,
    label_true: u32,
    label_false: u32,
    label_check: u32,
    label_counter: u32,
    pub registers: Vec<VirtualRegister>,
    pub constants: Vec<Value>,
    pub constant_set: HashSet<Value>,
    pub locals: HashSet<VirtualRegister>,
    pub str_constants: HashMap<String, usize>,
    pub state: Vec<bool>,
    pub code: Vec<Ins>,
    pub scope: Box<Scope>,
}

impl ByteCompiler {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn parent(&self) -> Option<&mut Self> {
        self.parent.map(|x| unsafe { &mut *x.as_ptr() })
    }

    pub fn new_const(&mut self, val: Value) -> u32 {
        for (i, c) in self.constants.iter().enumerate() {
            if *c == val {
                return i as _;
            }
        }
        self.constants.push(val);
        self.constants.len() as u32 - 1
    }

    pub fn new_const_force(&mut self,val: Value) -> u32 {
        self.constants.push(val);
        self.constants.len() as u32 - 1
    }

    pub fn new_local(&mut self, name: impl AsRef<str>) -> VirtualRegister {
        let r = self.register_new();
        self.locals.insert(r);
        self.scope.locals.insert(name.as_ref().to_owned(), r);
        r
    }

    pub fn get_local(&mut self, name: impl AsRef<str>) -> Option<VirtualRegister> {
        self.scope.try_get(name.as_ref())
    }

    pub fn push_scope(&mut self) {
        let mut scope = Box::new(Scope::default());
        std::mem::swap(&mut scope, &mut self.scope);
        self.scope.parent = Some(scope);
    }
    pub fn new_string(&mut self, s: impl AsRef<str>) -> u32 {
       
        if let Some(ix) = self.str_constants.get(s.as_ref()) {
            return *ix as u32;
        } else {
            let ix = self.constants.len();
            self.str_constants.insert(s.as_ref().to_owned(), ix);
            self.constants.push(Value::undefined()); // fixed up later
            ix as u32
        }
    }
    pub fn pop_scope(&mut self) {
        let mut scope = Box::new(Scope::default());
        std::mem::swap(&mut self.scope, &mut scope);
        for (_, loc) in scope.locals.iter() {
            self.locals.remove(loc);
            self.state[loc.to_local() as usize] = false;
        }
        self.scope = scope.parent.expect("No parent scope was found");
    }
    pub fn allocate_regs(&mut self,count: usize) -> Vec<VirtualRegister> {
        let mut start = None;
        let mut ix = 0;
        for i in 0..self.state.len() {
            if !self.state[i] {
                ix += 1;
                if let None = start {
                    start = Some(i);
                }
                if ix - 1 == count {
                    let end = i;
                    let mut regs = vec![];
                    for i in start.unwrap()..end {
                        regs.push(virtual_register_for_local(i as _));
                    }
                    return regs;
                }
            } else {
                if let Some(_) = start {
                    break;
                }
            }
        }
        let mut regs = vec![];
        for _ in 0..count {
            regs.push(virtual_register_for_local(self.state.len() as _));
            self.state.push(false);
        }
        regs
    }
    pub fn register_new(&mut self) -> VirtualRegister {
        for i in 0..self.state.len() {
            if !self.state[i] {
                self.state[i] = true;
                return virtual_register_for_local(i as _);
            }
        }
        self.state.push(true);
        virtual_register_for_local(self.state.len() as i32 - 1)
    }
    pub fn register_push_temp(&mut self) -> VirtualRegister {
        self.register_new()
    }
    pub fn is_temp(&self, r: VirtualRegister) -> bool {
        self.locals.contains(&r) == false
    }
    pub fn register_pop(&mut self, protect: bool) -> VirtualRegister {
        self.registers
            .pop()
            .and_then(|r| {
                if r.is_local() {
                    if protect {
                        self.state[r.to_local() as usize] = true;
                    } else if self.is_temp(r) {
                        self.state[r.to_local() as usize] = false;
                    }
                }
                Some(r)
            })
            .unwrap()
    }

    pub fn cjmp(&mut self, zero: bool, r: VirtualRegister) -> impl FnOnce(&mut Self) {
        let ix = self.code.len();
        if zero {
            self.code.push(Ins::JmpIfZero(r, 0));
        } else {
            self.code.push(Ins::JmpIfNotZero(r, 0));
        }
        move |this| {
            let to = this.code.len();
            this.code[ix] = if zero {
                Ins::JmpIfZero(r, to as i32 - ix as i32)
            } else {
                Ins::JmpIfNotZero(r, to as i32 - ix as i32)
            };
        }
    }

    pub fn jmp(&mut self) -> impl FnOnce(&mut Self) {
        let ix = self.code.len();
        self.code.push(Ins::Jmp(ix as _));
        move |this| {
            let to = this.code.len() as i32 - ix as i32;
            this.code[ix] = Ins::Jmp(to);
        }
    }

    pub fn register_push(&mut self, r: VirtualRegister) {
        self.registers.push(r);
    }

    pub fn protect(&mut self, r: VirtualRegister) {
        self.state[r.to_local() as usize] = true;
    }

    pub fn unprotect(&mut self, r: VirtualRegister) {
        self.state[r.to_local() as usize] = false;
    }

    pub fn goto(&mut self, p: usize) {
        self.code.push(Ins::Jmp(p as i32 - self.code.len() as i32));
    }

    pub fn code_block(&mut self) -> Ref<CodeBlock> {
        let vm = crate::get_vm();
        let mut constants = self.constants.clone();
        for (s, ix) in self.str_constants.iter() {
            
            if s == "length" {
                constants[*ix] = vm.length;
            } else if s == "constructor" {
                constants[*ix] = vm.constructor;
            } else if s == "prototype" {
                constants[*ix] = vm.prototype;
            } else {
                
                constants[*ix] = Value::from(WaffleString::new(&mut vm.heap, s).cast());
                
            }
        }
        /*for c in constants.iter_mut() {
            if c.is_cell() {
                if c.as_cell().is_string() {
                    if c.as_cell().cast::<WaffleString>().len() == 0 {
                        *c = vm.empty_string;
                    }
                }
            }
        }*/
        // simple peephole opt
        self.code.retain(|ins| {
            if let Ins::Move(x, y) = ins {
                if x == y {
                    return false;
                }
            }
            true
        });
        let mut cb = CodeBlock::new();
        cb.constants = constants;
        cb.instructions = self.code.clone();
        cb.num_vars = self.state.len() as _;
        for _ in 0..cb.instructions.len() {
            cb.metadata.push(OpcodeMetadata::new());
        }
        vm.allocate(cb)
    }
}
use crate::frontend;
use frontend::ast::*;
use frontend::msg::*;
#[derive(Clone, Debug, PartialEq)]
pub enum Access {
    Env(i32),
    Stack(String, VirtualRegister),
    Global(i32, bool, String),
    Field(Box<Expr>, String),
    Index(i32),
    Array(Box<Expr>, Box<Expr>),
    This,
}
use std::cell::RefCell;
use std::rc::Rc;

pub struct Context {
    pub parent: Option<NonNull<Self>>,
    pub builder: ByteCompiler,
    pub fmap: Rc<RefCell<HashMap<String, VirtualRegister>>>,
    pub functions: Rc<RefCell<Vec<(Ref<CodeBlock>, VirtualRegister, String)>>>,
}

impl Context {
    pub fn new(
        fmap: Rc<RefCell<HashMap<String, VirtualRegister>>>,
        funcs: Rc<RefCell<Vec<(Ref<CodeBlock>, VirtualRegister, String)>>>,
    ) -> Self {
        Self {
            parent: None,
            fmap,
            functions: funcs,
            builder: ByteCompiler::new(),
        }
    }
    pub fn global(&self, name: &str) -> Option<VirtualRegister> {
        self.fmap.borrow().get(name).copied()
    }
    pub fn scoped<R, T: FnMut(&mut Self) -> R>(&mut self, mut f: T) -> R {
        self.builder.push_scope();
        let res = f(self);
        self.builder.pop_scope();
        res
    }
    fn access_env(&mut self, name: &str) -> Option<Access> {
        unsafe {
            let mut current = self.parent;
            let mut prev = vec![NonNull::new_unchecked(self as *mut _)];
            while let Some(mut ctxp) = current {
                let ctx = ctxp.as_mut();
                if let Some(_) = ctx.builder.get_local(name) {
                    let mut last_pos = 0;
                    for prev in prev.iter_mut() {
                        let prev: &mut Self = prev.as_mut();
                        let pos = if prev.builder.used_upvars.get(name).is_none() {
                            let pos = prev.builder.used_upvars.len();
                            prev.builder.used_upvars.insert(name.to_owned(), pos as _);
                            pos as i32
                        } else {
                            *prev.builder.used_upvars.get(name).unwrap() as i32
                        };
                        last_pos = pos;
                    }
                    return Some(Access::Env(last_pos));
                }
                current = ctx.parent;
                prev.push(ctxp);
            }
        }
        None
    }
    fn compile_function(
        &mut self,
        fpos: Position,
        params: &[Arg],
        e: &Box<Expr>,
        vname: Option<String>,
    ) -> Result<(), MsgWithPos> {
        let mut ctx = Context::new(
            Rc::new(RefCell::new(Default::default())),
            Rc::new(RefCell::new(Default::default())),
        );
        ctx.builder.new_const(Value::undefined());
        ctx.parent = Some(std::ptr::NonNull::new(self as *mut _).unwrap());
        let mut i = 0;
        for p in params.iter() {
            ctx.compile_arg(p, i, fpos)?;
            i += 1;
        }
        let c =
            VirtualRegister::new_constant_index(self.builder.new_const_force(Value::undefined()) as _);
        let c2 =
            VirtualRegister::new_constant_index(ctx.builder.new_const_force(Value::undefined()) as _);
        if vname.is_some() {
            self.fmap
                .borrow_mut()
                .insert(vname.as_ref().unwrap().to_owned(), c);
            ctx.fmap
                .borrow_mut()
                .insert(vname.as_ref().unwrap().to_owned(), c2);
        }

        ctx.compile(e)?;
        ctx.builder.code.push(Ins::Safepoint);
        let r = ctx.builder.register_pop(false);
        ctx.builder.code.push(Ins::Return(r));

        let reg = if ctx.builder.used_upvars.is_empty() {
            let dst = self.builder.register_new();
            self.builder.code.push(Ins::Move(dst, c));
            dst
        } else {
            let dst = self.builder.register_new();
            if self.builder.is_temp(dst) {
                self.builder.protect(dst);
            }
            let mut dest = vec![];
            for _ in 0..ctx.builder.used_upvars.len() {
                dest.push(self.builder.register_new());
            }
            for (i, (var, _)) in ctx.builder.used_upvars.iter().enumerate() {
                self.ident(var);
                let reg = self.builder.register_pop(false);
                if reg != dest[i] {
                    self.builder.code.push(Ins::Move(dest[i], reg));
                }
            }
            self.builder.code.push(Ins::Closure(dst, dest.len() as _));
            if self.builder.is_temp(dst) {
                self.builder.unprotect(dst);
            }
            dst
        };
        use crate::function::Function;
        let mut cb = ctx.builder.code_block();
        
        let anon = "<anonymous>".to_string();
        let f = Function::new(
            &mut crate::get_vm().heap,
            cb,
            vname.as_ref().unwrap_or(&anon),
        );
        cb.constants[c2.to_constant_index() as usize] = Value::from(f.cast());
        self.builder.constants[c.to_constant_index() as usize] = Value::from(f.cast());let mut b = String::new();
        if crate::get_vm().disasm {
            cb.dump(&mut b).unwrap();
            println!("{}", b);
        }
        self.builder.register_push(reg);
        Ok(())
    }

    fn compile_arg(&mut self, arg: &Arg, i: i32, p: Position) -> Result<(), MsgWithPos> {
        match arg {
            Arg::Ident(_, name) => {
                if self.builder.get_local(name).is_some() {
                    return Err(MsgWithPos::new(
                        p,
                        Msg::Custom(format!("argument '{}' already defined", name)),
                    ));
                }
                let r = self.builder.new_local(name);
                let arg = VirtualRegister::new_argument(i);
                self.builder.code.push(Ins::Move(r, arg));
                Ok(())
            }
            _ => todo!("NYI"),
        }
    }
    fn ident(&mut self, name: &str) {
        if let Some(loc) = self.builder.get_local(name) {
            self.builder.register_push(loc);
        } else {
            if let Some(Access::Env(x)) = self.access_env(name) {
                let dst = self.builder.register_new();
                self.builder.code.push(Ins::LoadU(dst, x as u32));
                self.builder.register_push(dst);
            } else {
                if let Some(r) = self.global(name) {
                    self.builder.register_push(r);
                } else {
                    let key = self.builder.new_string(name);
                    let dst = self.builder.register_new();
                    self.builder.code.push(Ins::LoadGlobal(dst, key));
                    self.builder.register_push(dst);
                }
            }
        }
    }

    pub fn access_get(&mut self, acc: Access) -> Result<(), MsgWithPos> {
        match acc {
            Access::Env { .. } => unreachable!(),
            Access::Stack(_, r) => {
                self.builder.register_push(r);
                Ok(())
            }
            Access::Global(x, n, name) => {
                let dst = self.builder.register_new();
                if !n {
                    self.builder
                        .code
                        .push(Ins::Move(dst, VirtualRegister::new_constant_index(x)));
                    self.builder.register_push(dst);
                    Ok(())
                } else {
                    let key = self.builder.new_string(name);
                    self.builder.code.push(Ins::LoadGlobal(dst, key));
                    self.builder.register_push(dst);
                    Ok(())
                }
            }
            Access::Field(e, f) => {
                let dst = self.builder.register_new();
                let key = self.builder.new_string(f);
                self.compile(&e)?;
                let r = self.builder.register_pop(false);
                self.builder.code.push(Ins::LoadId(dst, r, key));
                self.builder.register_push(dst);
                Ok(())
            }
            Access::This => {
                let dst = self.builder.register_new();
                self.builder.register_push(dst);
                self.builder.code.push(Ins::LoadThis(dst));
                Ok(())
            }
            _ => unimplemented!(),
        }
    }
    fn access_set(&mut self, acc: Access) -> Result<(), MsgWithPos> {
        match acc {
            Access::Stack(_, x) => {
                let v = self.builder.register_pop(false);
                self.builder.code.push(Ins::Move(x, v));
                Ok(())
            }
            Access::Global(_x, n, name) => {
                if !n {
                    panic!();
                } else {
                    let key = self.builder.new_string(name);
                    let val = self.builder.register_pop(false);
                    self.builder.code.push(Ins::StoreGlobal(val, key));
                    Ok(())
                }
            }
            Access::This => {
                let val = self.builder.register_pop(false);
                self.builder.code.push(Ins::StoreThis(val));
                Ok(())
            }
            Access::Field(e, f) => {
                let val = self.builder.register_pop(false);
                self.builder.protect(val);
                let key = self.builder.new_string(f);
                self.compile(&e)?;
                let r = self.builder.register_pop(false);
                self.builder.code.push(Ins::StoreId(r, key, val));
                self.builder.unprotect(val);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
    fn compile(&mut self, e: &Expr) -> Result<(), MsgWithPos> {
        match &e.expr {
            ExprKind::Function(vname, args, body) => {
                self.compile_function(e.pos, args, body, vname.clone())
            }
            ExprKind::New(call) => {
                match &call.expr {
                    ExprKind::Call(obj,arguments) => {
                        let mut callee;
                        self.compile(obj)?;
                        callee = self.builder.register_pop(false); let dst = self.builder.register_new();
                
                       
                        let dest = self.builder.allocate_regs(arguments.len() + 1);
                    
                        //if callee != dest[0] {
                            
                            self.builder.code.push(Ins::Move(dest[0],callee));
                            callee = dest[0];
                        //} 
                        dest.iter().for_each(|x| {
        
                            self.builder.protect(*x);
                        });
                        for (i, argument) in arguments.iter().enumerate() {
                            self.compile(argument)?;
                            let reg = self.builder.register_pop(false);
                            if reg != dest[i+1] {
                               
                                self.builder.code.push(Ins::Move(dest[i+1], reg));
                            }
                        }
                        self.builder
                            .code
                            .push(Ins::New(dst,  callee, arguments.len() as _));
                        for r in dest {
                            self.builder.unprotect(r);
                        }
                        if self.builder.is_temp(callee) && callee.is_local() {
                            self.builder.unprotect(callee);
                        }
                        
                        self.builder.register_push(dst);
                        Ok(())
                    }
                    _ => unreachable!()
                }
            }
            ExprKind::Call(callee, arguments) => {
                let (mut callee, this) = match &callee.expr {
                    ExprKind::Access(obj, f) => {
                        self.compile(obj)?;
                        let this = self.builder.register_pop(false);
                        let key = self.builder.new_string(f);
                        let callee = self.builder.register_new();
                        self.builder.code.push(Ins::LoadId(callee, this, key));
                        (callee, this)
                    }
                    _ => {
                        let this = self.builder.new_const(Value::undefined());
                        self.compile(callee)?;
                        (
                            self.builder.register_pop(false),
                            VirtualRegister::new_constant_index(this as _),
                        )
                    }
                };
                
                let dst = self.builder.register_new();
                
                if self.builder.is_temp(this) && this.is_local() {
                    self.builder.protect(this);
                }
                let dest = self.builder.allocate_regs(arguments.len() + 1);
            
                //if callee != dest[0] {
                   
                    self.builder.code.push(Ins::Move(dest[0],callee));
                    callee = dest[0];
                //} 
                dest.iter().for_each(|x| {

                    self.builder.protect(*x);
                });
                for (i, argument) in arguments.iter().enumerate() {
                    self.compile(argument)?;
                    let reg = self.builder.register_pop(false);
                    if reg != dest[i+1] {
                       
                        self.builder.code.push(Ins::Move(dest[i+1], reg));
                    }
                }
                self.builder
                    .code
                    .push(Ins::Call(dst, this, callee, arguments.len() as _));
                for r in dest {
                    self.builder.unprotect(r);
                }
                if self.builder.is_temp(callee) && callee.is_local() {
                    self.builder.unprotect(callee);
                }
                if self.builder.is_temp(this) && this.is_local() {
                    self.builder.unprotect(this);
                }
                self.builder.register_push(dst);
                Ok(())
            }
            ExprKind::Var(_, name, init) => {
                let dst = self.builder.new_local(name);
                if let Some(i) = init {
                    self.compile(&i)?;
                    let r = self.builder.register_pop(false);
                    self.builder.code.push(Ins::Move(dst, r));
                } else {
                    let c = self.builder.new_const(Value::undefined());
                    self.builder
                        .code
                        .push(Ins::Move(dst, VirtualRegister::new_constant_index(c as _)));
                }
                self.builder.register_push(dst);
                Ok(())
            }
            ExprKind::BinOp(lhs, op, rhs) => {
                let op: &str = op;
                self.compile(lhs)?;
                self.compile(rhs)?;
                let (rhs, lhs) = (
                    self.builder.register_pop(false),
                    self.builder.register_pop(false),
                );
                let dst = self.builder.register_new();
                macro_rules! b {
                    ($p: ident) => {
                        Ins::$p(dst, lhs, rhs)
                    };
                }
                let ins = match op {
                    "+" => b!(Add),
                    "-" => b!(Sub),
                    "/" => b!(Div),
                    "*" => b!(Mul),
                    "%" => b!(Mod),
                    ">>" => b!(RShift),
                    ">>>" => b!(RShift),
                    "<<" => b!(LShift),
                    ">" => b!(Greater),
                    ">=" => b!(GreaterOrEqual),
                    "<" => b!(Less),
                    "<=" => b!(LessOrEqual),
                    "==" => b!(Equal),
                    "!=" => b!(NotEqual),
                    _ => unimplemented!(),
                };
                self.builder.code.push(ins);
                self.builder.register_push(dst);
                Ok(())
            }
            ExprKind::If(cond, then, or_else) => {
                self.compile(cond)?;
                let c = self.builder.register_pop(true);
                let phi = self.builder.register_new();
                self.builder.unprotect(c);
                let co = self.builder.new_const(Value::undefined());
                self.builder
                    .code
                    .push(Ins::Move(phi, VirtualRegister::new_constant_index(co as _)));
                self.builder.protect(phi);
                let jelse = self.builder.cjmp(true, c);
                self.compile(then)?;
                let r = self.builder.register_pop(false);
                self.builder.code.push(Ins::Move(phi, r));
                if let Some(or_else) = or_else {
                    let jend = self.builder.jmp();
                    jelse(&mut self.builder);
                    self.compile(or_else)?;
                    let r = self.builder.register_pop(false);
                    self.builder.code.push(Ins::Move(phi, r));
                    jend(&mut self.builder);
                } else {
                    jelse(&mut self.builder);
                }
                self.builder.register_push(phi);
                self.builder.unprotect(phi);
                Ok(())
            }
            ExprKind::Return(Some(e)) => {
                self.compile(e)?;
                let r = self.builder.register_pop(false);
                self.builder.code.push(Ins::Return(r));
                self.builder.register_push(r);
                Ok(())
            }
            ExprKind::While(cond, body) => {
                let start = self.builder.code.len();
                self.compile(cond)?;
                let cond = self.builder.register_pop(false);
                let jend = self.builder.cjmp(true, cond);
                self.builder.code.push(Ins::LoopHint);
                self.compile(body)?;
                self.builder.goto(start);
                jend(&mut self.builder);
                Ok(())
            }
            ExprKind::Block(e) => {
                if e.is_empty() {
                    let undef = self.builder.new_const(Value::undefined());
                    /*let r = self.builder.register_new();
                    self.builder.code.push(Ins::Move(
                        r,
                    ));*/
                    self.builder
                        .register_push(VirtualRegister::new_constant_index(undef as _));
                } else {
                    let mut last = VirtualRegister::new_argument(0);
                    for e in e.iter() {
                        self.compile(e)?;
                        last = self.builder.register_pop(false);
                    }

                    self.builder.register_push(last);
                }
                Ok(())
            }
            ExprKind::Let(_, p, init) => {
                if let PatternDecl::Ident(x) = &p.decl {
                    self.compile(init)?;
                    let v = self.builder.register_pop(false);
                    let x = self.builder.new_local(x);
                    self.builder.code.push(Ins::Move(x, v));
                    self.builder.register_push(x);
                } else {
                    panic!("NYI");
                }
                Ok(())
            }
            ExprKind::ConstInt(i) => {
                let i = *i;
                let c = if i as i32 as i64 == i {
                    let c = self.builder.new_const(Value::new_int(i as i32));
                    c
                } else {
                    self.builder.new_const(Value::new_double(i as f64))
                };
                //let dst = self.builder.register_new();
                /*self.builder
                .code
                .push(Ins::Move(dst, VirtualRegister::new_constant_index(c as _)));*/
                self.builder
                    .register_push(VirtualRegister::new_constant_index(c as _));
                Ok(())
            }

            ExprKind::Assign(e, val) => {
                let acc = self.compile_access(&e.expr)?;
                self.compile(val)?;
                let val = self.builder.register_pop(false);
                self.builder.register_push(val);
                self.access_set(acc)?;
                self.builder.register_push(val);
                Ok(())
            }
            ExprKind::Ident(n) => Ok(self.ident(n)),
            ExprKind::ConstStr(s) => {
                let k = self.builder.new_string(s);
                self.builder
                    .register_push(VirtualRegister::new_constant_index(k as _));
                Ok(())
            }
            ExprKind::This => {
                let dst = self.builder.register_new();
                self.builder.code.push(Ins::LoadThis(dst));
                self.builder.register_push(dst);
                Ok(())
            }
            ExprKind::Access(_,_) => {
                let acc = self.compile_access(&e.expr)?;
                self.access_get(acc)
            }
            _ => todo!("{:?}", e),
        }
    }

    fn compile_access(&mut self, e: &ExprKind) -> Result<Access, MsgWithPos> {
        match e {
            ExprKind::Ident(i) => {
                if let Some(x) = self.builder.get_local(i) {
                    return Ok(Access::Stack(i.to_owned(), x));
                } else {
                    if let Some(acc) = self.access_env(i) {
                        return Ok(acc);
                    } else {
                        if let Some(x) = self.global(i) {
                            return Ok(Access::Global(
                                x.to_constant_index() as _,
                                true,
                                i.to_owned(),
                            ));
                        } else {
                            return Ok(Access::Global(0, false, i.to_owned()));
                        }
                    }
                }
            }
            ExprKind::Access(object,field) => {
                Ok(Access::Field(object.clone(),field.clone()))
            }
            ExprKind::ArrayIndex(object,ix) => {
                Ok(Access::Array(object.clone(),ix.clone()))
            }
            _ => unimplemented!(),
        }
    }
}
use frontend::token::*;
pub fn compile(ast: &[Box<Expr>]) -> Result<Ref<CodeBlock>, MsgWithPos> {
    //let vm = crate::get_vm();
    let ast = Box::new(Expr {
        pos: Position::new(0, 0),
        expr: ExprKind::Block(ast.to_vec()),
    });
    let mut ctx = Context::new(
        Rc::new(RefCell::new(Default::default())),
        Rc::new(RefCell::new(Default::default())),
    );
    ctx.builder.new_const(Value::undefined());
    let _ = ctx.compile(&ast)?;
    ctx.builder.code.push(Ins::Safepoint);
    let r = ctx.builder.register_pop(false);
    ctx.builder.code.push(Ins::Return(r));
    let cb = ctx.builder.code_block();
    
    Ok(cb)
}
