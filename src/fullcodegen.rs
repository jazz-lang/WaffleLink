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

    pub fn compile(&mut self, module: WaffleCellPointer<BCModule>, code: Vec<Vec<Vec<Opcode>>>) {
        for ix in 0..module.functions.len() {
            let code = &code[ix];
            self.ctx.clear();
            self.ctx
                .func
                .signature
                .returns
                .extend(&[AbiParam::new(types::I8), AbiParam::new(types::I64)]);
            self.ctx
                .func
                .signature
                .params
                .push(AbiParam::new(types::I64));
            // Create the builder to builder a function.
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);
            // Create the entry block, to start emitting code in.
            let mut trans = FullCodegenTranslator {
                callframe: Value::from_u32(0),
                builder,
                module: &mut self.module,
                bc_module: module,
            };
            trans.compile(code);
            trans.builder.seal_all_blocks();
            trans.builder.finalize();
            self.ctx.compute_cfg();
            self.ctx.compute_domtree();
            self.ctx.compute_loop_analysis();
            self.ctx.licm(self.module.isa()).unwrap();
            self.ctx.preopt(self.module.isa()).unwrap();
            self.ctx.simple_gvn(self.module.isa()).unwrap();
            self.ctx
                .eliminate_unreachable_code(self.module.isa())
                .unwrap();
            self.ctx.dce(self.module.isa()).unwrap();
            self.ctx.remove_constant_phis(self.module.isa()).unwrap();
            self.ctx.regalloc(self.module.isa()).unwrap();
            self.ctx
                .redundant_reload_remover(self.module.isa())
                .unwrap();
            println!("{}", self.ctx.func.display(None));
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
        let i = self.builder.ins().iconst(types::I64, V::NUMBER_TAG);
        let x = self.builder.ins().band(val, i);
        self.builder.ins().icmp_imm(IntCC::Equal, x, V::NUMBER_TAG)
    }
    pub fn is_not_int32(&mut self, val: Value) -> Value {
        let i = self.builder.ins().iconst(types::I64, V::NUMBER_TAG);
        let x = self.builder.ins().band(val, i);
        self.builder
            .ins()
            .icmp_imm(IntCC::NotEqual, x, V::NUMBER_TAG)
    }

    pub fn is_number(&mut self, value: Value) -> Value {
        let i = self.builder.ins().iconst(types::I64, V::NUMBER_TAG);
        let x = self.builder.ins().band(value, i);
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

    pub fn compile(&mut self, code: &Vec<Vec<Opcode>>) {
        let bc = &code;
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let entry = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry);
        for (i, _block) in bc.iter().enumerate() {
            let bb = self.builder.create_block();
            map.insert(i, bb);
        }
        self.callframe = self.builder.block_params(entry)[0];
        self.builder.switch_to_block(entry);
        self.builder.ins().jump(map[&0], &[]);

        let mut slow_paths: Vec<Box<dyn FnOnce(&mut Self)>> = vec![];
        for (i, block) in bc.iter().enumerate() {
            self.builder.switch_to_block(map[&i]);
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
                    Opcode::Ret(r) => {
                        let v = self.load_register(r);
                        let ok = self.builder.ins().iconst(types::I8, 1);
                        self.builder.ins().return_(&[ok, v]);
                    }
                    Opcode::Add(dst, lhs, rhs) => {
                        let lhs = self.load_register(lhs);
                        let rhs = self.load_register(rhs);
                        let res = self.gen_binop(&mut slow_paths, BinOp::Add, lhs, rhs);
                        self.store_reg(dst, res);
                    }
                    _ => todo!(),
                }
            }
        }
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
                let c = self.builder.create_block();
                let c2 = self.builder.create_block();
                let c3 = self.builder.create_block();
                let c4 = self.builder.create_block();
                let undef = self.builder.create_block();
                let end = self.builder.create_block();
                self.builder.append_block_param(end, types::I64);
                let b1 = self.is_not_number(x);
                self.builder.ins().brnz(b1, undef, &[]);
                self.builder.ins().fallthrough(c, &[]);
                self.builder.switch_to_block(c);
                let b2 = self.is_not_number(y);
                self.builder.ins().brnz(b2, undef, &[]);
                self.builder.ins().fallthrough(c2, &[]);
                self.builder.switch_to_block(c2);
                let i1 = self.is_not_int32(x);
                self.builder.ins().brnz(i1, double_op, &[]);
                self.builder.ins().fallthrough(c3, &[]);
                self.builder.switch_to_block(c3);
                let i1_2 = self.is_not_int32(y);
                self.builder.ins().brnz(i1_2, double_op, &[]);
                self.builder.ins().fallthrough(c4, &[]);
                self.builder.switch_to_block(c4);
                let rx = self.builder.ins().ireduce(types::I32, x);
                let ry = self.builder.ins().ireduce(types::I32, y);
                let r = self.builder.ins().iadd(rx, ry);
                let rhs_is_negative = self.builder.ins().icmp_imm(IntCC::SignedLessThan, ry, 0);
                let slt = self.builder.ins().icmp(IntCC::SignedLessThan, r, rx);
                let ovf = self.builder.ins().bxor(rhs_is_negative, slt);
                self.builder.ins().brz(ovf, end, &[r]);
                self.builder.ins().fallthrough(double_op, &[]);
                self.builder.switch_to_block(double_op);
                let rx = self.as_double(x);
                let ry = self.as_double(y);
                let r = self.builder.ins().fadd(rx, ry);
                let r = self.new_number(r);
                self.builder.ins().jump(end, &[r]);
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
            BinOp::Sub => {
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
                let r = self.builder.ins().isub(rx, ry);
                let rhs_is_negative = self.builder.ins().icmp_imm(IntCC::SignedLessThan, ry, 0);
                let slt = self.builder.ins().icmp(IntCC::SignedGreaterThan, r, rx);
                let ovf = self.builder.ins().bxor(rhs_is_negative, slt);
                self.builder.ins().brz(ovf, end, &[r]);
                self.builder.ins().fallthrough(double_op, &[]);
                self.builder.switch_to_block(double_op);
                let rx = self.as_double(x);
                let ry = self.as_double(y);
                let r = self.builder.ins().fsub(rx, ry);
                let r = self.new_number(r);
                self.builder.ins().jump(end, &[r]);
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
            BinOp::Div => {
                let undef = self.builder.create_block();
                let end = self.builder.create_block();
                let rn = self.is_not_number(x);
                self.builder.append_block_param(end, types::I64);
                self.builder.ins().brnz(rn, undef, &[]);
                let rn = self.is_not_number(y);
                self.builder.ins().brnz(rn, undef, &[]);
                let not_int = self.builder.create_block();
                let skip = self.builder.create_block();
                let cond = self.is_not_int32(x);
                self.builder.ins().brnz(cond, not_int, &[]);
                let i = self.as_int32(x);
                let i = self.builder.ins().sextend(types::I64, i);
                let f = self.builder.ins().fcvt_from_sint(types::F64, i);
                self.builder.append_block_param(skip, types::F64);
                self.builder.ins().jump(skip, &[f]);
                self.builder.switch_to_block(not_int);
                let f = self.as_double(x);
                self.builder.ins().fallthrough(skip, &[f]);
                self.builder.switch_to_block(skip);
                let not_int = self.builder.create_block();
                let skip2 = self.builder.create_block();
                self.builder.append_block_param(skip2, types::F64);
                self.builder.append_block_param(skip2, types::F64);
                let cond = self.is_not_int32(y);
                self.builder.ins().brnz(cond, not_int, &[]);
                let d = self.builder.block_params(skip)[0];
                let i = self.as_int32(y);
                let i = self.builder.ins().sextend(types::I64, i);
                let f = self.builder.ins().fcvt_from_sint(types::F64, i);
                self.builder.ins().jump(skip2, &[f, d]);
                self.builder.switch_to_block(not_int);
                let f = self.as_double(x);
                self.builder.ins().fallthrough(skip2, &[f, d]);
                self.builder.switch_to_block(skip2);
                let xf = self.builder.block_params(skip2)[0];
                let xy = self.builder.block_params(skip2)[1];
                let res = self.builder.ins().fadd(xf, xy);
                let res = self.new_number(res);
                self.builder.ins().jump(end, &[res]);
                self.builder.switch_to_block(undef);
                let undef = self
                    .builder
                    .ins()
                    .iconst(types::I64, unsafe { V::undefined().u.as_int64 });
                self.builder.ins().fallthrough(end, &[undef]);
                self.builder.switch_to_block(end);
                let res = self.builder.block_params(end)[0];
                // todo: try to build i32 there.
                return res;
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
