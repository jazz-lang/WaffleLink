use std::collections::HashMap;

pub struct CCodeGen {
    pub buffer: String,
    pub ty_info: HashMap<usize, Type>,
    pub variables: HashMap<String, Type>,
    pub complex_types: HashMap<String, Type>,
}

static mut FN_TY_C: i32 = 0;
static mut FUNC_TYPES: Option<String> = None;
use super::ast::*;

impl CCodeGen {
    pub fn new() -> CCodeGen {
        CCodeGen {
            buffer: String::new(),
            ty_info: HashMap::new(),
            variables: HashMap::new(),
            complex_types: HashMap::new(),
        }
    }
}

fn ty_to_c(ty: &Type) -> String {
    match &ty.kind {
        TypeKind::Basic(basic) => basic.to_owned(),
        TypeKind::Structure(name, _) => name.to_owned(),
        TypeKind::Pointer(to) => format!("{}*", ty_to_c(to)),
        TypeKind::Void => "void".to_owned(),
        TypeKind::Optional(ty) => format!("{}*", ty_to_c(ty)),
        TypeKind::Function(returns, params) => {
            let ty = unsafe { FUNC_TYPES.as_mut().unwrap() };
            ty.push_str("typedef ");
            ty.push_str(&ty_to_c(returns));
            ty.push_str(" ");
            ty.push_str(&format!("(*_{})", unsafe { FN_TY_C }));
            unsafe {
                FN_TY_C += 1;
            }
            ty.push_str("(");
            for (i, param) in params.iter().enumerate() {
                ty.push_str(&ty_to_c(param));
                if i != params.len() - 1 {
                    ty.push_str(", ");
                }
            }
            ty.push_str(");\n");
            format!("{}", unsafe { FN_TY_C - 1 })
        }
        _ => unimplemented!(),
    }
}

impl CCodeGen {
    fn get_ty(&self, ty: &Type) -> Type {
        if ty.is_basic() {
            if let TypeKind::Basic(name) = &ty.kind {
                if self.complex_types.contains_key(name) {
                    return self.complex_types.get(name).unwrap().clone();
                }
            }
        }

        return ty.clone();
    }

    pub fn write(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    pub fn gen_toplevel(&mut self, elements: &[Element]) {
        unsafe {
            FUNC_TYPES = Some(String::new());
        }
        for elem in elements.iter() {
            match elem {
                Element::Struct(s) => {
                    self.write(&format!("typedef struct {} {};\n", s.name,s.name));
                }
                _ => (),
            }
        }

        unsafe {
            self.write("\n");
            self.write(FUNC_TYPES.as_ref().unwrap());
            self.write("\n");
        }
        for elem in elements.iter() {
            match elem {
                Element::Func(func) => {
                    let func: &Function = func;

                    if func.external {
                        self.write("\nextern\n");
                    }
                    self.write(&ty_to_c(&func.returns));
                    self.write(&format!(" {} (", func.mangle_name()));
                    if func.this.is_some() {
                        let this = func.this.as_ref().unwrap();
                        self.write(&format!("{}", ty_to_c(&this.1)));
                        if func.parameters.len() != 0 {
                            self.write(", ");
                        }
                    }
                    for (i, (_, ty)) in func.parameters.iter().enumerate() {
                        self.write(&format!("{}", ty_to_c(ty)));
                        if i != func.parameters.len() - 1 {
                            self.write(",");
                        }
                    }
                    if func.variadic {
                        if func.parameters.len() != 0 {
                            self.write(",");
                        }
                        self.write("...");
                    }
                    self.write(");\n");
                }
                Element::Const(constant) => {
                    self.write(&format!("#define {} ", constant.name));
                    self.gen_expr(&constant.value);
                    self.write("\n");
                }
                Element::Var(var) => {
                    self.write(&ty_to_c(&var.ty));
                    self.write(&format!(" {};\n", var.name));
                }

                _ => (),
            }
        }
        for elem in elements.iter() {
            match elem {
                Element::Struct(s) => {
                    let s: &Struct = s;
                    self.write(&format!("typedef struct {} {{\n", s.name));
                    for (_, name, ty) in s.fields.iter() {
                        self.write(&ty_to_c(ty));
                        self.write(&format!(" {};\n", name));
                    }
                    self.write(&format!("}} {};\n",s.name));
                }
                _ => (),
            }
            
        }
        for elem in elements.iter() {
            match elem {
                
                Element::Func(func) => {
                    let func: &Function = func;
                    if func.external || func.body.is_none() {
                        continue;
                    }
                    
                    self.write(&ty_to_c(&func.returns));
                    self.write(&format!(" {} (", func.mangle_name()));
                    if func.this.is_some() {
                        let this = func.this.as_ref().unwrap();
                        self.write(&format!("{} {}", ty_to_c(&this.1), this.0));
                        if func.parameters.len() != 0 {
                            self.write(", ");
                        }
                    }
                    for (i, (name, ty)) in func.parameters.iter().enumerate() {
                        self.write(&format!("{} {}", ty_to_c(ty), name));
                        if i != func.parameters.len() {
                            self.write(",");
                        }
                    }
                    self.write(")\n");
                    self.gen_statement(func.body.as_ref().unwrap());
                    self.write("\n");
                }
                _ => (),
            }
        }
    }

    fn gen_statement(&mut self, s: &StmtKind) {
        match s {
            StmtKind::Expr(expr) => {
                self.gen_expr(expr);
                self.write(";\n");
            }
            StmtKind::Block(stmts) => {
                self.write("{\n");
                for stmt in stmts.iter() {
                    self.gen_statement(stmt);
                }
                self.write("}\n");
            }
            StmtKind::Break => self.write("break"),
            StmtKind::Continue => self.write("continue"),
            StmtKind::If(cond, then, or) => {
                self.write("if (");
                self.gen_expr(cond);
                self.write(")\n{");
                self.gen_statement(then);
                self.write("}");
                if or.is_some() {
                    self.write("else ");
                    self.gen_statement(or.as_ref().unwrap());
                }
            }
            StmtKind::VarDecl(name, ty, val) => {
                let c_ty = if ty.is_some() {
                    ty_to_c(ty.as_ref().unwrap())
                } else {
                    let ty_info = self.ty_info.get(&val.as_ref().unwrap().id).unwrap().clone();
                    ty_to_c(&ty_info)
                };

                self.write(&c_ty);
                self.write(&format!(" {}", name));
                self.write("=");
                if val.is_none() {
                    self.write(";\n");
                } else {
                    self.gen_expr(val.as_ref().unwrap());
                    self.write(";\n");
                }
            }
            StmtKind::While(cond, then) => {
                self.write("while (");
                self.gen_expr(cond);
                self.write(") \n{");
                self.gen_statement(then);
                self.write("}\n");
            }
            StmtKind::Return(val) => {
                if val.is_none() {
                    self.write("return;");
                } else {
                    self.write("return ");
                    self.gen_expr(val.as_ref().unwrap());
                    self.write(";");
                }
            }
            _ => unimplemented!(),
        }
    }

    fn gen_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Array(values) => {
                self.write("{ ");
                for (i, value) in values.iter().enumerate() {
                    self.gen_expr(value);
                    if i != values.len() {
                        self.write(", ");
                    }
                }
                self.write("} ");
            }
            ExprKind::Binary(op, lhs, rhs) => {
                self.gen_expr(lhs);
                self.write(&format!(" {} ", op));
                self.gen_expr(rhs);
            }
            ExprKind::Unary(op, lhs) => {
                self.write("-");
                self.gen_expr(lhs);
            }
            ExprKind::Identifier(ident) => {
                self.write(&format!("{}", ident));
            }
            ExprKind::Member(object, field) => {
                let ty = self.ty_info.get(&object.id).unwrap().clone();
                self.gen_expr(object);
                if ty.is_pointer() {
                    self.write("->");
                } else {
                    self.write(".");
                }
                self.write(field);
            }
            ExprKind::Integer(val, suffix) => {
                use crate::lexer::IntSuffix;

                match suffix {
                    IntSuffix::Long => self.write(&format!("{}LL", val)),
                    IntSuffix::ULong => self.write(&format!("{}ULL", val)),
                    _ => self.write(&format!("{}", val)),
                }
            }
            ExprKind::Float(val, suffix) => {
                use crate::lexer::FloatSuffix;
                match suffix {
                    FloatSuffix::Float => {
                        self.write(&format!("{}F", val));
                    }
                    _ => self.write(&format!("{}", val)),
                }
            }
            ExprKind::String(str) => {
                self.write(&format!("{:?}", str));
            }
            ExprKind::Character(character) => {
                self.write(&format!("{:?}", character));
            }
            ExprKind::Bool(boolean) => {
                self.write(&format!("{}", boolean));
            }
            ExprKind::Conv(value, to) => {
                self.write(&format!("({})", ty_to_c(to)));
                self.gen_expr(value);
            }
            ExprKind::Deref(value) => {
                self.write("*");
                self.gen_expr(value);
            }
            ExprKind::Null => {
                self.write("NULL");
            }
            ExprKind::SizeOf(ty) => {
                self.write(&format!("sizeof({})", ty_to_c(ty)));
            }
            ExprKind::Subscript(val, idx) => {
                self.gen_expr(val);
                self.write("[");
                self.gen_expr(idx);
                self.write("]");
            }
            ExprKind::Undefined => {}
            ExprKind::StructConstruct(_, fields) => {
                self.write("{");
                for (i, field) in fields.iter().enumerate() {
                    self.write(&format!(".{} = ", field.0));
                    self.gen_expr(&field.1);
                    if i != fields.len() - 1 {
                        self.write(", ");
                    }
                }
                self.write("}");
            }
            ExprKind::AddrOf(expr) => {
                self.write("&");
                self.gen_expr(expr);
            }
            ExprKind::Call(name, this, arguments) => {
                let name = if this.is_some() {
                    let ty = self
                        .ty_info
                        .get(&this.as_ref().unwrap().id)
                        .unwrap()
                        .clone();
                    if ty.is_pointer() {
                        format!("this{}_{}", ty.get_subty().unwrap(), name)
                    } else {
                        format!("this{}_{}", ty, name)
                    }
                } else {
                    name.to_owned()
                };

                self.write(&format!("{}(", name));
                if this.is_some() {
                    let this = this.as_ref().unwrap().clone();
                    let ty = self.ty_info.get(&this.id).unwrap().clone();
                    if ty.is_pointer() {
                        self.gen_expr(&this);
                    } else {
                        self.write("&");
                        self.gen_expr(&this);

                        if arguments.len() != 0 {
                            self.write(",");
                        }
                    }
                }
                for (i, argument) in arguments.iter().enumerate() {
                    self.gen_expr(argument);
                    if i != arguments.len() - 1 {
                        self.write(",")
                    };
                }

                self.write(")");
            }

            ExprKind::Assign(to, from) => {
                self.gen_expr(to);
                self.write(" = ");
                self.gen_expr(from);
            }
            _ => unimplemented!(),
        }
    }
}
