//! SimpleJIT implementation.
//!
//! The first execution of any function always starts in the interpreter tier. As soon as any statement
//! in the function executes more than 100 times, or the function is called more than 10 times (whichever comes first),
//! execution is diverted into code compiled by the Simple JIT
//!
//!
//! What this Simple JIT does is removes interpreter loop dispatch overhead and compiles bytecode into single stream of
//! machine instructions without interpreter dispatch, this improves performance by helping CPU predicting branches.

use crate::bytecode::op::*;
use crate::dynasmrt::DynasmApi;
use crate::dynasmrt::DynasmLabelApi;
use crate::runtime;
use runtime::frame::Frame;
use runtime::function::*;
use runtime::value::*;
pub struct SimpleJIT<'a> {
    func: &'a mut Function,
    ops: dynasmrt::x64::Assembler,
}
impl<'a> SimpleJIT<'a> {
    pub fn number(&mut self) {
        dynasm!(self.ops
            ; cvttsd2si ecx, xmm0
            ; cvtsi2sd xmm1, ecx
            ; mov rax, 562949953421312
            ; mov rdx, rax
            ; movq rax, xmm0
            ; add rdx, rax
            ; movabs rax,-562949953421312
            ; or rax, rcx
            ; ucomisd xmm1,xmm0
            ; cmovne rax,rdx
            ; cmovp rax,rdx
            ; test ecx,ecx
            ; cmovne rax,rdx
            ; xorps xmm1,xmm1
            ; ucomisd xmm1, xmm0
            ; cmova rax,rdx
        );
    }

    pub fn load_regs_to_rax(&mut self) {
        dynasm!(self.ops
           ; mov rax, QWORD [r15 + offset_of!(Frame, regs) as i32]
        );
    }

    pub fn load_register(&mut self, r: u8) {
        self.load_regs_to_rax();
        dynasm!(
            self.ops
            ; mov rax, QWORD [rax + r as i32]
        )
    }
    pub fn load_acc(&mut self) {
        dynasm!(
            self.ops
            ; mov rax, QWORD [r15 + offset_of!(Frame,rax) as i32]
        )
    }
    pub fn store_acc(&mut self) {
        dynasm!(
            self.ops
            ; mov QWORD [r15 + offset_of!(Frame,rax) as i32], rax
        );
    }
    pub fn store_r(&mut self, r: u8) {
        dynasm!(
            self.ops
            ; mov rdx,QWORD [r15 + offset_of!(Frame, regs) as i32]
            ; mov QWORD [rdx + r as i32], rax
        );
    }
    pub fn lda_int(&mut self, x: i32) {
        dynasm!(
            self.ops
            ; mov ecx,x
            ; movabs rax, -562949953421312
            ; or rax, rcx
        );
        self.store_acc();
    }

    pub fn value_from_vtag(&mut self, tag: i32) {
        dynasm!(
            self.ops
            ; mov dil, tag as i8
            ; movzx eax, dil
        );
    }

    pub fn compile(&mut self, code: OpV) {
        use OpV::*;
        match code {
            Star(r) => {
                self.load_acc();
                self.store_r(r);
            }
            Ldar(r) => {
                self.load_register(r);
                self.store_acc();
            }
            LdaArguments => {
                dynasm!(
                    self.ops
                    ; mov rax, QWORD [r15 + offset_of!(Frame,arguments) as i32]
                );
                self.store_acc();
            }
            Mov(r0, r1) => {
                self.load_register(r1);
                self.store_r(r0);
            }
            Add(rhs, fdbk) => {
                self.load_regs_to_rax();
                dynasm!(
                    self.ops
                    ; mov rcx, QWORD [rax + rhs as i32]
                );
                self.load_acc();
                dynasm!(
                    self.ops
                    // acc.is_int32()
                    ; shr rax, 49
                    ; cmp rax, 32766
                    ; ja >not_i32_acc
                    // rhs.is_int32()
                    ; shr rcx,49
                    ; cmp rcx, 32766
                    ; ja >not_i32_rhs
                    // acc + rhs
                    ; add eax,ecx
                    ; mov ecx, eax
                    // Value::new_int
                    ; movabs rax,-562949953421312
                    ; or rax, rcx
                    // frame.rax = acc + rhs
                    ; mov QWORD [r15 + offset_of!(Frame,rax) as i32], rax
                    ; not_i32_acc:
                    ; mov rdi,rax
                    ; mov rax,rdi
                    ; shr rax, 49
                    ; je >acc_nan
                    ; shl rax, 49
                    ; movabs rax, -562949953421312
                    ; add rax,rdi
                    ; movq xmm0, rax
                    ; acc_nan:
                    ; movsd xmm0, [->nan]
                    ; jmp >do_fadd
                    ; not_i32_rhs:
                    ; mov rdi, rcx
                    ; mov rax, rcx
                    ; shr rax, 49
                    ; je >rhs_nan
                    ; shl rax, 49
                    ; movabs rax, -562949953421312
                    ; add rax,rdi
                    ; movq xmm0, rax
                    ; rhs_nan:
                    ; movsd xmm0, [->nan]
                    ; jmp >do_fadd
                );
                dynasm!(self.ops
                    ; .arch x64
                    ; -> nan:
                    ; .bytes unsafe {std::mem::transmute::<u64,[u8;8]>(9221120237041090560).iter()}
                );
            }
            _ => unimplemented!(),
        }
    }
}
