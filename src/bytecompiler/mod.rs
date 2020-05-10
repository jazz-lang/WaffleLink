pub mod graph_coloring;
pub mod interference_graph;
pub mod loopanalysis;

pub mod strength_reduction;
use crate::bytecode::*;
use crate::runtime::*;
use cgc::api::*;
use def::*;
use std::collections::HashMap;
use value::*;
use virtual_reg::*;
pub struct ByteCompiler {
    pub code: Rooted<CodeBlock>,
    vid: i32,
    args: i32,
    free_args: std::collections::VecDeque<VirtualRegister>,
    current: u32,
    strings: HashMap<VirtualRegister, String>,
    pub vars: HashMap<String, VirtualRegister>,
}

impl ByteCompiler {
    pub fn new_constant(&mut self, val: Value) -> VirtualRegister {
        self.code.new_constant(val)
    }
    pub fn new_string(&mut self, s: impl AsRef<str>) -> VirtualRegister {
        let x = self.code.new_constant(Value::undefined());
        self.strings.insert(x, s.as_ref().to_string());
        x
    }
    pub fn new(rt: &mut Runtime) -> Self {
        let block = rt.allocate(CodeBlock {
            constants: Default::default(),
            constants_: vec![],
            arg_regs_count: 0,
            tmp_regs_count: 255,
            hotness: 0,
            code: Vec::new(),
            cfg: None,
            loopanalysis: None,
            jit_stub: None,
        });
        Self {
            code: block,
            vid: 256,
            args: 0,
            free_args: Default::default(),
            current: 0,
            strings: Default::default(),
            vars: Default::default(),
        }
    }

    pub fn cjmp(&mut self, val: VirtualRegister) -> (impl FnMut(), impl FnMut()) {
        let p = self.current;
        let p2 = self.code.code[p as usize].code.len();
        self.code.code[p as usize].code.push(Ins::Jump { dst: 0 }); // this is replaced later.
        let this = unsafe { &mut *(self as *mut Self) };
        let this2 = unsafe { &mut *(self as *mut Self) };
        (
            move || {
                this.code.code[p as usize].code[p2 as usize] = Ins::JumpConditional {
                    cond: val,
                    if_true: this.current,
                    if_false: 0,
                };
            },
            move || {
                if let Ins::JumpConditional {
                    cond: _,
                    if_true: _,
                    if_false,
                } = &mut this2.code.code[p as usize].code[p2 as usize]
                {
                    *if_false = this2.current;
                } else {
                    this2.code.code[p as usize].code[p2 as usize] = Ins::JumpConditional {
                        cond: val,
                        if_true: 0,
                        if_false: this2.current,
                    };
                }
            },
        )
    }

    pub fn fallthrough(&mut self) {
        let mut j = self.jmp();
        let bb = self.create_new_block();
        self.switch_to_block(bb);
        j();
    }
    pub fn jmp(&mut self) -> impl FnMut() {
        let p = self.current;
        let p2 = self.code.code[p as usize].code.len();
        self.code.code[p as usize].code.push(Ins::Jump { dst: 0 }); // this is replaced later.
        let this = unsafe { &mut *(self as *mut Self) };
        move || this.code.code[p as usize].code[p2 as usize] = Ins::Jump { dst: this.current }
    }

    pub fn create_new_block(&mut self) -> u32 {
        let id = self.code.code.len() as u32;
        let bb = BasicBlock::new(id);
        self.code.code.push(bb);
        id
    }

    pub fn switch_to_block(&mut self, id: u32) {
        self.current = id;
    }
    pub fn emit(&mut self, ins: Ins) {
        self.code.code[self.current as usize].code.push(ins);
    }
    pub fn mov(&mut self, to: VirtualRegister, from: VirtualRegister) {
        self.emit(Ins::Mov { dst: to, src: from });
    }

    pub fn vreg(&mut self) -> VirtualRegister {
        let x = self.vid;
        self.vid += 1;
        VirtualRegister::tmp(x as _)
    }
    pub fn areg(&mut self) -> VirtualRegister {
        if let Some(reg) = self.free_args.pop_front() {
            return reg;
        } else {
            let x = self.args;
            self.args += 1;
            self.free_args.push_back(VirtualRegister::argument(x as _));
            self.areg()
        }
    }

    pub fn do_call(
        &mut self,
        func: VirtualRegister,
        this: VirtualRegister,
        arguments: &[VirtualRegister],
    ) -> VirtualRegister {
        let dst = self.vreg();
        if arguments.is_empty() {
            self.emit(Ins::CallNoArgs {
                function: func,
                this,
                dst
            });
            return dst;
        }
        let mut used = std::collections::VecDeque::new();
        for arg in arguments.iter() {
            if let Some(reg) = self.free_args.pop_front() {
                self.emit(Ins::Mov {
                    dst: reg,
                    src: *arg,
                });
                used.push_back(reg);
            } else {
                let reg = self.areg();
                used.push_back(reg);
                self.emit(Ins::Mov {
                    dst: reg,
                    src: *arg,
                });
            }
        }

        self.emit(Ins::Call {
            dst,
            function: func,
            this,
            begin: *used.front().unwrap(),
            end: *used.back().unwrap(),
        });
        while let Some(x) = used.pop_front() {
            self.free_args.push_front(x);
        }

        dst
    }

    pub fn close_env(
        &mut self,
        func: VirtualRegister,
        arguments: &[VirtualRegister],
    ) -> VirtualRegister {
        let dst = self.vreg();
        let mut used = std::collections::VecDeque::new();
        for arg in arguments.iter() {
            if let Some(reg) = self.free_args.pop_front() {
                self.emit(Ins::Mov {
                    dst: reg,
                    src: *arg,
                });
                used.push_back(reg);
            } else {
                let reg = self.areg();
                used.push_back(reg);
                self.emit(Ins::Mov {
                    dst: reg,
                    src: *arg,
                });
            }
        }

        self.emit(Ins::CloseEnv {
            dst,
            function: func,
            begin: *used.front().unwrap(),
            end: *used.back().unwrap(),
        });
        while let Some(x) = used.pop_front() {
            self.free_args.push_front(x);
        }

        dst
    }
    pub fn def_var(&mut self, name: String, val: VirtualRegister) {
        self.vars.insert(name, val);
    }
    pub fn has_var(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    pub fn get_var(&self, name: &str) -> VirtualRegister {
        *self.vars.get(name).unwrap()
    }
    pub fn set_var(&mut self, name: String, new: VirtualRegister) {
        self.def_var(name, new);
    }
    pub fn finish(mut self, rt: &mut Runtime) -> Rooted<CodeBlock> {
        self.code.arg_regs_count = self.args as _;
        self.code.tmp_regs_count = 255;

        strength_reduction::regalloc_and_reduce_strength(self.code.to_heap(), rt);
        self.code
    }
}
use crate::frontend;
use deref_ptr::*;
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

pub struct Functions {}

#[derive(Clone)]
pub struct LoopControlInfo {
    pub break_point: u16,
    pub continue_point: u16,
}

pub struct Context<'a> {
    pub rt: &'a mut Runtime,
    pub builder: ByteCompiler,
    pub parent: Option<DerefPointer<Self>>,
    pub fmap: Rc<RefCell<HashMap<String, VirtualRegister>>>,
    pub functions: Rc<RefCell<Vec<(Rooted<CodeBlock>, VirtualRegister, String)>>>,
    pub used_upvars: indexmap::IndexMap<String, i32>,
}

impl<'a> Context<'a> {
    pub fn global(&self, name: &str) -> Option<VirtualRegister> {
        self.fmap.borrow().get(name).copied()
    }
    pub fn scoped<R, T: FnMut(&mut Self) -> R>(&mut self, mut f: T) -> R {
        let prev = self.builder.vars.clone();
        let res = f(self);
        self.builder.vars = prev;
        res
    }

    pub fn access_get(&mut self, acc: Access) -> Result<VirtualRegister, MsgWithPos> {
        match acc {
            Access::Env { .. } => unreachable!(),
            Access::Stack(_, r) => return Ok(r),
            Access::Global(x, n, name) => {
                let dst = self.builder.vreg();
                if !n {
                    self.builder.emit(Ins::Mov {
                        dst,
                        src: VirtualRegister::constant(x),
                    });
                    return Ok(dst);
                } else {
                    let key = self.builder.new_string(name);
                    self.builder.emit(Ins::LoadGlobal { dst, name: key });
                    return Ok(dst);
                }
            }
            Access::Field(e, f) => {
                let dst = self.builder.vreg();
                let key = self.builder.new_string(f);
                let o = self.compile(&e)?;
                self.builder.emit(Ins::GetById {
                    dst: dst,
                    base: o,
                    id: key,
                    fdbk: 0,
                });
                return Ok(dst);
            }
            Access::This => {
                let dst = self.builder.vreg();
                self.builder.emit(Ins::LoadThis { dst });
                Ok(dst)
            }
            _ => unimplemented!(),
        }
    }
    fn access_env(&mut self, name: &str) -> Option<Access> {
        let mut current = self.parent;
        let mut prev = vec![DerefPointer::new(self)];
        while let Some(mut ctx) = current {
            let ctx: &mut Context = &mut *ctx;
            if ctx.builder.has_var(name) {
                let mut last_pos = 0;
                for prev in prev.iter_mut().rev() {
                    let pos = if !prev.used_upvars.contains_key(name) {
                        let pos = prev.used_upvars.len();
                        prev.used_upvars.insert(name.to_owned(), pos as _);
                        pos as i32
                    } else {
                        *prev.used_upvars.get(name).unwrap() as i32
                    };
                    last_pos = pos;
                }
                return Some(Access::Env(last_pos));
            }
            current = ctx.parent;
            prev.push(DerefPointer::new(ctx));
        }
        None
    }
    fn ident(&mut self, name: &str) -> VirtualRegister {
        if self.builder.has_var(name) {
            return self.builder.get_var(name);
        } else {
            if let Some(Access::Env(x)) = self.access_env(name) {
                let dst = self.builder.vreg();
                self.builder.emit(Ins::LoadUp { dst, up: x as _ });
                self.builder.def_var(name.to_owned(), dst);
                return dst;
            } else {
                if let Some(x) = self.global(name) {
                    let dst = self.builder.vreg();
                    self.builder.emit(Ins::Mov { dst, src: x });
                    return dst;
                } else {
                    let dst = self.builder.vreg();
                    let x = self.rt.allocate_string(name);
                    let c = self.builder.new_constant(Value::from(x));
                    self.builder.emit(Ins::LoadGlobal { dst, name: c });
                    return dst;
                }
            }
        }
    }
    pub fn compile_access(&mut self, e: &ExprKind) -> Access {
        match e {
            ExprKind::Ident(x) => {
                let name = x;
                if self.builder.has_var(name) {
                    return Access::Stack(name.to_owned(), self.builder.get_var(name));
                } else {
                    if let Some(Access::Env(x)) = self.access_env(name) {
                        let dst = self.builder.vreg();
                        self.builder.emit(Ins::LoadUp { dst, up: x as _ });
                        self.builder.def_var(name.to_owned(), dst);
                        return Access::Stack(name.to_owned(), self.builder.get_var(name));
                    } else {
                        if let Some(x) = self.global(name) {
                            /*self.builder.emit(Ins::Mov { dst, src: x });
                            return dst;*/
                            return Access::Global(x.to_constant(), true, name.to_string());
                        } else {
                            return Access::Global(0, false, name.to_string());
                        }
                    }
                }
            }
            ExprKind::Access(e, f) => {
                return Access::Field(e.clone(), f.clone());
            }
            ExprKind::This => Access::This,
            ExprKind::ArrayIndex(a, i) => return Access::Array(a.clone(), i.clone()),
            _ => unimplemented!(),
        }
    }

    pub fn compile_function(
        &mut self,
        params: &[Arg],
        e: &Box<Expr>,
        vname: Option<String>,
    ) -> Result<VirtualRegister, MsgWithPos> {
        let rt = unsafe { &mut *(self.rt as *mut Runtime) };
        let mut ctx = Context::new(
            rt,
            Rc::new(RefCell::new(Default::default())),
            Rc::new(RefCell::new(Default::default())),
            Some(DerefPointer::new(self)),
        );
        let mut used = vec![];
        for p in params.iter() {
            used.push(ctx.compile_arg(Position::new(0, 0), p)?);
        }
        while let Some(x) = used.pop() {
            ctx.builder.free_args.push_front(x);
        }
        let c = self.builder.code.creg();
        let c2 = ctx.builder.code.creg();
        if vname.is_some() {
            self.fmap
                .borrow_mut()
                .insert(vname.as_ref().unwrap().to_owned(), c);
            ctx.fmap
                .borrow_mut()
                .insert(vname.as_ref().unwrap().to_owned(), c2);
        }

        let res = ctx.compile(e)?;
        if res.is_local() && res.to_local() != 0 {
            ctx.builder.emit(Ins::Return { val: res });
        }
        let reg = if ctx.used_upvars.is_empty() == false {
            let mut args = vec![];
            for (var, _) in ctx.used_upvars.iter() {
                args.push(self.ident(var));
            }
            let dst = self.builder.vreg();
            self.builder.emit(Ins::Mov { dst, src: c });

            let dst = self.builder.close_env(dst, &args);
            dst
        } else {
            let dst = self.builder.vreg();
            self.builder.emit(Ins::Mov { dst, src: c });

            dst
        };
        let code = ctx.builder.finish(self.rt);
        let f = function_from_codeblock(
            self.rt,
            code.to_heap(),
            &vname
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or("<anonymous>".to_string()),
        );
        use crate::runtime::cell::*;
        if let CellValue::Function(Function::Regular(ref mut reg)) = f.as_cell().value {
            reg.code.constants_[c2.to_constant() as usize] = f;
        } else {
            unreachable!();
        }
        self.builder.code.constants_[c.to_constant() as usize] = f;
        Ok(reg)
    }
    pub fn access_set(&mut self, acc: Access, val: VirtualRegister) -> Result<(), MsgWithPos> {
        match acc {
            Access::Stack(name, _) => {
                self.builder.def_var(name.to_owned(), val);
                Ok(())
            }
            _ => unimplemented!(),
        }
    }
    pub fn compile(&mut self, e: &Expr) -> Result<VirtualRegister, MsgWithPos> {
        match &e.expr {
            ExprKind::Function(name, params, body) => {
                self.compile_function(params, body, name.clone())
            }
            ExprKind::Throw(e) => {
                let x = self.compile(e)?;
                self.builder.emit(Ins::Throw { src: x });
                return Ok(VirtualRegister::tmp(0));
            }
            ExprKind::Block(v) => {
                if v.is_empty() {
                    let dst = self.builder.vreg();
                    let x = self.builder.new_constant(Value::undefined());
                    self.builder.emit(Ins::Mov { dst, src: x });
                    return Ok(dst);
                } else {
                    self.builder.fallthrough();
                    let (last, _ret) = self.scoped(|this| {
                        let mut last = None;
                        for x in v.iter() {
                            last = Some(this.compile(x)?);
                        }
                        let ret = if this.builder.code.code[this.builder.current as usize]
                            .code
                            .last()
                            .expect(&format!("{}", {
                                let mut b = String::new();
                                let _ = this.builder.code.dump(&mut b, this.rt);
                                b
                            }))
                            .is_final()
                            == true
                        {
                            if let Ins::Return { .. } = this.builder.code.code
                                [this.builder.current as usize]
                                .code
                                .last()
                                .unwrap()
                            {
                                let bb = this.builder.create_new_block();
                                this.builder.switch_to_block(bb);
                                false
                            } else {
                                let bb = this.builder.create_new_block();
                                this.builder.switch_to_block(bb);
                                false
                            }
                        } else {
                            false
                        };
                        Ok((last.unwrap(), ret))
                    })?;
                    if last.is_local() && last.to_local() == 0 {
                        let dst = self.builder.vreg();
                        let x = self.builder.new_constant(Value::undefined());
                        self.builder.emit(Ins::Mov { dst, src: x });
                        return Ok(dst);
                    } else {
                        return Ok(last);
                    }
                }
            }
            ExprKind::ConstInt(x) => {
                let x = self.builder.new_constant(Value::number(*x as f64));
                let dst = self.builder.vreg();
                self.builder.emit(Ins::Mov { dst, src: x });
                return Ok(dst);
            }
            ExprKind::ConstFloat(x) => {
                let x = self.builder.new_constant(Value::number(*x as f64));
                let dst = self.builder.vreg();
                self.builder.emit(Ins::Mov { dst, src: x });
                return Ok(dst);
            }
            ExprKind::ConstStr(x) => {
                let x = self.rt.allocate_string(x);
                let x = self.builder.new_constant(Value::from(x));
                let dst = self.builder.vreg();
                self.builder.emit(Ins::Mov { dst, src: x });
                return Ok(dst);
            }
            ExprKind::Ident(x) => {
                return Ok(self.ident(x));
            }
            ExprKind::Return(Some(e)) => {
                let val = self.compile(e)?;
                self.builder.emit(Ins::Return { val });
                return Ok(val);
            }
            ExprKind::Return(None) => unimplemented!(),
            ExprKind::Let(_, pat, expr) => {
                let val = self.compile(expr)?;
                self.compile_var_pattern(e.pos, &pat, false, val)?;
                return Ok(val);
            }
            ExprKind::Assign(lhs, rhs) => {
                let acc = self.compile_access(&lhs.expr);
                let val = self.compile(rhs)?;
                self.access_set(acc, val)?;
                Ok(val)
            }
            ExprKind::If(cond, if_true, if_false) => {
                let dst = self.builder.vreg();
                let res = self.compile(cond)?;
                let (mut x, mut y) = self.builder.cjmp(res);
                let bb = self.builder.create_new_block();
                self.builder.switch_to_block(bb);
                x();
                let x = self.compile(if_true)?;
                self.builder.mov(dst, x);
                /*let bb = self.builder.create_new_block();
                self.builder.switch_to_block(bb);
                y();*/
                match if_false {
                    Some(expr) => {
                        let mut jend = self.builder.jmp();
                        let bb = self.builder.create_new_block();
                        self.builder.switch_to_block(bb);
                        y();
                        let x = self.compile(expr)?;
                        self.builder.mov(dst, x);
                        jend();
                    }
                    _ => {
                        y();
                    }
                }

                return Ok(dst);
            }
            ExprKind::BinOp(lhs, op, rhs) => self.compile_binop(op, lhs, rhs),
            ExprKind::Call(func,parameters) => {
                let (func,this) = match func.expr {
                    ExprKind::Access(ref obj,_) => {
                        let this = self.compile(obj)?;
                        (self.compile(func)?,this)
                    }
                    _ => {

                        let this = self.builder.vreg();
                        self.builder.emit(Ins::LoadThis {dst: this});
                        (self.compile(func)?,this)
                    }
                };

                let mut arguments = vec![];
                for x in parameters.iter() {
                    arguments.push(self.compile(x)?);
                }
                Ok(self.builder.do_call(func,this,&arguments))
            }
            _ => unimplemented!(),
        }
    }

    pub fn compile_binop(
        &mut self,
        op: &str,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<VirtualRegister, MsgWithPos> {
        let dst = self.builder.vreg();
        let ins = match op {
            "+" => {
                let a = self.compile(lhs)?;
                let b = self.compile(rhs)?;
                Ins::Add {
                    dst,
                    lhs: a,
                    src: b,
                    fdbk: 0,
                }
            }
            "-" => {
                let a = self.compile(lhs)?;
                let b = self.compile(rhs)?;
                Ins::Sub {
                    dst,
                    lhs: a,
                    src: b,
                    fdbk: 0,
                }
            }
            "*" => {
                let a = self.compile(lhs)?;
                let b = self.compile(rhs)?;
                Ins::Mul {
                    dst,
                    lhs: a,
                    src: b,
                    fdbk: 0,
                }
            }
            "/" => {
                let a = self.compile(lhs)?;
                let b = self.compile(rhs)?;
                Ins::Div {
                    dst,
                    lhs: a,
                    src: b,
                    fdbk: 0,
                }
            }
            _ => unimplemented!(),
        };
        self.builder.emit(ins);
        Ok(dst)
    }

    pub fn compile_var_pattern(
        &mut self,
        _pos: Position,
        pat: &Box<Pattern>,
        _mutable: bool,
        r: VirtualRegister,
    ) -> Result<(), MsgWithPos> {
        match &pat.decl {
            PatternDecl::Ident(x) => {
                self.builder.def_var(x.to_owned(), r);
                return Ok(());
            }
            _ => unimplemented!("Other var patterns not yet implemented"),
        }
    }

    pub fn compile_arg(&mut self, p: Position, arg: &Arg) -> Result<VirtualRegister, MsgWithPos> {
        match arg {
            Arg::Ident(_, name) => {
                if self.builder.has_var(name) {
                    return Err(MsgWithPos::new(
                        p,
                        Msg::Custom(format!("argument '{}' already defined", name)),
                    ));
                }
                let r = self.builder.areg();
                let dst = self.builder.vreg();
                self.builder.emit(Ins::Mov { dst, src: r });
                self.builder.def_var(name.to_owned(), dst);
                Ok(r)
            }
            _ => unimplemented!(),
        }
    }

    pub fn new(
        rt: &'a mut Runtime,
        fmap: Rc<RefCell<HashMap<String, VirtualRegister>>>,
        functions: Rc<RefCell<Vec<(Rooted<CodeBlock>, VirtualRegister, String)>>>,
        parent: Option<DerefPointer<Context<'_>>>,
    ) -> Self {
        let mut builder = ByteCompiler::new(rt);
        builder.create_new_block();
        Self {
            builder,
            parent: unsafe { std::mem::transmute(parent) },
            fmap,
            functions,
            rt,
            used_upvars: Default::default(),
        }
    }
}

pub fn compile(rt: &mut Runtime, ast: &[Box<Expr>]) -> Result<Rooted<CodeBlock>, MsgWithPos> {
    let ast = Box::new(Expr {
        pos: Position::new(0, 0),
        expr: ExprKind::Block(ast.to_vec()),
    });
    let mut ctx = Context::new(
        rt,
        Rc::new(RefCell::new(Default::default())),
        Rc::new(RefCell::new(Default::default())),
        None,
    );
    let r = ctx.compile(&ast)?;
    let r = if r.is_local() && r.to_local() == 0 {
        let dst = ctx.builder.vreg();
        let c = ctx.builder.new_constant(Value::undefined());
        ctx.builder.emit(Ins::Mov { dst, src: c });
        dst
    } else {
        r
    };
    ctx.builder.emit(Ins::Return { val: r });

    Ok(ctx.builder.finish(rt))
}

use frontend::token::*;

pub fn function_from_codeblock(rt: &mut Runtime, code: Handle<CodeBlock>, name: &str) -> Value {
    let mut b = String::new();
    code.dump(&mut b, rt).unwrap();
    println!("\nfunction {}(...): \n{}", name, b);
    use crate::runtime::cell::*;
    let name = Value::from(rt.allocate_string(name));
    let func = RegularFunction {
        code,
        name,
        env: Value::undefined(),
        kind: RegularFunctionKind::Ordinal,
        arguments: vec![],
        source: String::new(),
    };
    let proto = rt.function_prototype.to_heap();
    let f = rt.allocate_cell(Cell::new(
        CellValue::Function(Function::Regular(func)),
        Some(proto),
    ));
    Value::from(f)
}
