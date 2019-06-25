use crate::ast::Type as CType;
use cranelift::prelude::*;
use cranelift_module::*;
use cranelift_simplejit::*;

use crate::ast::{
    Constant, Expr, ExprKind, Function, Interface, Location, StmtKind, Struct, TypeKind,Element
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

pub fn ty_to_cranelift(ty: &CType) -> Type {
    match &ty.kind {
        TypeKind::Pointer(_)
        | TypeKind::Structure(_, _)
        | TypeKind::Optional(_)
        | TypeKind::Function(_, _) => types::I64,
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
                    #[cfg(target_pointer_width = "32")]
                    {
                        return types::I32;
                    }
                    #[cfg(target_pointer_width = "64")]
                    {
                        return types::I64;
                    }
                    #[cfg(target_pointer_width = "16")]
                    {
                        return types::I16;
                    }
                    #[cfg(target_pointer_width = "8")]
                    {
                        return types::I8;
                    }
                }
                _ => unreachable!(),
            }
        }
        TypeKind::Array(_, _) => types::I64,
        _ => unreachable!(),
    }
}

pub struct Codegen<T: Backend> {
    pub module: Module<T>,
    pub builder_ctx: FunctionBuilderContext,
    pub ctx: codegen::Context,
    data_ctx: DataContext,
    ty_info: HashMap<usize, CType>,
    elements: Vec<Element>,
    func_info: HashMap<String,Function>
}

impl<T: Backend> Codegen<T> {
    pub fn new(ty_info: HashMap<usize, CType>, backend: T::Builder,ast: Vec<Element>) -> Codegen<T> {
        let module = Module::new(backend);

        Self {
            builder_ctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
            ty_info,
            elements: ast,
            func_info: HashMap::new()
        }
    }

    pub fn translate(&mut self) {
        let elements = self.elements.clone();

        for elem in elements.iter() {
            match elem {
                Element::Func(func) => {
                    
                    if !func.returns.is_struct() && !func.returns.is_array() && !func.returns.is_void() {
                        self.ctx.func.signature.returns.push(AbiParam::new(ty_to_cranelift(&func.returns)));
                    } else if func.returns.is_void() {
                        //self.ctx.func.signature.params.push(AbiParam::new(types::I32));
                    } else {
                        self.ctx.func.signature.params.push(AbiParam::new(types::I64));
                    }

                    if func.this.is_some() {
                        self.ctx.func.signature.params.push(AbiParam::new(types::I64));
                    }

                    for p in func.parameters.iter() {
                        self.ctx.func.signature.params.push(AbiParam::new(ty_to_cranelift(&p.1)));
                    }


                    
                    if func.external || func.internal || func.body.is_none() {
                        let maybe_err = self.module.declare_function(&func.name,Linkage::Import,&self.ctx.func.signature);
                        match maybe_err {
                            Ok(_) => {},
                            Err(e) => {
                                eprintln!("{}",e);
                            } 
                        }
                    } else {

                        let mut builder = FunctionBuilder::new(&mut self.ctx.func,&mut self.builder_ctx);

                        let entry_ebb = builder.create_ebb();
                        builder.switch_to_block(entry_ebb);
                        builder.seal_block(entry_ebb);

                        let mut trans = FunctionTranslator {
                            module: &mut self.module,
                            ty_info: self.ty_info.clone(),
                            builder,
                            data_ctx: &mut self.data_ctx,
                            variables: HashMap::new(),
                            func_info: &self.func_info
                        };
                        trans.translate_stmt(func.body.as_ref().unwrap());

                        trans.builder.finalize();
                        println!("IR dump of `{}` function:",func.name);
                        println!("{}",trans.builder.func.display(None));
                    }

                    self.func_info.insert(func.name.clone(),func.clone()); // TODO: Mangle function names

                }
                _ => {/* TODO */}
            }
        }
    }
}

use std::collections::HashMap;



pub struct FunctionTranslator<'a, T: Backend> {
    pub ty_info: HashMap<usize,CType>,
    pub module: &'a mut Module<T>,
    pub data_ctx: &'a mut DataContext,
    pub func_info: &'a HashMap<String,Function>,
    pub builder: FunctionBuilder<'a>,
    pub variables: HashMap<String, Var>,
}

impl<'a, T: Backend> FunctionTranslator<'a, T> {
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

                let local_id = self
                    .module
                    .declare_data_in_func(id, &mut self.builder.func);
                let pointer = self.module.target_config().pointer_type();
                return (self.builder.ins().symbol_value(pointer, local_id), None);
            }
            ExprKind::Deref(val) => {
                let value = self.translate_expr(val);
                let cty = self.ty_info.get(&expr.id).unwrap();
                let ty = ty_to_cranelift(cty.get_subty().unwrap());
                return (
                    self.builder.ins().load(ty, MemFlags::new(), value.0, 0),
                    value.1,
                );
            }
            ExprKind::Call(name,this,parameters) => {
                if this.is_none() {
                    let mut sig = self.module.make_signature();

                    let return_ty = self.ty_info.get(&expr.id).unwrap().clone();
                    let mut args = Vec::new();
                    let mut return_addr = None;
                    
                    if return_ty.is_array() || return_ty.is_struct() {
                        sig.params.push(AbiParam::new(types::I64));
                        let slot = self.builder.create_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, ty_size(&return_ty) as u32));
                        let addr = self.builder.ins().stack_addr(types::I64,slot,0);

                        args.push(addr);
                        return_addr = Some(addr);
                    } else if !return_ty.is_void() {
                        sig.returns.push(AbiParam::new(ty_to_cranelift(&return_ty)));
                    } else {}

                    let fun = self.func_info.get(name).unwrap().clone();
                    for param in fun.parameters.iter() {
                        let ty = ty_to_cranelift(&param.1);
                        sig.params.push(AbiParam::new(ty));
                    }

                    let callee = self
                        .module
                        .declare_function(&name,Linkage::Import,&sig)
                        .expect("problem declaring function");
                    let local_callee = self.module
                        .declare_func_in_func(callee, &mut self.builder.func);
                    

                    
                    
                    for arg in parameters.iter() {
                        args.push(self.translate_expr(arg).0);
                    }
                    let call = self.builder.ins().call(local_callee,&args);

                    let return_value = if return_ty.is_struct() || return_ty.is_array() {
                        return_addr.unwrap()
                    } else {
                        if return_ty.is_void() {
                            self.builder.ins().iconst(types::I32,0)
                        } else {
                            self.builder.inst_results(call)[0]
                        }
                        
                    };

                    (return_value,None)
                } else {
                    unimplemented!()
                }
            }
            ExprKind::Identifier(name) => {
                if self.variables.contains_key(name) {
                    let var: &Var = self.variables.get(name).unwrap();
                    if var.on_stack {
                        let value = self.builder.use_var(var.value);
                        return (
                            self.builder.ins().load(var.ty, MemFlags::new(), value, 0),
                            None,
                        );
                    } else {
                        return (self.builder.use_var(var.value), Some(var.value));
                    }
                } else {
                    unimplemented!()
                }
            }
            ExprKind::AddrOf(val) => {
                let value = self.translate_expr(val);
                let ty = self.ty_info.get(&val.id).unwrap();
                let slot = self.builder.create_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    ty_size(ty) as u32,
                ));
                self.builder.ins().stack_store(value.0, slot, 0);
                let addr = self.builder.ins().stack_addr(types::I64, slot, 0);
                return (addr, None);
            }
            ExprKind::Assign(to, from) => {
                let to_ty = self.ty_info.get(&to.id).unwrap().clone();
                //let from_ty = self.ty_info.get(&from.id).unwrap().clone();
                let lhs = self.translate_expr(to);
                let rhs = self.translate_expr(from);

                if to_ty.is_pointer() || to_ty.is_basic() {
                    self.builder.def_var(lhs.1.unwrap(), rhs.0);
                } else {
                    match &to.kind {
                        ExprKind::Member(object, field) => {
                            let mut offset = 0;
                            let object_val = self.translate_expr(object);
                            let ty = self.ty_info.get(&object.id).unwrap().clone();
                            if ty.is_struct() {
                                let x = if let TypeKind::Structure(_, fields) = &ty.kind {
                                    fields.clone()
                                } else {
                                    unreachable!()
                                };

                                for field_ in x.iter() {
                                    if &field_.0 != field {
                                        offset += ty_size(&field_.1);
                                    }
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
                            self.builder.ins().store(MemFlags::new(), rhs.0, lhs.0, 0);
                        }
                    }
                }

                return rhs;
            }
            _ => unimplemented!(),
        }
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
                if val.is_none() {
                    self.builder.ins().return_(&[]);
                } else {
                    let val = self.translate_expr(val.as_ref().unwrap());
                    self.builder.ins().return_(&[val.0]);
                }
            }
            StmtKind::VarDecl(name, ty, val) => {
                let ty = if ty.is_some() {
                    *ty.clone().unwrap()
                } else {
                    let val = val.as_ref().unwrap();
                    self.ty_info.get(&val.id).unwrap().clone()
                };

                let variable = Variable::new(self.variables.len());
                self.builder.declare_var(variable, ty_to_cranelift(&ty));
                let value = if val.is_none() {
                    let slot = self.builder.create_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        ty_size(&ty) as u32,
                    ));
                    if ty.is_struct() || ty.is_array() {
                        self.builder.ins().stack_addr(types::I64, slot, 0)
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

                self.variables.insert(name.to_owned(),var);
            }
            _ => unimplemented!(),
        }
    }
}
