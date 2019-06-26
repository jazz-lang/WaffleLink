use super::Context;

use super::ast::*;
use colored::Colorize;
use std::collections::HashMap;

#[derive(Clone)]
pub struct CStructure {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub ty: Type,
}

pub struct TypeChecker<'a> {
    pub ctx: &'a mut Context,
    pub type_info: HashMap<usize, Type>,
    functions: HashMap<String, Function>,
    methods: HashMap<String, Vec<Function>>,
    structures: HashMap<String, CStructure>,
    interfaces: HashMap<String, Interface>,
    current_func: Option<Function>,
    globals: HashMap<String, (Type, bool)>,
    locals: HashMap<String, Type>,
    pub complex: HashMap<String, Type>,
}

impl<'a> TypeChecker<'a> {
    fn ty_impls_interface(&self, ty: &Type, interface: &Interface) -> bool {
        let methods = self.methods.get(&format!("{}",ty)).clone().unwrap();
        let funcs = interface.functions.iter();
        for ifunc in funcs {
            for method in methods.iter() {
                if method.name == ifunc.name
                    && method.parameters == ifunc.parameters
                    && method.returns == ifunc.returns
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn new(ctx: &'a mut Context) -> TypeChecker<'a> {
        TypeChecker {
            ctx,
            type_info: HashMap::new(),
            functions: HashMap::new(),
            structures: HashMap::new(),
            interfaces: HashMap::new(),
            current_func: None,
            globals: HashMap::new(),
            locals: HashMap::new(),
            methods: HashMap::new(),
            complex: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let elements = self.ctx.merged.as_ref().unwrap().ast.clone();

        for elem in elements.iter() {
            match elem {
                Element::Struct(s) => {
                    if self.structures.contains_key(&s.name) {
                        error!(
                            &format!("Structure with name `{}` already exists", s.name),
                            s.loc
                        );
                    }

                    self.structures.insert(
                        s.name.clone(),
                        CStructure {
                            name: s.name.clone(),
                            fields: s
                                .fields
                                .iter()
                                .map(|(_, x, y)| (x.clone(), *y.clone()))
                                .collect::<Vec<_>>(),
                            ty: Type::new(
                                s.loc.clone(),
                                TypeKind::Structure(
                                    s.name.clone(),
                                    s.fields
                                        .iter()
                                        .map(|(_, x, y)| (x.clone(), y.clone()))
                                        .collect::<Vec<_>>(),
                                ),
                            ),
                        },
                    );
                }

                Element::Interface(interface) => {
                    if self.interfaces.contains_key(&interface.name) {
                        error!(
                            &format!(
                                "Interface with name `{}` exists ( declared at {} )",
                                interface.name,
                                &self.interfaces.get(&interface.name).unwrap().pos
                            ),
                            interface.pos
                        )
                    }

                    self.interfaces
                        .insert(interface.name.clone(), interface.clone());
                }

                _ => (),
            }
        }

        for elem in elements.iter() {
            match elem {
                Element::Struct(s) => {
                    let new_fields = s
                        .fields
                        .iter()
                        .map(|(_, name, ty)| (name.clone(), self.infer_type(ty)))
                        .collect::<Vec<_>>();

                    let s = self.structures.get_mut(&s.name).unwrap();
                    s.fields = new_fields.clone();
                    s.ty = Type::new(
                        s.ty.pos.clone(),
                        TypeKind::Structure(
                            s.name.clone(),
                            new_fields
                                .into_iter()
                                .map(|(name, ty)| (name.clone(), box ty.clone()))
                                .collect::<Vec<_>>(),
                        ),
                    );
                }
                Element::Interface(interface) => {
                    let funcs = interface
                        .functions
                        .iter()
                        .map(|func| {
                            let mut new: Function = func.clone();
                            new.returns = box self.infer_type(&new.returns.clone());
                            new.parameters = new
                                .parameters
                                .into_iter()
                                .map(|(name, ty)| (name.clone(), box self.infer_type(&ty.clone())))
                                .collect::<Vec<_>>();
                            new
                        })
                        .collect();
                    let interface = self.interfaces.get_mut(&interface.name).unwrap();

                    interface.functions = funcs;
                }

                _ => (),
            }
        }

        for elem in elements.iter() {
            match elem {
                Element::Var(var) => {
                    let ty = self.infer_type(&var.ty);
                    self.globals.insert(var.name.clone(), (ty, true));
                }
                Element::Const(constant) => {
                    let ty = if constant.ty.is_some() {
                        let ty = self.infer_type(&constant.ty.as_ref().unwrap());
                        let expr_ty = self.check_expr(&constant.value);
                        if ty != expr_ty {
                            error!(
                                &format!("can not assign `{}` to `{}`", expr_ty, ty),
                                constant.pos
                            )
                        }
                        ty
                    } else {
                        self.check_expr(&constant.value)
                    };
                    self.globals.insert(constant.name.clone(), (ty, false));
                }
                Element::Func(func) => {
                    if let Some(this) = func.this.clone() {
                        let this_ty = self.infer_type(&this.1);
                        assert!(this_ty.is_pointer());
                       
                        if this_ty.get_pointee().unwrap().is_pointer() {
                            error!("Methods for pointers not implemented", this_ty.pos);
                        }
                        if this_ty.get_pointee().unwrap().is_interface() {
                            error!("Interface can not be `This` type", this_ty.pos);
                        }
                        if self.methods.contains_key(&format!("{}",this_ty.get_pointee().unwrap())) {
                            let mut func = func.clone();
                            if this_ty.is_interface() {
                                unimplemented!()
                            }
                            func.returns = box self.infer_type(&func.returns);
                            func.parameters = func
                                .parameters
                                .iter()
                                .map(|(name, ty)| (name.clone(), box self.infer_type(&ty.clone())))
                                .collect::<Vec<_>>();
                            func.this = Some((this.0.clone(), box this_ty.clone()));
                            let methods = self.methods.get_mut(&format!("{}",this_ty)).unwrap();
                            for method in methods.into_iter() {
                                if method.name == func.name {
                                    error!(
                                        &format!("Method with name `{}` already exists", func.name),
                                        func.pos
                                    );
                                }
                            }

                            methods.push(func.clone());
                        } else {
                            let mut func = func.clone();
                            func.returns = box self.infer_type(&func.returns);
                            func.parameters = func
                                .parameters
                                .iter()
                                .map(|(name, ty)| (name.clone(), box self.infer_type(&ty.clone())))
                                .collect::<Vec<_>>();
                            
                            func.this = Some((this.0.clone(), box this_ty.clone()));
                            self.methods
                                .insert(format!("{}",this_ty.get_pointee().unwrap().clone()), vec![func]);
                        }
                    } else {
                        if self.functions.contains_key(&func.name) {
                            error!("Function already exists", func.pos);
                        }

                        let mut func = func.clone();
                        func.returns = box self.infer_type(&func.returns);
                        func.parameters = func
                            .parameters
                            .iter()
                            .map(|(name, ty)| (name.clone(), box self.infer_type(&ty.clone())))
                            .collect::<Vec<_>>();
                        self.functions.insert(func.name.clone(), func);
                    }
                }
                _ => (),
            }
        }

        for (_name, func) in self.functions.clone().iter() {
            if func.body.is_none() {
                continue;
            }
            self.locals.clear();
            for param in func.parameters.iter() {
                self.locals.insert(param.0.clone(), *param.1.clone());
            }

            self.current_func = Some(func.clone());

            self.check_stmt(func.body.as_ref().unwrap());
            self.locals.clear();
        }

        for (_, methods) in self.methods.clone().iter() {
            for method in methods.iter() {
                if method.body.is_none() {
                    continue;
                }

                self.locals.clear();
                for param in method.parameters.iter() {
                    self.locals.insert(param.0.clone(), *param.1.clone());
                }
                self.current_func = Some(method.clone());
                self.locals.insert(
                    method.this.as_ref().unwrap().0.clone(),
                    *method.this.as_ref().unwrap().1.clone(),
                );

                self.check_stmt(method.body.as_ref().unwrap());
                self.locals.clear();
            }
        }

        /*for elem in elements.iter() {
            match elem {
                Element::Func(func) => {
                    if func.body.is_none() {
                        continue;
                    }
                    if func.this.is_none() &&  self.functions.contains_key(&func.name) {
                        let real_func = self.functions.get(&func.name).unwrap().clone();

                        self.locals.clear();
                        for param in real_func.parameters.iter() {
                            self.locals.insert(param.0.clone(),*param.1.clone());
                        }

                        self.check_stmt(real_func.body.as_ref().unwrap());
                        self.locals.clear();
                    } else if func.this.is_some() {


                        let this: (String,Box<Type>)= func.this.as_ref().unwrap().clone();
                        let this_ty = self.infer_type(&this.1);
                        let funcs = this_ty.get
                        self.locals.clear();

                        self.locals.insert(this.0,this_ty.clone());
                        for param in func.parameters.into_iter() {
                            self.locals.insert(param.0.clone(),*param.1.clone());
                        }

                        self.check_stmt(func.body.as_ref().unwrap());
                    }
                }
                _ => ()
            }
        };*/
    }

    pub fn check_expr(&mut self, expr: &Expr) -> Type {
        let pos = expr.pos.clone();
        match &expr.kind {
            ExprKind::Integer(_, suffix) => {
                use crate::lexer::IntSuffix;
                let kind = match suffix {
                    IntSuffix::Byte => TypeKind::Basic("byte".to_owned()),
                    IntSuffix::UByte => TypeKind::Basic("ubyte".to_owned()),
                    IntSuffix::Int => TypeKind::Basic("int".to_owned()),
                    IntSuffix::Long => TypeKind::Basic("long".to_owned()),
                    IntSuffix::ULong => TypeKind::Basic("ulong".to_owned()),
                    IntSuffix::UInt => TypeKind::Basic("uint".to_owned()),
                };

                let ty = Type::new(pos, kind);
                self.type_info.insert(expr.id, ty.clone());
                return ty;
            }
            ExprKind::Float(_, suffix) => {
                use crate::lexer::FloatSuffix;
                let kind = match suffix {
                    FloatSuffix::Float => TypeKind::Basic("float32".to_owned()),
                    FloatSuffix::Double => TypeKind::Basic("float64".to_owned()),
                };
                let ty = Type::new(pos, kind);
                self.type_info.insert(expr.id, ty.clone());
                return ty;
            }
            ExprKind::String(_) => {
                let char_ty = Type::new(pos.clone(), TypeKind::Basic("char".to_owned()));
                let ty = Type::new(pos.clone(), TypeKind::Pointer(box char_ty));
                self.type_info.insert(expr.id, ty.clone());

                return ty;
            }
            ExprKind::Array(values) => {
                if values.len() == 0 {
                    error!("Cannot infer array type", pos);
                } else {
                    let subty = self.check_expr(&values[0]);
                    if values.len() > 1 {
                        for val in values.iter() {
                            let val_ty = self.check_expr(val);
                            if val_ty != subty {
                                error!(
                                    &format!(
                                        "Expected `{}` type in array,found `{}`",
                                        subty, val_ty
                                    ),
                                    val.pos
                                );
                            }
                        }
                    }

                    let ty = Type::new(pos, TypeKind::Array(box subty, Some(values.len() as _)));
                    self.type_info.insert(expr.id, ty.clone());
                    return ty;
                }
            }
            ExprKind::Assign(to, from) => {
                let ty_to = self.check_expr(to);
                let ty_from = self.check_expr(from);
                if ty_to != ty_from {
                    error!(&format!("can not assign `{}` to `{}`", ty_from, ty_to), pos)
                }
                self.type_info.insert(expr.id, ty_from.clone());
                return ty_to;
            }
            ExprKind::Binary(op, lhs, rhs) => {
                let ty1 = self.check_expr(lhs);
                let ty2 = self.check_expr(rhs);

                if ty1.is_basic() && ty2.is_basic() {
                    let op: &str = op;
                    match op {
                        x if ["==", "!=", ">", "<", ">=", "<=", "||", "&&"].contains(&x) => {
                            let ty = Type::new(pos.clone(), TypeKind::Basic("bool".to_owned()));
                            self.type_info.insert(expr.id, ty.clone());
                            return ty;
                        }
                        _ => {}
                    }
                    if ty1 != ty2 {
                        error!(
                            &format!(
                                "can not apply `{}` on different values of `{}` and `{}`",
                                op, ty1, ty2
                            ),
                            pos
                        );
                    }
                    self.type_info.insert(expr.id, ty1.clone());
                    return ty1;
                } else if ty1.is_pointer()
                    && ty2.is_basic_names(&[
                        "int", "uint", "ulong", "long", "short", "ushort", "ubyte", "byte",
                        "usize", "isize",
                    ])
                {
                    self.type_info.insert(expr.id, ty1.clone());
                    return ty1;
                } else {
                    error!(
                        &format!("can not apply `{}` on `{}` and `{}`", op, ty1, ty2),
                        pos
                    );
                }
            }

            ExprKind::Conv(from, to) => {
                self.check_expr(from);
                let ty = self.infer_type(&to);
                self.type_info.insert(expr.id, ty.clone());

                return ty;
            }
            ExprKind::Unary(_, lhs) => {
                let ty = self.check_expr(lhs);
                assert!(ty.is_basic());

                return ty;
            }
            ExprKind::Subscript(array, index) => {
                let ty = self.check_expr(array);
                let index_ty = self.check_expr(index);
                if ty.is_array() || ty.is_pointer() {
                    if !index_ty.is_basic_names(&[
                        "int", "uint", "ulong", "long", "short", "ushort", "ubyte", "byte",
                        "usize", "isize",
                    ]) {
                        error!(&format!("expected integer type,found `{}`", index_ty), pos);
                    }
                    self.type_info
                        .insert(expr.id, ty.get_subty().unwrap().clone());

                    return ty.get_subty().unwrap().clone();
                } else {
                    error!(&format!("Can not apply subscript on `{}` ", ty), pos);
                }
            }
            ExprKind::Identifier(name) => {
                if self.locals.contains_key(name) {
                    let ty = self.locals.get(name).unwrap().clone();
                    self.type_info.insert(expr.id, ty.clone());

                    return ty;
                } else if self.globals.contains_key(name) {
                    let ty = self.globals.get(name).unwrap().0.clone();
                    self.type_info.insert(expr.id, ty.clone());

                    return ty;
                } else if self.functions.contains_key(name) {
                    let func: Function = self.functions.get(name).unwrap().clone();
                    let ty_kind = TypeKind::Function(
                        func.returns.clone(),
                        func.parameters.iter().map(|(_, ty)| ty.clone()).collect(),
                    );
                    let ty = Type::new(pos.clone(), ty_kind);

                    self.type_info.insert(expr.id, ty.clone());

                    return ty;
                } else {
                    error!(&format!("not found `{}`", name), pos);
                }
            }
            ExprKind::Call(func, object, arguments) => {
                if self.functions.contains_key(func) && object.is_none() {
                    let func = self.functions.get(func).unwrap().clone();
                    if !func.variadic {
                        if arguments.len() < func.parameters.len() {
                            error!("not enough arguments", pos);
                        } else if arguments.len() > func.parameters.len() {
                            error!("too much arguments", pos);
                        }
                    }
                    for (i, param) in func.parameters.into_iter().enumerate() {
                        if param.1.is_interface() {
                            let expr_ty = self.check_expr(&arguments[i]);
                            let name = if let TypeKind::Interface(name, _) = &param.1.kind {
                                name.clone()
                            } else {
                                unreachable!();
                            };
                            let interface = self.interfaces.get(&name).unwrap().clone();
                            if !self.ty_impls_interface(&expr_ty, &interface) {
                                error!(
                                    &format!(
                                        "`{}` does not implements `{}` interface",
                                        expr_ty, param.1
                                    ),
                                    pos
                                );
                            }
                        } else {
                            let expr_ty = self.check_expr(&arguments[i]);
                            if expr_ty != *param.1 {
                                error!(
                                    &format!("expected `{}` type,found `{}`", param.1, expr_ty),
                                    pos
                                );
                            }
                        }
                    }

                    self.type_info.insert(expr.id, *func.returns.clone());

                    return *func.returns;
                } else if object.is_some() {
                    let object: Type = self.check_expr(&*object.as_ref().unwrap());
                    let obj_ty = if object.is_pointer() | object.is_option() {
                        object.get_subty().unwrap().clone()
                    } else if object.is_struct() || object.is_basic() {
                        
                        object
                    } else {
                        error!(
                            &format!(
                                "Expected structure or pointer to struct type,found: `{}`",
                                object
                            ),
                            pos
                        );
                    };

                    let methods = self.methods.get(&format!("{}",obj_ty)).clone();
                    
                    if methods.is_none() {
                        error!(&format!("type `{}` does not have any method", obj_ty), pos);
                    }
                    let methods = methods.unwrap().clone();
                    for method in methods.into_iter() {
                        if &method.name == func {
                            let tys = method
                                .parameters
                                .iter()
                                .map(|(_, x)| *x.clone())
                                .collect::<Vec<_>>();
                            let argument_tys = arguments
                                .iter()
                                .map(|x| self.check_expr(x))
                                .collect::<Vec<_>>();

                            if tys == argument_tys {
                                self.type_info.insert(expr.id, *method.returns.clone());
                                return *method.returns.clone();
                            }
                        }
                    }
                    error!("Method not found", pos);
                } else {
                    error!(&format!("function `{}` not found", func), pos);
                }
            }
            ExprKind::Member(object, name) => {
                let ty = self.check_expr(object);
                let ty = if ty.is_struct() {
                    ty
                } else if ty.is_pointer() {
                    ty.get_subty().unwrap().clone()
                } else {
                    unimplemented!();
                };
                if let TypeKind::Structure(_name_, fields) = &ty.kind {
                    for field in fields.iter() {
                        if field.0 == *name {
                            self.type_info.insert(expr.id, *field.1.clone());
                            return *field.1.clone();
                        }
                    }

                    error!(&format!("field `{}` not found on `{}` type", name, ty), pos);
                } else {
                    println!("{}", ty);
                    unimplemented!()
                }
            }
            ExprKind::Bool(_) => {
                let ty = Type::new(pos.clone(), TypeKind::Basic("bool".to_owned()));
                self.type_info.insert(expr.id, ty.clone());
                return ty;
            }
            ExprKind::AddrOf(val) => {
                let val_ty = self.check_expr(val);

                let ty = Type::new(pos.clone(), TypeKind::Pointer(box val_ty));

                self.type_info.insert(expr.id, ty.clone());

                return ty;
            }
            ExprKind::Deref(deref) => {
                let val_ty = self.check_expr(deref);
                assert!(val_ty.is_pointer() || val_ty.is_option());
                self.type_info
                    .insert(expr.id, val_ty.get_subty().unwrap().clone());
                return val_ty.get_subty().clone().unwrap().clone();
            }
            ExprKind::SizeOf(_) => {
                let ty = Type::new(pos, TypeKind::Basic("usize".to_owned()));
                self.type_info.insert(expr.id, ty.clone());
                return ty;
            }
            ExprKind::StructConstruct(name, fields) => {
                let ty: CStructure = self.structures.get(name).unwrap().clone();
                assert!(ty.fields.len() == fields.len());
                for (i, field) in fields.iter().enumerate() {
                    let field: (String, Box<Expr>) = field.clone();
                    if field.0 != ty.fields[i].0 {
                        unimplemented!() // TODO: Print error
                    }
                    let vty = self.check_expr(&field.1);
                    if vty != ty.fields[i].1 {
                        error!(
                            &format!(
                                "field `{}` expected `{}` type,found `{}`",
                                field.0, ty.fields[i].1, vty
                            ),
                            pos
                        );
                    }
                }

                self.type_info.insert(expr.id, ty.ty.clone());

                return ty.ty.clone();
            }
            _ => unimplemented!(),
        };
    }

    pub fn check_stmt(&mut self, s: &StmtKind) {
        match s {
            StmtKind::Return(val) => {
                if val.is_none() && self.current_func.as_ref().unwrap().returns.is_void() {
                    return;
                }
                let vty = self.check_expr(val.as_ref().unwrap());
                let ty = self.current_func.as_ref().unwrap().returns.clone();

                if *ty != vty {
                    error!(&format!("expected `{}` type,found `{}`", ty, vty), vty.pos);
                }

                return;
            }
            StmtKind::Block(block) => {
                let old_scope = self.locals.clone();
                for stmt in block.iter() {
                    self.check_stmt(stmt);
                }

                self.locals = old_scope;
            }
            StmtKind::Expr(expr) => {
                self.check_expr(expr);
            }
            StmtKind::Continue => {}
            StmtKind::Break => {}
            StmtKind::While(cond, then) => {
                self.check_expr(cond);
                self.check_stmt(then);
            }
            StmtKind::For(decl, cond, then, block) => {
                let old_scope = self.locals.clone();
                self.check_stmt(decl);
                self.check_expr(cond);
                self.check_expr(then);
                self.check_stmt(block);
                self.locals = old_scope;
            }
            StmtKind::If(cond, then, otherwise) => {
                self.check_expr(cond);
                self.check_stmt(then);
                if otherwise.is_some() {
                    self.check_stmt(&otherwise.as_ref().unwrap());
                }
            }
            StmtKind::Switch(_, _) => unimplemented!(),
            StmtKind::VarDecl(name, ty, val) => {
                if self.locals.contains_key(name) {
                    error!(
                        &format!("variable `{}` exists", name),
                        self.current_func.as_ref().unwrap().pos.clone()
                    );
                }

                let var_ty = if ty.is_some() && val.is_some() {
                    let vty = self.check_expr(&val.clone().unwrap());
                    let ty = ty.as_ref().unwrap().clone();
                    let ty_ = self.infer_type(&ty);
                    if vty.is_void() {
                        error!(&format!("invalid use of void type"),vty.pos);
                    }
                    if vty != ty_ {
                        error!(&format!("can not assign `{}` to `{}`", vty, ty_), vty.pos);
                    }
                    ty_
                } else if val.is_some() && ty.is_none() {
                    let vty = self.check_expr(&val.clone().unwrap());
                    if vty.is_void() {
                        error!(&format!("invalid use of void type"),vty.pos);
                    }
                    vty
                } else if ty.is_some() {
                    self.infer_type(&ty.as_ref().unwrap())
                } else {
                    unreachable!()
                };

                self.locals.insert(name.to_owned(), var_ty);
            }
        }
    }

    pub fn infer_type(&mut self, ty: &Type) -> Type {
        match &ty.kind {
            TypeKind::Basic(name) => {
                if self.structures.contains_key(name) {
                    self.complex.insert(
                        name.to_owned(),
                        self.structures.get(name).unwrap().ty.clone(),
                    );
                    return self.structures.get(name).unwrap().ty.clone();
                } else if let Some(interface) = self.interfaces.get(name) {
                    /* return Type::new(
                        interface.pos.clone(),
                        TypeKind::Interface(
                            interface.name.clone(),
                            interface
                                .functions
                                .iter()
                                .map(|func| {
                                    (
                                        func.name.clone(),
                                        func.parameters
                                            .iter()
                                            .map(|(name, ty)| {
                                                (name.clone(), box self.infer_type(&ty.clone()))
                                            })
                                            .collect::<Vec<_>>(),
                                        box self.infer_type(&func.returns),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        ),
                    );*/
                    unimplemented!()
                } else {
                    return ty.clone();
                }
            }
            TypeKind::Pointer(to) => {
                return Type::new(ty.pos.clone(), TypeKind::Pointer(box self.infer_type(to)))
            }
            TypeKind::Void => return ty.clone(),
            TypeKind::Function(ret, params) => {
                return Type::new(
                    ty.pos.clone(),
                    TypeKind::Function(
                        box self.infer_type(ret),
                        params
                            .iter()
                            .map(|x| box self.infer_type(x))
                            .collect::<Vec<_>>(),
                    ),
                )
            }
            TypeKind::Array(subty, len) => {
                return Type::new(
                    ty.pos.clone(),
                    TypeKind::Array(box self.infer_type(subty), *len),
                )
            }
            TypeKind::Structure(name, _) | TypeKind::Interface(name, _) => {
                self.complex.insert(name.to_owned(), ty.clone());
                return ty.clone();
            }
            TypeKind::Optional(optional) => Type::new(
                ty.pos.clone(),
                TypeKind::Optional(box self.infer_type(optional)),
            ),
        }
    }
}
