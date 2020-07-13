use super::*;
use crate::*;
fn slow_path_for(jit: &mut JIT<'_>, vm: &crate::VM, slow_path_func: *const u8) {
    jit.emit_function_prologue();
    jit.masm
        .store64(BP, Mem::Absolute(&vm.top_call_frame as *const _ as _));
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        // windows calling convention is weird: we have to return SlowPathReturn on stack instead of using rax and rdx
        // so we simulate using these two regs by allocating space on stack and then loading return value from stack to
        // these 2 registers.
        jit.masm
            .add64_imm32(-(MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL as i32), SP, SP);
        jit.masm.move_rr(T2, AGPR0);
        jit.masm.move_rr(T3, AGPR2);
        jit.masm.move_rr(AGPR0, AGPR3);
        jit.masm.add64_imm32(32, SP, AGPR0);
        jit.masm.move_rr(BP, AGPR1);
        jit.masm.move_i64(slow_path_func as _, Reg::R10);
        jit.masm.call_r(Reg::R10);
        jit.masm.load64(Mem::Base(RET0, 8), RET1);
        jit.masm.load64(Mem::Base(RET0, 0), RET0);
        jit.masm
            .add64_imm32(MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL as i32, SP, SP);
    }
    #[cfg(not(windows))]
    {
        if MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL != 0 {
            jit.masm
                .add64_imm32(-(MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL as i32), SP, SP);
        }
        #[cfg(target_arch = "x86_64")]
        const NON_ARG_GP0: Reg = Reg::R10;

        jit.masm.pass_reg_as_arg(T3, 0);
        jit.masm.pass_reg_as_arg(T2, 1);
        jit.masm.move_i64(slow_path_func as _, NON_ARG_GP0);
        jit.masm.call_r(NON_ARG_GP0);
        if MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL != 0 {
            jit.masm
                .add64_imm32(MAX_FRAME_EXTENT_FOR_SLOW_PATH_CALL as i32, SP, SP);
        }
    }
    jit.masm.function_epilogue();
    let do_not_trash = jit
        .masm
        .branch64_test_imm64(ResultCondition::Zero, RET1, -1);
    jit.masm.pop(Reg::R10);
    jit.prepare_for_tail_call_slow(RET0);
    do_not_trash.link(&mut jit.masm);
    jit.masm.far_jump_r(RET0);
}

pub fn link_call_thunk_generator(vm: &VM) -> *const u8 {
    let cb = CodeBlock::new();
    let mut jit = JIT::new(&cb);
    slow_path_for(&mut jit, vm, operations::operation_link_call as *const u8);
    let mut patch_buf = JITLinkBuffer::from_masm(&mut jit.masm);
    patch_buf.perform_finalization();
    patch_buf.code
}

#[repr(C)]
pub struct SlowPathReturn {
    pub a: usize,
    pub b: usize,
}

impl SlowPathReturn {
    pub const fn encode(x: usize, y: usize) -> Self {
        Self { a: x, b: y }
    }
}
