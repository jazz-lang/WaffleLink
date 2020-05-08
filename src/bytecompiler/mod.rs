pub mod graph_coloring;
pub mod interference_graph;
pub mod loopanalysis;
pub mod strength_reduction;

use crate::bytecode::*;
use cgc::api::*;
use def::*;
use virtual_reg::*;
pub struct ByteCompiler {
    code: Rooted<CodeBlock>,
    vid: usize,
    args: usize,
    free_args: std::collections::VecDeque<VirtualRegister>,
    current: u32,
}

impl ByteCompiler {
    pub fn emit(&mut self, ins: Ins) {
        self.code.code[self.current as usize].code.push(ins);
    }
    pub fn mov(&mut self, to: VirtualRegister, from: VirtualRegister) {
        self.emit(Ins::Mov { dst: to, src: from });
    }

    pub fn vreg(&mut self) -> VirtualRegister {
        let x = self.vid;
        self.vid += 1;
        VirtualRegister::tmp(x as _)
    }
    pub fn areg(&mut self) -> VirtualRegister {
        let x = self.args;
        self.args += 1;
        VirtualRegister::argument(x as _)
    }

    pub fn do_call(
        &mut self,
        func: VirtualRegister,
        this: VirtualRegister,
        arguments: &[VirtualRegister],
    ) -> VirtualRegister {
        let dst = self.vreg();
        let mut used = std::collections::VecDeque::new();
        for arg in arguments.iter() {
            if let Some(reg) = self.free_args.pop_front() {
                self.emit(Ins::Mov {
                    dst: reg,
                    src: *arg,
                });
                used.push_back(reg);
            } else {
                let reg = self.areg();
                used.push_back(reg);
                self.emit(Ins::Mov {
                    dst: reg,
                    src: *arg,
                });
            }
        }

        self.emit(Ins::Call {
            dst,
            function: func,
            this,
            begin: *used.front().unwrap(),
            end: *used.back().unwrap(),
        });
        while let Some(x) = used.pop_front() {
            self.free_args.push_front(x);
        }

        dst
    }

    pub fn close_env(
        &mut self,
        func: VirtualRegister,
        arguments: &[VirtualRegister],
    ) -> VirtualRegister {
        let dst = self.vreg();
        let mut used = std::collections::VecDeque::new();
        for arg in arguments.iter() {
            if let Some(reg) = self.free_args.pop_front() {
                self.emit(Ins::Mov {
                    dst: reg,
                    src: *arg,
                });
                used.push_back(reg);
            } else {
                let reg = self.areg();
                used.push_back(reg);
                self.emit(Ins::Mov {
                    dst: reg,
                    src: *arg,
                });
            }
        }

        self.emit(Ins::CloseEnv {
            dst,
            function: func,
            begin: *used.front().unwrap(),
            end: *used.back().unwrap(),
        });
        while let Some(x) = used.pop_front() {
            self.free_args.push_front(x);
        }

        dst
    }
}
