use super::*;

impl<'a> JIT<'a> {
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
        self.emit_get_virtual_register(callee, T0);
        let mut label = DataLabelPtr::default();
        let slow_case =
            self.branch_ptr_with_patch(RelationalCondition::NotEqual, T0, &mut label, 0);
        self.add_slow_case(slow_case);
        let call = self.emit_naked_call(std::ptr::null());
        self.call_compilation_info
            .push(CallCompilationInfo::default());
        self.call_compilation_info[call_link_info_idx].hot_path_begin = label;
        self.call_compilation_info[call_link_info_idx].hot_path_other = call;
    }

    pub fn compile_op_call_slowcase(
        &mut self,
        ins: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
        call_link_info_idx: u32,
    ) {
        self.link_all_slow_cases(slow_cases);
        self.masm.move_i64(crate::get_vm() as *const _ as i64, T3);
        //let x = self.call_compilation_info[call_link_info_idx as usize].call
        self.call_compilation_info[call_link_info_idx as usize].call_return_location =
            self.emit_naked_call(0 as *mut _);
        match ins {
            Ins::Call(_, dest, ..) => {
                self.emit_put_virtual_register(*dest, RET0, T5);
            }
            _ => (),
        }
    }
}
