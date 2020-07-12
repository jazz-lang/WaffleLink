use super::*;
fn slow_path_for(jit: &mut JIT<'_>, vm: &crate::VM, slow_path_func: *const u8) {
    jit.emit_function_prologue();
    jit.masm
        .store64(BP, Mem::Absolute(&vm.top_call_frame as *const _ as _));
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
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
    let do_not_trash = jit
        .masm
        .branch64_test_imm64(ResultCondition::Zero, RET1, -1);
    jit.masm.function_epilogue();
    jit.masm.pop(Reg::R10);
    do_not_trash.link(&mut jit.masm);
}
