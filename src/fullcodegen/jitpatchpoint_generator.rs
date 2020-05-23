use super::*;
use crate::assembler;
use crate::bytecode;
use crate::heap::api::*;
use crate::interpreter::callstack::*;
use crate::jit::*;
use crate::runtime;
use assembler::cpu::*;
use assembler::masm::*;
use bytecode::{def::*, virtual_reg::*, *};
use func::*;
pub struct JITPatchPointGenerator {
    pub slow_path: Label,
    pub end: Label,
    pub ins: Ins,
    pub cb: extern "C" fn(usize, &mut CallFrame) -> Value,
    pub size: usize,
}

extern "C" fn patchpoint(ra: usize, x: &mut CallFrame) -> Value {
    Value::null()
}

const MIN_PATCHPOINT_SIZE: usize = 40;

impl FullGenerator for JITPatchPointGenerator {
    fn fast_path(&mut self, gen: &mut FullCodegen) -> bool {
        
        gen.masm
            .emit_lazy_compilation_site(LazyCompilationSite::PatchPoint {
                size_to_nop:self.size,
                size: MIN_PATCHPOINT_SIZE + self.size
            });
        unimplemented!()
        false
    }
    fn slow_path(&mut self, gen: &mut FullCodegen) {}
}
