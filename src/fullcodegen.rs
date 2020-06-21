use crate::function::Function;
use crate::opcode::*;
use crate::{module::Module as BCModule, object::WaffleCellPointer};
pub(super) use cranelift::prelude::*;
pub(super) use cranelift_module::Module;
pub(super) use cranelift_module::{DataContext, Linkage};
pub(super) use cranelift_simplejit::*;
pub mod loads;
pub mod stores;
pub struct FullCodegen {
    pub module: Module<SimpleJITBackend>,
    pub builder_ctx: FunctionBuilderContext,
    pub ctx: codegen::Context,
    pub data_ctx: DataContext,
}

impl FullCodegen {
    pub fn new() -> Self {
        let mut flags_builder = cranelift::codegen::settings::builder();
        flags_builder.set("opt_level", "speed_and_size").unwrap();
        flags_builder.set("use_colocated_libcalls", "true").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder.finish(settings::Flags::new(flags_builder));
        let builder = SimpleJITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = Module::new(builder);
        Self {
            builder_ctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
        }
    }
}
use crate::value::Value as V;
pub struct FullCodegenTranslator<'a> {
    builder: FunctionBuilder<'a>,
    module: &'a mut Module<SimpleJITBackend>,
    bc_module: WaffleCellPointer<BCModule>,
    callframe: Value,
}
impl<'a> FullCodegenTranslator<'a> {
    pub fn build_boolean(&mut self, x: bool) -> Value {
        #[cfg(target_pointer_width = "64")]
        {
            if x {
                self.builder.ins().iconst(types::I64, V::VALUE_TRUE)
            } else {
                self.builder.ins().iconst(types::I64, V::VALUE_FALSE)
            }
        }

        #[cfg(target_pointer_widht = "32")]
        {
            todo!();
        }
    }

    pub fn as_double(&mut self, val: Value) -> Value {
        /*let ic = self.builder.ins().iconst(types::I64, -562949953421312);
        let x = self.builder.ins().iadd(val, ic);
        self.builder.ins().bitcast(types::F64, x)*/
        let ic = self
            .builder
            .ins()
            .iconst(types::I64, V::DOUBLE_ENCODE_OFFSET);
        let x = self.builder.ins().isub(val, ic);
        self.builder.ins().bitcast(types::F64, x)
    }

    pub fn new_int(&mut self, val: Value) -> Value {
        //let ic = self.builder.ins().iconst(types::I64, -562949953421312);
        //let x = self.builder.ins().bor(val, ic);
        let x = self.builder.ins().ireduce(types::I32, val);
        let y = self.builder.ins().sextend(types::I64, x);
        self.builder.ins().bor_imm(y, V::NUMBER_TAG)
    }
    pub fn as_int32(&mut self, val: Value) -> Value {
        self.builder.ins().ireduce(types::I32, val)
    }
    pub fn is_double(&mut self, val: Value) -> Value {
        /*let x = self.builder.ins().iadd_imm(val, -562949953421312);
        let x = self.builder.ins().ushr_imm(x, 50);
        let x = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThan, x, 16383);
        x*/
        let x = self.is_int32(val);
        let y = self.is_number(val);
        let z = self.builder.ins().icmp_imm(IntCC::Equal, x, 0);
        self.builder.ins().band(y, z)
    }

    pub fn is_int32(&mut self, val: Value) -> Value {
        let x = self.builder.ins().band_imm(val, V::NUMBER_TAG);
        self.builder.ins().icmp_imm(IntCC::Equal, x, V::NUMBER_TAG)
    }
    pub fn is_not_int32(&mut self, val: Value) -> Value {
        let x = self.builder.ins().band_imm(val, V::NUMBER_TAG);
        self.builder
            .ins()
            .icmp_imm(IntCC::NotEqual, x, V::NUMBER_TAG)
    }

    pub fn is_number(&mut self, value: Value) -> Value {
        let x = self.builder.ins().band_imm(value, V::NUMBER_TAG);
        self.builder.ins().icmp_imm(IntCC::NotEqual, x, 0)
    }
    pub fn is_not_number(&mut self, value: Value) -> Value {
        let x = self.is_number(value);
        self.builder.ins().icmp_imm(IntCC::Equal, x, 0)
    }

    pub fn is_true(&mut self, val: Value) -> Value {
        self.builder
            .ins()
            .icmp_imm(IntCC::Equal, val, V::VALUE_TRUE)
    }

    pub fn is_false(&mut self, val: Value) -> Value {
        self.builder
            .ins()
            .icmp_imm(IntCC::Equal, val, V::VALUE_FALSE)
    }

    pub fn is_null(&mut self, val: Value) -> Value {
        self.builder
            .ins()
            .icmp_imm(IntCC::Equal, val, V::VALUE_NULL)
    }

    pub fn is_undefined(&mut self, val: Value) -> Value {
        unsafe {
            self.builder
                .ins()
                .icmp_imm(IntCC::Equal, val, V::undefined().u.as_int64)
        }
    }

    pub fn is_undefined_or_null(&mut self, val: Value) -> Value {
        let x = self.builder.ins().band_imm(val, !V::UNDEFINED_TAG);
        self.builder.ins().icmp_imm(IntCC::Equal, x, V::VALUE_NULL)
    }
    pub fn is_boolean(&mut self, val: Value) -> Value {
        let x = self.builder.ins().band_imm(val, !V::UNDEFINED_TAG);
        self.builder.ins().icmp_imm(IntCC::Equal, x, V::VALUE_FALSE)
    }
    pub fn is_cell(&mut self, val: Value) -> Value {
        let val = self.builder.ins().band_imm(val, V::NOT_CELL_MASK);
        self.builder.ins().icmp_imm(IntCC::Equal, val, 0)
    }

    pub fn new_number(&mut self, val: Value) -> Value {
        let x = self.builder.ins().bitcast(types::F64, val);
        self.builder.ins().iadd_imm(x, V::DOUBLE_ENCODE_OFFSET)
    }

    pub fn compile(&mut self, code: Vec<Vec<Vec<Opcode>>>) {
        for ix in 0..self.bc_module.value().functions.value().len() {
            let f = self.bc_module.value().functions.value().at(ix);
            debug_assert!(f.is_cell());
            let f = f.as_cell().cast::<Function>();

            let bc = &code[ix as usize];
            use std::collections::HashMap;
            let mut map = HashMap::new();
            let mut slow_paths: Vec<Box<dyn FnOnce(&mut Self)>> = vec![];
            for (i, block) in bc.iter().enumerate() {
                let bb = self.builder.create_block();
                self.builder.switch_to_block(bb);
                map.insert(i, bb);

                for op in block.iter() {
                    match *op {
                        Opcode::Mov(dst, src) => {
                            let r = self.load_register(src);
                            self.store_reg(dst, r);
                        }
                        Opcode::Constant(dst, ix) => {
                            let c = self.load_constant(ix);
                            self.store_reg(dst, c);
                        }
                        Opcode::True(dst) => {
                            let b = self.build_boolean(true);
                            self.store_reg(dst, b);
                        }

                        Opcode::False(dst) => {
                            let b = self.build_boolean(false);
                            self.store_reg(dst, b);
                        }
                        Opcode::Null(dst) => {
                            let v = self.builder.ins().iconst(types::I64, V::VALUE_NULL);
                            self.store_reg(dst, v);
                        }
                        _ => todo!(),
                    }
                }
            }
        }
        let mut slow_paths: Vec<Box<dyn FnOnce(&mut Self)>> = vec![];
    }

    pub fn gen_binop(
        &mut self,
        slow: &mut Vec<Box<dyn FnOnce(&mut Self)>>,
        op: BinOp,
        x: Value,
        y: Value,
    ) -> Value {
        match op {
            BinOp::Add => {
                let double_op = self.builder.create_block();
                let undef = self.builder.create_block();
                let end = self.builder.create_block();
                self.builder.append_block_param(end, types::I64);
                let b1 = self.is_not_number(x);
                self.builder.ins().brnz(b1, undef, &[]);
                let b2 = self.is_not_number(y);
                self.builder.ins().brnz(b2, undef, &[]);
                let i1 = self.is_not_int32(x);
                self.builder.ins().brnz(i1, double_op, &[]);
                let i1_2 = self.is_not_int32(y);
                self.builder.ins().brnz(i1_2, double_op, &[]);
                let rx = self.builder.ins().ireduce(types::I32, x);
                let ry = self.builder.ins().ireduce(types::I32, y);
                let r = self.builder.ins().iadd(rx, ry);
                let rhs_is_negative = self.builder.ins().icmp_imm(IntCC::SignedLessThan, ry, 0);
                let slt = self.builder.ins().icmp(IntCC::SignedLessThan, r, rx);
                let ovf = self.builder.ins().bxor(rhs_is_negative, slt);
                self.builder.ins().brz(ovf, end, &[r]);
                self.builder.ins().fallthrough(double_op, &[]);
                self.builder.switch_to_block(double_op);
                // TODO: Double op
                self.builder.switch_to_block(undef);
                let undef = self
                    .builder
                    .ins()
                    .iconst(types::I64, unsafe { V::undefined().u.as_int64 });
                self.builder.ins().fallthrough(end, &[undef]);

                self.builder.switch_to_block(end);
                let p = self.builder.block_params(end)[0];
                return p;
            }
            _ => (),
        }
        todo!();
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Div,
    Mul,
    Rem,
    Shr,
    Shl,

    Gt,
    Lt,
    Eq,
    Le,
    Ge,
}
