use super::*;
use crate::bytecode::*;
use crate::*;
use assembler::*;
use crate::heap::api::Handle;
use jit::*;
use masm::*;
use runtime::*;
use types::*;
pub trait FullGenerator {
    fn fast_path(&mut self, gen: &mut FullCodegen) -> bool;
    fn slow_path(&mut self, gen: &mut FullCodegen);
}
