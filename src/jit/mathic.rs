use super::*;
#[derive(Default)]
pub struct MathICGenerationState {
    pub fast_path_start: Label,
    pub fast_path_end: Label,
    pub slow_path_start: Label,
    pub slow_path_call: masm::Call,
    pub slow_path_jumps: JumpList,
    pub should_slow_path_repatch: bool,
}

pub struct MathIC<T: MathICGenerator> {
    pub generator: Option<T>,
    code: *mut u8,
    inline_start: *mut u8,
    inline_end: *mut u8,
    slow_path_call_loc: *mut u8,
    slow_path_start_loc: *mut u8,
    generate_fastpath_on_repatch: bool,
    arith_profile: Option<()>,
}

impl<T: MathICGenerator> MathIC<T> {
    pub fn new() -> Self {
        Self {
            generate_fastpath_on_repatch: false,
            generator: None,
            inline_end: 0 as *mut _,
            inline_start: 0 as *mut _,
            slow_path_call_loc: 0 as *mut _,
            slow_path_start_loc: 0 as *mut _,
            arith_profile: None,
            code: 0 as *mut _,
        }
    }
    pub fn generate_inline(
        &mut self,
        jit: &mut JIT<'_>,
        state: &mut MathICGenerationState,
        should_profile: bool,
    ) -> bool {
        state.fast_path_start = jit.masm.label();
        let start_size = jit.masm.asm.data().len();
        // TODO: Collect type info
        if let Some(ref _arith_profile) = self.arith_profile.as_ref() {
            todo!();
        }
        let result = self.generator.as_mut().unwrap().generate_inline(jit, state);
        match result {
            MathICResult::GenFastPath => {
                let inline_size = jit.masm.asm.data().len() - start_size;
                if inline_size < jit.patchable_jump_size() {
                    let nops_to_emit = jit.patchable_jump_size() - inline_size;
                    for _ in 0..nops_to_emit {
                        jit.masm.asm.nop();
                    }
                }
                state.should_slow_path_repatch = true;
                state.fast_path_end = jit.masm.label();
                return true;
            }
            MathICResult::GenFullSnippet => {
                let mut end_jump_list = JumpList::new();
                let result = self.generator.as_mut().unwrap().generate_fastpath(
                    jit,
                    &mut end_jump_list,
                    &mut state.slow_path_jumps,
                    should_profile,
                );
                if result {
                    state.fast_path_end = jit.masm.label();
                    state.should_slow_path_repatch = false;
                    end_jump_list.link(&mut jit.masm);
                    return true;
                }
                return false;
            }
            _ => return false,
        }
    }

    pub fn generate_out_of_line(&mut self, code_block: &CodeBlock, call_replacement: *const u8) {
        let link_jump_out_of_line_snippet = |this: &mut Self| {
            let mut jit = JIT::new(code_block);
            let jump = jit.masm.jump();
            let link_buffer = JITLinkBuffer::new(this.inline_start.clone());
            link_buffer.link_jump_ptr(jump.label().asm_label(), this.code);
        };
        let replace_call = |this: &mut Self| unsafe {
            #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
            {
                X86Asm::repatch_pointer(
                    this.slow_path_call_loc
                        .offset(-(REPATCH_OFFSET_CALL_R11 as isize)),
                    call_replacement as *mut u8,
                )
            }
        };

        if self.generate_fastpath_on_repatch {
            let mut jit = JIT::new(code_block);
            let mut state = MathICGenerationState::default();
            let generated_inline = self.generate_inline(&mut jit, &mut state, false);
            self.generate_fastpath_on_repatch = false;
            if generated_inline {
                let jump_to_done = jit.masm.label();
                let buffer = JITLinkBuffer::from_masm(&mut jit.masm);
                if !buffer.did_fail_to_allocate() {
                    for j in state.slow_path_jumps.jumps.iter() {
                        buffer.link_jump_ptr(j.label().asm_label(), self.slow_path_start_loc);

                        self.code = buffer.code;
                        if !state.should_slow_path_repatch {
                            replace_call(self);
                        }
                        link_jump_out_of_line_snippet(self);
                        return;
                    }
                    buffer.link_jump_ptr(jump_to_done.asm_label(), self.inline_end);
                }
            }
        }
        replace_call(self);
        let mut jit = JIT::new(code_block);
        let mut end_jump_list = JumpList::new();
        let mut slow_path_jump_list = JumpList::new();
        let emitted_fast_path = self.generator.as_mut().unwrap().generate_fastpath(
            &mut jit,
            &mut end_jump_list,
            &mut slow_path_jump_list,
            false,
        );
        if !emitted_fast_path {
            return;
        }
        let buffer = JITLinkBuffer::from_masm(&mut jit.masm);
        if buffer.did_fail_to_allocate() {
            return;
        }

        for j in end_jump_list.jumps.iter() {
            buffer.link_jump_ptr(j.label().asm_label(), self.inline_end);
        }
        for j in slow_path_jump_list.jumps.iter() {
            buffer.link_jump_ptr(j.label().asm_label(), self.slow_path_start_loc);
        }
        link_jump_out_of_line_snippet(self);
    }
}

pub trait MathICGenerator {
    fn generate_inline(
        &mut self,
        jit: &mut JIT<'_>,
        state: &mut MathICGenerationState,
    ) -> MathICResult;
    fn generate_fastpath(
        &mut self,
        jit: &mut JIT<'_>,
        end_jump_list: &mut JumpList,
        slow_path_jump_list: &mut JumpList,
        should_profile: bool,
    ) -> bool;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum MathICResult {
    GenFastPath,
    GenFullSnippet,
    DontGenerate,
}
pub trait BinaryMathICGenerator {
    fn new(
        result_r: Reg,
        left_reg: Reg,
        right_reg: Reg,
        left_fp: FPReg,
        right_fp: FPReg,
        scratch: Reg,
        scratch_fp: FPReg,
    ) -> Self;
}
