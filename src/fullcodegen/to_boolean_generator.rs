use super::*;
use crate::assembler;
use crate::bytecode;
use crate::interpreter::callstack::*;
use crate::jit::*;
use crate::runtime;
use assembler::cpu::*;
use assembler::masm::*;
use bytecode::{def::*, virtual_reg::*, *};
use crate::heap::api::*;
use func::*;

pub struct ToBooleanGenerator {
    ins: Ins,
    val: VirtualRegister,
    end: Label,
    slow_path: Label,
}

impl ToBooleanGenerator {
    pub const fn new(x: VirtualRegister, i: Ins) -> Self {
        Self {
            ins: i,
            val: x,
            end: Label(0),
            slow_path: Label(0),
        }
    }
}

impl FullGenerator for ToBooleanGenerator {
    fn fast_path(&mut self, gen: &mut FullCodegen) -> bool {
        /*self.end = gen.masm.create_label();
        self.slow_path = gen.masm.create_label();
        gen.load_register_to(self.val, REG_RESULT2, None);
        let is_num = gen.masm.create_label();
        let is_bool = gen.masm.create_label();
        gen.masm.jmp_is_number(is_num, REG_RESULT2);
        gen.masm.jmp_is_boolean(is_bool, REG_RESULT2);
        gen.masm
            .jmp_nis_undefined_or_null(self.slow_path, REG_RESULT2);
        gen.masm.jump(self.end);
        gen.masm.load_int_const(MachineMode::Int8, REG_RESULT, 0);
        gen.masm.bind_label(is_num);
        let is_i32 = gen.masm.create_label();
        gen.masm.jmp_is_int32(is_i32, REG_RESULT2);
        gen.masm.as_double(REG_RESULT2, XMM0);
        gen.masm.float_cmp(
            MachineMode::Float64,
            REG_RESULT,
            XMM0,
            XMM1,
            CondCode::NotEqual,
        );
        gen.masm.jump(self.end);
        gen.masm.bind_label(is_i32);
        gen.masm.as_int32(REG_RESULT2, REG_RESULT);
        gen.masm.cmp_zero(MachineMode::Int32, REG_RESULT);
        gen.masm.set(REG_RESULT, CondCode::Equal);
        gen.masm.jump(self.end);
        gen.masm.bind_label(is_bool);
        gen.masm.is_true(REG_RESULT2, REG_RESULT);
        gen.masm.bind_label(self.end);
        true*/
        gen.load_register_to(self.val, CCALL_REG_PARAMS[0], None);
        gen.masm.emit_comment(format!("to_boolean {}", self.val));
        gen.masm.raw_call(__slow_path_to_boolean as *const u8);
        false
    }

    fn slow_path(&mut self, gen: &mut FullCodegen) {
        gen.masm.bind_label(self.slow_path);
        gen.masm
            .copy_reg(MachineMode::Int64, CCALL_REG_PARAMS[0], REG_RESULT2);
        gen.masm.raw_call(__slow_path_to_boolean as *const _);
        gen.masm.jump(self.end);
    }
}

extern "C" fn __slow_path_to_boolean(x: Value) -> bool {
    /*assert!(!x.is_int32());
    if x.is_cell() {
        match x.as_cell().value {
            CellValue::String(ref x) => x.len() != 0,
            CellValue::Array(ref x) => x.len() != 0,
            CellValue::ByteArray(ref x) => x.len() != 0,
            _ => true,
        }
    } else {
        false
    }*/
    x.to_boolean()
}
