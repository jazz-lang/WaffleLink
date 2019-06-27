use crate::ast::Type as CType;
use cranelift::prelude::*;
use cranelift_module::*;
use cranelift_simplejit::*;

use crate::ast::{
    Constant, Element, Expr, ExprKind, Function, Interface, Location, StmtKind, Struct, TypeKind,
};

#[derive(Clone)]
pub struct Var {
    pub name: String,
    pub wty: CType,
    pub ty: types::Type,
    pub value: Variable,
    pub on_stack: bool,
}

pub fn ty_size(ty: &CType) -> usize {
    match &ty.kind {
        TypeKind::Structure(_, fields) => {
            let mut total = 0;
            for field in fields.into_iter() {
                total += ty_size(&field.1);
            }

            return total;
        }
        TypeKind::Pointer(_) | TypeKind::Optional(_) | TypeKind::Function(_, _) => 8,
        TypeKind::Array(subty, len) => {
            if len.is_some() {
                return ty_size(subty) * (len.unwrap()) as usize;
            } else {
                8 // Pointer
            }
        }
        TypeKind::Basic(name) => {
            let name: &str = name;
            match name {
                "bool" => 1,
                "int" | "int32" | "uint" | "uint32" => 4,
                "long" | "ulong" => 8,
                "ubyte" | "byte" | "char" => 1,
                "ushort" | "short" => 2,
                "float32" => 4,
                "float64" => 8,
                "usize" | "isize" => {
                    #[cfg(target_pointer_width = "32")]
                    {
                        4
                    }
                    #[cfg(target_pointer_width = "64")]
                    {
                        8
                    }
                    #[cfg(target_pointer_width = "16")]
                    {
                        2
                    }
                    #[cfg(target_pointer_width = "8")]
                    {
                        1
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => unimplemented!(),
    }
}

pub static mut PTY: Type = types::INVALID;

pub fn ty_to_cranelift(ty: &CType) -> Type {
    match &ty.kind {
        TypeKind::Pointer(_)
        | TypeKind::Structure(_, _)
        | TypeKind::Optional(_)
        | TypeKind::Function(_, _) => {
            return unsafe { PTY };
        }
        TypeKind::Basic(value) => {
            let value: &str = value;
            match value {
                "bool" => types::B1,
                "int" | "int32" | "uint" | "uint32" => types::I32,
                "long" | "ulong" => types::I64,
                "ubyte" | "byte" | "char" => types::I8,
                "ushort" | "short" => types::I16,
                "float32" => types::F32,
                "float64" => types::F64,
                "usize" | "isize" => {
                    return unsafe { PTY };
                }
                x => panic!("{:?}", x),
            }
        }
        TypeKind::Array(_, _) => types::I64,
        x => panic!("{:?}", x),
    }
}

fn retrieve_from_load(expr: &Expr) -> &Expr {
    match &expr.kind {
        ExprKind::Deref(val) => val,
        _ => expr,
    }
}

pub struct Codegen<T: Backend> {
    pub module: Module<T>,
    pub builder_ctx: FunctionBuilderContext,
    pub ctx: codegen::Context,
    data_ctx: DataContext,
    ty_info: HashMap<usize, CType>,
    elements: Vec<Element>,
    func_info: HashMap<String, Function>,
    functions: HashMap<String, FuncId>,
    pub complex_types: HashMap<String, CType>,
}

impl<T: Backend> Codegen<T> {
    pub fn new(
        ty_info: HashMap<usize, CType>,
        backend: T::Builder,
        ast: Vec<Element>,
    ) -> Codegen<T> {
        let module = Module::new(backend);
        unsafe {
            PTY = module.target_config().pointer_type();
        }
        Self {
            builder_ctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
            ty_info,
            elements: ast,
            func_info: HashMap::new(),
            functions: HashMap::new(),
            complex_types: HashMap::new(),
        }
    }

    pub fn get_function(&mut self, name: &str) -> Option<T::FinalizedFunction> {
        match self.functions.get(name) {
            Some(id) => Some(self.module.get_finalized_function(*id)),
            None => None,
        }
    }

    fn get_ty(&self, ty: &CType) -> CType {
        if ty.is_basic() {
            if let TypeKind::Basic(name) = &ty.kind {
                if self.complex_types.contains_key(name) {
                    return self.complex_types.get(name).unwrap().clone();
                }
            }
        }

        return ty.clone();
    }
    pub fn translate(&mut self) {
        let elements = self.elements.clone();
        for elem in elements.iter() {
            match elem {
                Element::Func(func) => {
                    self.func_info.insert(func.mangle_name(), func.clone());
                }
                _ => (),
            }
        }
        for elem in elements.iter() {
            match elem {
                Element::Func(func) => {
                    if !self.get_ty(&func.returns).is_struct()
                        && !self.get_ty(&func.returns).is_array()
                        && !func.returns.is_void()
                        || func.external
                    {
                        if !func.returns.is_void() {
                            self.ctx
                                .func
                                .signature
                                .returns
                                .push(AbiParam::new(ty_to_cranelift(&self.get_ty(&func.returns))));
                        }
                    } else if func.returns.is_void() {
                        //self.ctx.func.signature.params.push(AbiParam::new(types::I32));
                    } else {
                        self.ctx
                            .func
                            .signature
                            .params
                            .push(AbiParam::new(self.module.target_config().pointer_type()));
                    }

                    let mut ebb_params = vec![];
                    for p in func.parameters.iter() {
                        let ty = ty_to_cranelift(&self.get_ty(&p.1));
                        ebb_params.push(ty);
                        self.ctx.func.signature.params.push(AbiParam::new(ty));
                    }
                    if func.this.is_some() {
                        ebb_params.push(self.module.target_config().pointer_type());
                        self.ctx
                            .func
                            .signature
                            .params
                            .push(AbiParam::new(self.module.target_config().pointer_type()));
                    }

                    if func.external || func.internal || func.body.is_none() {
                        let maybe_err = self.module.declare_function(
                            &func.mangle_name(),
                            Linkage::Import,
                            &self.ctx.func.signature,
                        );
                        match maybe_err {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("{}", e);
                                std::process::exit(-1);
                            }
                        }
                    } else {
                        let mut builder =
                            FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

                        let entry_ebb = builder.create_ebb();
                        builder.switch_to_block(entry_ebb);
                        builder.append_ebb_params_for_function_params(entry_ebb);
                        builder.seal_block(entry_ebb);

                        let mut trans = FunctionTranslator {
                            module: &mut self.module,
                            ty_info: self.ty_info.clone(),
                            builder,
                            data_ctx: &mut self.data_ctx,
                            variables: HashMap::new(),
                            func_info: &self.func_info,
                            complex_types: &self.complex_types,
                            return_addr: None,
                            terminated: false,
                            break_ebb: vec![],
                            continue_ebb: vec![],
                        };
                        if trans.get_ty(&func.returns).is_struct() {
                            trans.return_addr = Some(trans.builder.ebb_params(entry_ebb)[0]);
                        }

                        for (i, param) in func.parameters.iter().enumerate() {
                            let i = if trans.get_ty(&func.returns).is_struct() {
                                i + 1
                            } else {
                                i
                            };

                            let val = trans.builder.ebb_params(entry_ebb)[i];
                            let var = Variable::new(trans.variables.len());
                            trans
                                .builder
                                .declare_var(var, ty_to_cranelift(&trans.get_ty(&param.1)));
                            trans.builder.def_var(var, val);
                            let var = Var {
                                name: param.0.clone(),
                                ty: ebb_params[i],
                                wty: trans.get_ty(&param.1),
                                on_stack: false,
                                value: var,
                            };
                            trans.variables.insert(param.0.clone(), var);
                        }
                        if let Some((name, ty)) = &func.this {
                            let val = *trans.builder.ebb_params(entry_ebb).last().unwrap();
                            let var = Variable::new(trans.variables.len());
                            trans
                                .builder
                                .declare_var(var, trans.module.target_config().pointer_type());
                            trans.builder.def_var(var, val);
                            let var = Var {
                                name: name.to_owned(),
                                ty: *ebb_params.last().unwrap(),
                                wty: trans.get_ty(ty),
                                on_stack: false,
                                value: var,
                            };
                            trans.variables.insert(name.to_owned(), var);
                        }
                        if let StmtKind::Block(stmts) = &**func.body.as_ref().unwrap() {
                            for stmt in stmts.iter() {
                                trans.translate_stmt(stmt);
                            }
                        } else {
                            trans.translate_stmt(func.body.as_ref().unwrap());
                        }

                        if unsafe { crate::DUMP_IR } {
                            let ir: String = format!("{}", trans.builder.display(None));
                            println!("{}", ir.replace("u0:0", &func.mangle_name()));
                        }

                        trans.builder.finalize();
                    }
                    let linkage = if func.external {
                        Linkage::Import
                    } else {
                        Linkage::Export
                    };

                    let id = match self.module.declare_function(
                        &func.mangle_name(),
                        linkage,
                        &self.ctx.func.signature,
                    ) {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("{}", e);
                            std::process::exit(-1);
                        }
                    };
                    if !func.external {
                        let error = self.module.define_function(id, &mut self.ctx);
                        match error {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("{}", e);
                                std::process::exit(-1);
                            }
                        }
                    }
                    self.functions.insert(func.mangle_name(), id);
                    self.module.clear_context(&mut self.ctx);
                }
                _ => { /* TODO */ }
            }
        }

        self.module.finalize_definitions();
    }
}

use std::collections::HashMap;

pub struct FunctionTranslator<'a, T: Backend> {
    pub ty_info: HashMap<usize, CType>,
    pub module: &'a mut Module<T>,
    pub data_ctx: &'a mut DataContext,
    pub func_info: &'a HashMap<String, Function>,
    pub builder: FunctionBuilder<'a>,
    pub variables: HashMap<String, Var>,
    return_addr: Option<Value>,
    complex_types: &'a HashMap<String, CType>,
    terminated: bool,
    break_ebb: Vec<Ebb>,
    continue_ebb: Vec<Ebb>,
}

impl<'a, T: Backend> FunctionTranslator<'a, T> {
    pub(crate) fn translate_binop(&mut self, op: &str, lhs: &Expr, rhs: &Expr) -> Value {
        let ty = self.ty_info.get(&lhs.id).unwrap().clone();
        let ty2 = self.ty_info.get(&rhs.id).unwrap().clone();
        let x = self.translate_expr(lhs).0;
        let y = self.translate_expr(rhs).0;
        assert!(ty.is_basic() || ty.is_pointer());
        match &ty.kind {
            TypeKind::Basic(name) => {
                let name: &str = name;
                match name {
                    n if [
                        "int", "uint", "ulong", "long", "short", "ushort", "ubyte", "byte",
                        "usize", "isize",
                    ]
                    .contains(&n) =>
                    {
                        assert!(ty2.is_basic());
                        match op {
                            "+" => return self.builder.ins().iadd(x, y),
                            "-" => return self.builder.ins().isub(x, y),
                            "/" => return self.builder.ins().sdiv(x, y),
                            "*" => return self.builder.ins().imul(x, y),
                            ">>" => return self.builder.ins().sshr(x, y),
                            "<<" => return self.builder.ins().ishl(x, y),
                            ">" => return self.builder.ins().icmp(IntCC::SignedGreaterThan, x, y),
                            "<" => return self.builder.ins().icmp(IntCC::SignedLessThan, x, y),
                            ">=" => {
                                return self.builder.ins().icmp(
                                    IntCC::SignedGreaterThanOrEqual,
                                    x,
                                    y,
                                )
                            }
                            "<=" => {
                                return self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, x, y)
                            }
                            "|" => return self.builder.ins().bor(x, y),
                            "&" => return self.builder.ins().band(x, y),
                            "^" => return self.builder.ins().bxor(x, y),
                            "==" => return self.builder.ins().icmp(IntCC::Equal, x, y),
                            "!=" => return self.builder.ins().icmp(IntCC::NotEqual, x, y),
                            _ => unimplemented!(),
                        }
                    }
                    n if n == "bool" => match op {
                        "||" => return self.builder.ins().bor(x, y),
                        "&&" => return self.builder.ins().band(x, y),
                        "==" => return self.builder.ins().icmp(IntCC::Equal, x, y),
                        "!=" => return self.builder.ins().icmp(IntCC::NotEqual, x, y),
                        _ => unreachable!(),
                    },

                    n if ["float32", "float64"].contains(&n) => match op {
                        "+" => return self.builder.ins().fadd(x, y),
                        "-" => return self.builder.ins().fsub(x, y),
                        "/" => return self.builder.ins().fdiv(x, y),
                        "*" => return self.builder.ins().fmul(x, y),
                        ">" => return self.builder.ins().fcmp(FloatCC::GreaterThan, x, y),
                        "<" => return self.builder.ins().fcmp(FloatCC::LessThan, x, y),
                        ">=" => return self.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, x, y),
                        "<=" => return self.builder.ins().fcmp(FloatCC::LessThanOrEqual, x, y),
                        "==" => return self.builder.ins().fcmp(FloatCC::Equal, x, y),
                        "!=" => return self.builder.ins().fcmp(FloatCC::NotEqual, x, y),
                        _ => unimplemented!(),
                    },

                    _ => unreachable!(),
                }
            }
            _ => unimplemented!(),
        }
    }

    fn translate_cast(&mut self, value: Value, from: &CType, to: &CType) -> Value {
        if to == from {
            return value;
        }
        if to.is_basic() && from.is_basic() {
            if to.is_basic_names(&["uint", "ulong", "ubyte", "ushort", "ulong", "usize"])
                && from.is_basic_names(&[
                    "uint", "ulong", "ubyte", "ushort", "ulong", "usize", "int", "long", "byte",
                    "short", "isize",
                ])
            {
                if ty_size(to) < ty_size(from) {
                    return self.builder.ins().ireduce(ty_to_cranelift(to), value);
                } else if ty_size(to) == ty_size(from) {
                    return value;
                } else {
                    return self.builder.ins().uextend(ty_to_cranelift(to), value);
                }
            }
            if to.is_basic_names(&["int", "long", "byte", "short", "isize"])
                && from.is_basic_names(&[
                    "int", "long", "byte", "short", "size", "uint", "ulong", "ubyte", "ushort",
                    "ulong", "usize",
                ])
            {
                if ty_size(to) < ty_size(from) {
                    return self.builder.ins().ireduce(ty_to_cranelift(to), value);
                } else if ty_size(to) == ty_size(from) {
                    return value;
                } else {
                    return self.builder.ins().sextend(ty_to_cranelift(to), value);
                }
            }

            if to.is_basic_names(&["uint", "ulong", "ubyte", "ushort", "ulong", "usize"])
                && from.is_basic_names(&["float32", "float64"])
            {
                return self.builder.ins().fcvt_to_uint(ty_to_cranelift(to), value);
            }
            if to.is_basic_names(&["int", "long", "byte", "short", "long", "isize"])
                && from.is_basic_names(&["float32", "float64"])
            {
                return self.builder.ins().fcvt_to_sint(ty_to_cranelift(to), value);
            }

            if to.is_basic_name("float32") && from.is_basic_name("float64") {
                return self.builder.ins().fpromote(types::F64, value);
            }

            if to.is_basic_name("float64") && from.is_basic_name("float32") {
                return self.builder.ins().fdemote(types::F32, value);
            }

            if from.is_basic_names(&["uint", "ulong", "ubyte", "ushort", "ulong", "usize"])
                && to.is_basic_names(&["float32", "float64"])
            {
                return self
                    .builder
                    .ins()
                    .fcvt_from_uint(ty_to_cranelift(to), value);
            }
            if from.is_basic_names(&["int", "long", "byte", "short", "long", "isize"])
                && to.is_basic_names(&["float32", "float64"])
            {
                return self
                    .builder
                    .ins()
                    .fcvt_from_sint(ty_to_cranelift(to), value);
            } else {
                let cty1 = ty_to_cranelift(to);
                let cty2 = ty_to_cranelift(from);
                if cty1 == cty2 {
                    return value;
                }
                return self.builder.ins().bitcast(ty_to_cranelift(to), value);
            }
        } else {
            let cty1 = ty_to_cranelift(to);
            let cty2 = ty_to_cranelift(from);
            if cty1 == cty2 {
                return value;
            }
            return self.builder.ins().bitcast(ty_to_cranelift(to), value);
        }
    }

    pub(crate) fn translate_expr(&mut self, expr: &Expr) -> (Value, Option<Variable>) {
        match &expr.kind {
            ExprKind::Integer(value, suffix) => {
                use crate::lexer::IntSuffix;
                let value = *value;
                return (
                    match suffix {
                        IntSuffix::Int | IntSuffix::UInt => {
                            self.builder.ins().iconst(types::I32, value as i64)
                        }
                        IntSuffix::Long | IntSuffix::ULong => {
                            self.builder.ins().iconst(types::I64, value as i64)
                        }
                        IntSuffix::Byte | IntSuffix::UByte => {
                            self.builder.ins().iconst(types::I8, value as i64)
                        }
                    },
                    None,
                );
            }
            ExprKind::Float(value, suffix) => {
                use crate::lexer::FloatSuffix;
                let value = *value;
                return (
                    match suffix {
                        FloatSuffix::Float => self.builder.ins().f32const(value as f32),
                        FloatSuffix::Double => self.builder.ins().f64const(value),
                    },
                    None,
                );
            }

            ExprKind::Bool(val) => (self.builder.ins().bconst(types::B1, *val), None),
            ExprKind::Conv(val, _) => {
                let output = self.get_ty(&self.ty_info.get(&expr.id).unwrap().clone());
                let from = self.get_ty(&self.ty_info.get(&val.id).unwrap().clone());
                let value = self.translate_expr(val);

                (self.translate_cast(value.0, &from, &output), None)
            }
            ExprKind::Binary(op, lhs, rhs) => return (self.translate_binop(op, lhs, rhs), None),
            ExprKind::String(value) => {
                self.data_ctx
                    .define(value.clone().into_boxed_str().into_boxed_bytes());
                let id = self
                    .module
                    .declare_data(&format!("__str_{}", expr.id), Linkage::Export, true)
                    .map_err(|e| e.to_string())
                    .unwrap();

                self.module.define_data(id, &self.data_ctx).unwrap();
                self.module.finalize_definitions();

                let local_id = self.module.declare_data_in_func(id, &mut self.builder.func);
                let pointer = self.module.target_config().pointer_type();
                self.data_ctx.clear();
                return (self.builder.ins().symbol_value(pointer, local_id), None);
            }
            ExprKind::Deref(val) => {
                let value = self.translate_expr(val);
                let cty = self.ty_info.get(&expr.id).unwrap();
                let ty = ty_to_cranelift(cty);
                return (
                    self.builder.ins().load(ty, MemFlags::new(), value.0, 0),
                    value.1,
                );
            }
            ExprKind::Call(name, this, parameters) => {
                let mut sig = self.module.make_signature();

                let return_ty = self.ty_info.get(&expr.id).unwrap().clone();
                let mut args = Vec::new();
                let mut return_addr = None;
                let name = if this.is_some() {
                    let ty = self.ty_info.get(&this.as_ref().unwrap().id).unwrap();
                    if !ty.is_pointer() {
                        format!("this{}_{}", ty, name)
                    } else {
                        format!("this{}_{}", ty.get_subty().unwrap(), name)
                    }
                } else {
                    name.to_owned()
                };
                let fun = self.func_info.get(&name).unwrap().clone();
                if (return_ty.is_array() || return_ty.is_struct()) & !fun.external {
                    sig.params
                        .push(AbiParam::new(self.module.target_config().pointer_type()));
                    let slot = self.builder.create_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        ty_size(&return_ty) as u32,
                    ));
                    let addr = self.builder.ins().stack_addr(
                        self.module.target_config().pointer_type(),
                        slot,
                        0,
                    );

                    args.push(addr);
                    return_addr = Some(addr);
                } else if !return_ty.is_void() {
                    sig.returns.push(AbiParam::new(ty_to_cranelift(&return_ty)));
                } else {
                }

                for param in fun.parameters.iter() {
                    let ty = ty_to_cranelift(&self.get_ty(&param.1));
                    sig.params.push(AbiParam::new(ty));
                }

                if this.is_some() {
                    sig.params
                        .push(AbiParam::new(self.module.target_config().pointer_type()));
                }

                let callee = self.module.declare_function(&name, Linkage::Import, &sig);
                let callee = match callee {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(-1);
                    }
                };
                let local_callee = self
                    .module
                    .declare_func_in_func(callee, &mut self.builder.func);

                for arg in parameters.iter() {
                    args.push(self.translate_expr(arg).0);
                }
                if this.is_some() {
                    let this = this.as_ref().unwrap();

                    let value = self.translate_expr(this);

                    args.push(value.0);
                }

                let call = self.builder.ins().call(local_callee, &args);

                let return_value = if return_ty.is_struct() || return_ty.is_array() {
                    return_addr.unwrap()
                } else {
                    if return_ty.is_void() {
                        return (Value::new(0 as usize), None);
                    } else {
                        self.builder.inst_results(call)[0]
                    }
                };

                (return_value, None)
            }
            ExprKind::Identifier(name) => {
                let value = if self.variables.contains_key(name) {
                    let var: &Var = self.variables.get(name).unwrap();
                    if var.on_stack {
                        let value = self.builder.use_var(var.value);
                        return (value, None);
                    } else {
                        return (self.builder.use_var(var.value), Some(var.value));
                    }
                } else {
                    unimplemented!()
                };

                value
            }
            ExprKind::AddrOf(val) => self.translate_expr(retrieve_from_load(val)),
            ExprKind::Assign(to, from) => {
                let to_ty = self.ty_info.get(&to.id).unwrap().clone();
                //let from_ty = self.ty_info.get(&from.id).unwrap().clone();
                //let lhs = self.translate_expr(to);
                let rhs = self.translate_expr(from);

                match &to.kind {
                    ExprKind::Member(object, field) => {
                        let mut offset = 0;
                        let object_val = self.translate_expr(object);
                        let ty = self.ty_info.get(&object.id).unwrap().clone();
                        let ty = if ty.is_struct() {
                            ty
                        } else {
                            ty.get_subty().unwrap().clone()
                        };
                        if ty.is_struct() {
                            let x = if let TypeKind::Structure(_, fields) = &ty.kind {
                                fields.clone()
                            } else {
                                unreachable!()
                            };
                            for field_ in x.iter() {
                                if &field_.0 == field {
                                    break;
                                }

                                offset += ty_size(&field_.1);
                            }
                        }

                        self.builder.ins().store(
                            MemFlags::new(),
                            rhs.0,
                            object_val.0,
                            offset as i32,
                        );
                    }

                    _ => {
                        let lhs = self.translate_expr(to);
                        if to_ty.is_basic() {
                            self.builder.def_var(lhs.1.unwrap(), rhs.0);
                        } else {
                            self.builder.ins().store(MemFlags::new(), rhs.0, lhs.0, 0);
                        }
                    }
                }

                return rhs;
            }
            ExprKind::Member(object, field) => {
                let ty = self.ty_info.get(&object.id).unwrap().clone();
                let val = self.translate_expr(object).0;
                let mut offset = 0;
                let ty = if ty.is_struct() {
                    ty
                } else {
                    ty.get_subty().unwrap().clone()
                };
                if ty.is_struct() {
                    let x = if let TypeKind::Structure(_, fields) = &ty.kind {
                        fields.clone()
                    } else {
                        unreachable!()
                    };

                    for field_ in x.iter() {
                        if &field_.0 == field {
                            break;
                        }
                        offset += ty_size(&field_.1);
                    }
                }
                let output = ty_to_cranelift(self.ty_info.get(&expr.id).as_ref().unwrap());
                return (
                    self.builder
                        .ins()
                        .load(output, MemFlags::new(), val, offset as i32),
                    None,
                );
            }
            x => panic!("{:?}", x),
        }
    }

    fn get_ty(&self, ty: &CType) -> CType {
        if ty.is_basic() {
            if let TypeKind::Basic(name) = &ty.kind {
                if self.complex_types.contains_key(name) {
                    return self.complex_types.get(name).unwrap().clone();
                }
            }
        }

        return ty.clone();
    }

    pub fn translate_stmt(&mut self, s: &StmtKind) {
        match s {
            StmtKind::Expr(expr) => {
                self.translate_expr(expr);
            }
            StmtKind::Block(stmts) => {
                let old_locals = self.variables.clone();
                let block = self.builder.create_ebb();
                self.builder.ins().jump(block, &[]);
                self.builder.switch_to_block(block);
                self.builder.seal_block(block);
                for stmt in stmts.iter() {
                    self.translate_stmt(stmt);
                }

                self.variables = old_locals;
            }
            StmtKind::Return(val) => {
                if self.return_addr.is_some() {
                    let val_ = self.translate_expr(val.as_ref().unwrap());
                    self.builder
                        .ins()
                        .store(MemFlags::new(), val_.0, self.return_addr.unwrap(), 0);
                    self.builder.ins().return_(&[]);
                    self.terminated = true;
                    return;
                }
                if val.is_none() {
                    self.builder.ins().return_(&[]);
                } else {
                    let val = self.translate_expr(val.as_ref().unwrap());
                    self.builder.ins().return_(&[val.0]);
                }
                self.terminated = true;
            }
            StmtKind::If(cond, then, otherwise) => {
                let cond_value = self.translate_expr(cond).0;
                let else_block = self.builder.create_ebb();
                let merge_block = self.builder.create_ebb();

                self.builder.ins().brz(cond_value, else_block, &[]);
                if let StmtKind::Block(stmts) = &**then {
                    for stmt in stmts.iter() {
                        self.translate_stmt(stmt);
                    }
                } else {
                    self.translate_stmt(then);
                }
                if !self.terminated {
                    self.builder.ins().jump(merge_block, &[]);
                }
                self.terminated = false;
                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                if otherwise.is_some() {
                    if let StmtKind::Block(stmts) = &**otherwise.as_ref().unwrap() {
                        for stmt in stmts.iter() {
                            self.translate_stmt(stmt);
                        }
                    } else {
                        self.translate_stmt(otherwise.as_ref().unwrap());
                    }
                }

                if !self.terminated {
                    self.builder.ins().jump(merge_block, &[]);
                }
                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);
                self.terminated = false;
            }
            StmtKind::Continue => {
                self.builder.ins().jump(
                    *self
                        .continue_ebb
                        .last()
                        .expect("break statement outside fo loop"),
                    &[],
                );
                let dead_block = self.builder.create_ebb();
                self.builder.switch_to_block(dead_block);
                self.builder.seal_block(dead_block);
            }
            StmtKind::Break => {
                self.builder.ins().jump(
                    *self
                        .break_ebb
                        .last()
                        .expect("break statement outside fo loop"),
                    &[],
                );
                let dead_block = self.builder.create_ebb();
                self.builder.switch_to_block(dead_block);
                self.builder.seal_block(dead_block);
            }
            StmtKind::VarDecl(name, ty, val) => {
                let ty = if ty.is_some() {
                    self.get_ty(ty.as_ref().unwrap())
                } else {
                    let val = val.as_ref().unwrap();
                    self.ty_info.get(&val.id).unwrap().clone()
                };

                let variable = Variable::new(self.variables.len());
                self.builder
                    .declare_var(variable, ty_to_cranelift(&self.get_ty(&ty)));

                let value = if val.is_none() {
                    let slot = self.builder.create_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        ty_size(&ty) as u32,
                    ));
                    if ty.is_struct() || ty.is_array() {
                        self.builder.ins().stack_addr(
                            self.module.target_config().pointer_type(),
                            slot,
                            0,
                        )
                    } else {
                        self.builder.ins().stack_load(ty_to_cranelift(&ty), slot, 0)
                    }
                } else {
                    self.translate_expr(val.as_ref().unwrap()).0
                };

                self.builder.def_var(variable, value);

                let var = Var {
                    name: name.to_owned(),
                    ty: ty_to_cranelift(&ty),
                    wty: ty.clone(),
                    on_stack: ty.is_struct() || ty.is_array(),
                    value: variable,
                };

                self.variables.insert(name.to_owned(), var);
            }
            StmtKind::While(cond, body) => {
                let variables = self.variables.clone();
                let header_ebb = self.builder.create_ebb();
                let exit_ebb = self.builder.create_ebb();
                self.break_ebb.push(exit_ebb);
                self.continue_ebb.push(header_ebb);
                self.builder.ins().jump(header_ebb, &[]);

                self.builder.switch_to_block(header_ebb);

                let cond_val = self.translate_expr(cond).0;

                self.builder.ins().brz(cond_val, exit_ebb, &[]);
                if let StmtKind::Block(stmts) = &**body {
                    for stmt in stmts.iter() {
                        self.translate_stmt(stmt);
                    }
                } else {
                    self.translate_stmt(body);
                }

                self.builder.ins().jump(header_ebb, &[]);
                self.builder.switch_to_block(exit_ebb);
                self.builder.seal_block(header_ebb);
                self.builder.seal_block(exit_ebb);
                self.break_ebb.pop();
                self.continue_ebb.pop();
                self.variables = variables;
            }
            StmtKind::For(decl, cond, then, body) => {
                let variables = self.variables.clone();
                self.translate_stmt(decl);
                let header_ebb = self.builder.create_ebb();
                let exit_ebb = self.builder.create_ebb();
                self.break_ebb.push(exit_ebb);
                self.continue_ebb.push(header_ebb);
                self.builder.ins().jump(header_ebb, &[]);

                self.builder.switch_to_block(header_ebb);

                let cond_val = self.translate_expr(cond).0;

                self.builder.ins().brz(cond_val, exit_ebb, &[]);
                if let StmtKind::Block(stmts) = &**body {
                    for stmt in stmts.iter() {
                        self.translate_stmt(stmt);
                    }
                } else {
                    self.translate_stmt(body);
                }
                self.translate_expr(then);

                self.builder.ins().jump(header_ebb, &[]);
                self.builder.switch_to_block(exit_ebb);
                self.builder.seal_block(header_ebb);
                self.builder.seal_block(exit_ebb);
                self.break_ebb.pop();
                self.continue_ebb.pop();
                self.variables = variables;
            }
            _ => unimplemented!(),
        }
    }
}
