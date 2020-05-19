use crate::bytecode;
use bytecode::def::*;
use bytecode::virtual_reg::*;
use regalloc::{Reg, RegClass, RegUsageCollector, Writable};
macro_rules! r {
    ($x: expr) => {
        if $x.to_local() < 255 {
            Reg::new_real(RegClass::I64, 0, $x.to_local() as _)
        } else {
            Reg::new_virtual(RegClass::I64, $x.to_local() as _)
        }
    };
}
fn get_usage(ins: Ins, collector: &mut RegUsageCollector) {
    let uses = ins.get_uses();
    for u in uses {
        collector.add_def(Writable::from_reg(r!(u)));
    }

    let defs = ins.get_defs();
    for def in defs {
        collector.add_use(r!(def))
    }
}
