use super::*;
use crate::interpreter::callframe::*;
use crate::value::*;
impl<'a> JIT<'a> {
    pub fn compile_setup_frame(&mut self, ins: &Ins) {
        let (argcount_including_this, register_offset) = match ins {
            Ins::Call(_, _, argc, argv) => (*argc as i32, -(*argv as i32)),
            _ => todo!(),
        };

        self.masm.add64_imm32(
            register_offset * 8 + std::mem::size_of::<CallerFrameAndPc>() as i32,
            BP,
            SP,
        );
        self.masm.store32_imm(
            argcount_including_this,
            Mem::Base(
                SP,
                CallFrameSlot::ArgumentCountIncludingThis as i32 * 8
                    + offset_of!(AsBits, payload) as i32
                    - std::mem::size_of::<CallerFrameAndPc>() as i32,
            ),
        );
    }

    pub fn compile_op_call(&mut self, ins: &Ins, call_link_info_idx: usize) {
        let callee = match ins {
            Ins::Call(callee, ..) => *callee,
            _ => unimplemented!(),
        };
        /* Caller always:
            - Updates BP to callee callFrame.
            - Initializes ArgumentCount; CallerFrame; Callee.
           For a Waffle call:
            - Callee initializes ReturnPC; CodeBlock.
            - Callee restores BP before return.
           For a non-Waffle call:
            - Caller initializes ReturnPC; CodeBlock.
            - Caller restores BP after return.
        */
        self.compile_setup_frame(ins);

        self.emit_get_virtual_register(callee, T0);
        self.masm.store64(
            T0,
            Mem::Base(
                SP,
                CallFrameSlot::Callee as i32 * 8 - std::mem::size_of::<CallerFrameAndPc>() as i32,
            ),
        );
        let mut label = DataLabelPtr::default();
        let slow_case =
            self.branch_ptr_with_patch(RelationalCondition::NotEqual, T0, &mut label, 0);
        self.add_slow_case(slow_case);
        let call = self.emit_naked_call(std::ptr::null());
        self.call_compilation_info
            .push(CallCompilationInfo::default());
        self.call_compilation_info[call_link_info_idx].hot_path_begin = label;
        self.call_compilation_info[call_link_info_idx].hot_path_other = call;
        self.masm.add64_imm32(
            virtual_register::virtual_register_for_local(
                jit_frame_register_count_for(self.code_block) as i32 - 1,
            )
            .offset()
                * 8,
            BP,
            SP,
        );
    }
}
