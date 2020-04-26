//! SimpleJIT implementation.
//!
//! The first execution of any function always starts in the interpreter tier. As soon as any statement
//! in the function executes more than 100 times, or the function is called more than 10 times (whichever comes first),
//! execution is diverted into code compiled by the Simple JIT
//!
//!
//! What this Simple JIT does is removes interpreter loop dispatch overhead and compiles bytecode into single stream of
//! machine instructions without interpreter dispatch, this improves performance by helping CPU predicting branches.

use crate::bytecode::*;
use crate::runtime;
use cranelift::{codegen::*, frontend::*, prelude::*};
use cranelift_module::*;
use cranelift_simplejit::*;
use runtime::function::Function as WFunc;
use runtime::function::*;
use runtime::value::Value as CVal;
pub struct WaffleSimpleJIT<'a> {
    function: &'a mut Function,
    builder: FunctionBuilder<'a>,
    module: &'a mut Module<SimpleJITBackend>,
    slow_path_sig: ir::entities::SigRef,
}
impl<'a> WaffleSimpleJIT<'a> {
    pub fn emit_is_int32(&mut self, value: Value) -> Value {
        let x = self.builder.ins().band_imm(value, CVal::NUMBER_TAG);
        self.builder
            .ins()
            .icmp_imm(IntCC::Equal, x, CVal::NUMBER_TAG)
    }
    pub fn emit_not_int32(&mut self, value: Value) -> Value {
        let x = self.builder.ins().band_imm(value, CVal::NUMBER_TAG);
        self.builder
            .ins()
            .icmp_imm(IntCC::NotEqual, x, CVal::NUMBER_TAG)
    }
    pub fn emit_is_number(&mut self, value: Value) -> Value {
        let x = self.builder.ins().band_imm(value, CVal::NUMBER_TAG);
        self.builder.ins().icmp_imm(IntCC::NotEqual, x, 0)
    }

    pub fn emit_as_double(&mut self, value: Value) -> Value {
        let imm = self
            .builder
            .ins()
            .iconst(types::I64, CVal::DOUBLE_ENCODE_OFFSET);
        let x = self.builder.ins().isub(value, imm);
        self.builder.ins().raw_bitcast(types::F64, x)
    }
    pub fn emit_is_double(&mut self, value: Value) -> Value {
        let not_int = self.emit_not_int32(value);
        let is_number = self.emit_is_number(value);
        self.builder.ins().band(not_int, is_number)
    }

    pub fn emit_as_int32(&mut self, value: Value) -> Value {
        self.builder.ins().ireduce(types::I32, value)
    }

    pub fn emit_to_number(&mut self, value: Value) -> Value {
        let if_is_int = self.builder.create_block();
        let if_is_double = self.builder.create_block();
        let slow_path = self.builder.create_block();
        let merge_block = self.builder.create_block();

        self.builder.append_block_param(merge_block, types::F64);

        let is_n = self.emit_is_number(value);
        // if !is_number(value) goto slow_path;
        self.builder.ins().brz(is_n, slow_path, &[]);

        let is_i = self.emit_is_int32(value);
        // if !is_int32(value) goto if_is_double;
        self.builder.ins().brz(is_i, if_is_double, &[]);
        self.builder.ins().jump(if_is_int, &[]);
        self.builder.switch_to_block(if_is_int);
        self.builder.seal_block(if_is_int);
        // value.as_int32() as f64
        let v = self.emit_as_int32(value);
        let x = self.builder.ins().fcvt_from_sint(types::F64, v);
        self.builder.ins().jump(merge_block, &[x]);
        self.builder.switch_to_block(if_is_double);
        self.builder.seal_block(if_is_double);
        let x = self.emit_as_double(value);
        self.builder.ins().jump(merge_block, &[x]);

        self.builder.switch_to_block(slow_path);
        self.builder.seal_block(slow_path);
        let slow_fn = self
            .builder
            .ins()
            .iconst(types::I64, simplejit_to_number_slow as i64);
        let call = self
            .builder
            .ins()
            .call_indirect(self.slow_path_sig, slow_fn, &[value]);
        let x = self.builder.inst_results(call)[0];
        self.builder.ins().jump(merge_block, &[x]);
        self.builder.switch_to_block(merge_block);
        // We've now seen all the predecessors of the merge block.
        self.builder.seal_block(merge_block);
        let phi = self.builder.block_params(merge_block)[0];

        phi
    }
}

fn simplejit_to_number_slow(value: CVal) -> f64 {
    value.to_number_slow()
}
