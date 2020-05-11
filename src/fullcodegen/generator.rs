use crate::bytecode::*;
use crate::*;
use assembler::*;
use cgc::api::Handle;
use jit::*;
use masm::*;
use runtime::*;
use types::*;
pub trait FullGenerator {
    fn generate(&self, _masm: &mut MacroAssembler, _code: Handle<CodeBlock>);
}
