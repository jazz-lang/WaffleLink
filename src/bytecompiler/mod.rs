pub mod graph_coloring;
pub mod interference_graph;
pub mod loopanalysis;
pub mod strength_reduction;

use crate::bytecode::*;
use crate::runtime::*;
use cgc::api::*;
use def::*;
use std::collections::HashMap;
use virtual_reg::*;
pub struct ByteCompiler {
    code: Rooted<CodeBlock>,
    vid: usize,
    args: usize,
    free_args: std::collections::VecDeque<VirtualRegister>,
    current: u32,
    vars: HashMap<String, VirtualRegister>,
}

impl ByteCompiler {
    pub fn new(rt: &mut Runtime) -> Self {
        let block = rt.allocate(CodeBlock {
            constants: Default::default(),
            constants_: vec![],
            arg_regs_count: 0,
            tmp_regs_count: 255,
            hotness: 0,
            code: Vec::new(),
            cfg: None,
            loopanalysis: None,
            jit_stub: None,
        });
        Self {
            code: block,
            vid: 0,
            args: 0,
            free_args: Default::default(),
            current: 0,
            vars: Default::default(),
        }
    }

    pub fn cjmp(&mut self, val: VirtualRegister) -> (impl FnMut(), impl FnMut()) {
        let p = self.current;
        let p2 = self.code.code[p as usize].code.len();
        self.code.code[p as usize].code.push(Ins::Jump { dst: 0 }); // this is replaced later.
        let this = unsafe { &mut *(self as *mut Self) };
        let this2 = unsafe { &mut *(self as *mut Self) };
        (
            move || {
                this.code.code[p as usize].code[p2 as usize] = Ins::JumpConditional {
                    cond: val,
                    if_true: this.current,
                    if_false: 0,
                };
            },
            move || {
                if let Ins::JumpConditional {
                    cond: _,
                    if_true: _,
                    if_false,
                } = &mut this2.code.code[p as usize].code[p2 as usize]
                {
                    *if_false = this2.current;
                } else {
                    this2.code.code[p as usize].code[p2 as usize] = Ins::JumpConditional {
                        cond: val,
                        if_true: 0,
                        if_false: this2.current,
                    };
                }
            },
        )
    }
    pub fn jmp(&mut self) -> impl FnMut() {
        let p = self.current;
        let p2 = self.code.code[p as usize].code.len();
        self.code.code[p as usize].code.push(Ins::Jump { dst: 0 }); // this is replaced later.
        let this = unsafe { &mut *(self as *mut Self) };
        move || this.code.code[p as usize].code[p2 as usize] = Ins::Jump { dst: this.current }
    }

    pub fn create_new_block(&mut self) -> u32 {
        let id = self.code.code.len() as u32;
        let bb = BasicBlock::new(id);
        self.code.code.push(bb);
        id
    }

    pub fn switch_to_block(&mut self, id: u32) {
        self.current = id;
    }
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
    pub fn def_var(&mut self, name: String, val: VirtualRegister) {
        self.vars.insert(name, val);
    }
    pub fn has_var(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    pub fn get_var(&self, name: &str) -> VirtualRegister {
        *self.vars.get(name).unwrap()
    }
    pub fn set_var(&mut self, name: String, new: VirtualRegister) {
        self.def_var(name, new);
    }
    pub fn finish(mut self) -> Rooted<CodeBlock> {
        self.code.arg_regs_count = self.args as _;
        self.code.tmp_regs_count = 255;

        strength_reduction::regalloc_and_reduce_strength(self.code.to_heap());
        self.code
    }
}
