/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

extern crate waffle;
use process::*;
use reader::*;
use std::path::PathBuf;
use waffle::bytecode::*;
use waffle::runtime::config::{Config, CONFIG};
use waffle::runtime::*;

#[allow(unused_macros)]
macro_rules! waffle_asm {
    (
        $(
            c $value: expr;
        )*
        code_start:
        $(
            func $func_name: ident : $argc: expr => {
                $(
                    $block_index: expr => {
                        $($rest: tt)*
                    }
                )*
            }
        )*
    ) => {{
        let mut module = Arc::new(Module::new("Main"));
        $(
            module.globals.push(Value::from(RUNTIME.state.intern_string($value.to_owned())));

        )*
        let mut fn_map = std::collections::HashMap::new();
        $(
            let mut blocks = vec![];
            let mut i = 0;
            $(
                i += 1;
                let mut code = vec![];
                waffle_asm!(@ins code => $($rest)*);
                let bb = waffle::bytecode::basicblock::BasicBlock {
                    instructions: code,
                    index: i
                };
                blocks.push(bb);
            )*
            let func = Function {
                upvalues: vec![],
                name: Value::from(RUNTIME.state.intern_string(stringify!($func_name).to_owned())),
                module: module.clone(),
                code: Arc::new(blocks),
                native: None,
                argc: $argc,
                md: Default::default(),
            };

            let value = RUNTIME.state.allocate_fn(func);
            fn_map.insert(stringify!($func_name),value);
            module.globals.push(value);
        )*

        for (i,global) in module.globals.iter().enumerate() {
            println!("Global {}: {}",i,global.to_string());
        }
        (module,fn_map)
    }};

    (@ins $bcode: expr => load_int $r0: expr, $i: expr; $($rest: tt)*) => {
        $bcode.push(Instruction::LoadInt($r0 as u16,$i as i32));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => add $r0: expr,$r1: expr,$r2: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::Binary(BinOp::Add,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => sub $r0: expr,$r1: expr,$r2: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::Binary(BinOp::Sub,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode =>  $($rest)*);
    };
    (@ins $bcode: expr => mul $r0: expr,$r1: expr,$r2: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::Binary(BinOp::Mul,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode =>  $($rest)*);
    };
    (@ins $bcode: expr => div $r0: expr,$r1: expr,$r2: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::Binary(BinOp::Div,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode =>  $($rest)*);
    };
    (@ins $bcode: expr => cmp $cmp_op: ident $r0: expr,$r1: expr,$r2: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::Binary(BinOp::$cmp_op,$r0,$r1,$r2));
        waffle_asm!(@ins $bcode =>  $($rest)*);
    };
    (@ins $bcode: expr => call $r0: expr,$r1: expr,$r2: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::Call($r0,$r1,$r2));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => tail_call $r0: expr,$r1: expr,$r2: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::TailCall($r0,$r1,$r2));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => virtcall $r0: expr,$r1: expr,$r2: expr,$r3: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::VirtCall($r0,$r1,$r2,$r3));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => new $r0: expr,$r1: expr,$r2: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::New($r0,$r1,$r2));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => load_by_id $r0: expr,$r1: expr,$id: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::LoadById($r0,$r1,$id));
        waffle_asm!(@ins $bcode =>  $($rest)*);
    };
    (@ins $bcode: expr => load_static_by_id $r0: expr,$id: expr;$($rest:tt)*) => {
        $bcode.push(Instruction::LoadStaticById($r0,$id));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr =>  move $r0: expr,$r1: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::Move($r0,$r1));
        waffle_asm!(@ins $bcode =>  $($rest)*);
    };
    (@ins $bcode: expr => retv $r0: expr;$($rest: tt)*) => {
        $bcode.push(Instruction::Return(Some($r0)));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => load_const $r0: expr, $id: expr;$($rest:tt)*) => {
        $bcode.push(Instruction::LoadConst($r0,$id));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => push $r0: expr;$($rest:tt)*) => {
        $bcode.push(Instruction::Push($r0));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => pop $r0: expr;$($rest:tt)*) => {
        $bcode.push(Instruction::Pop($r0));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => conditional_branch $r0: expr,$x: expr,$y: expr;$($rest:tt)*) => {
        $bcode.push(Instruction::ConditionalBranch($r0,$x,$y));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr => branch $t: expr;$($rest:tt)*) => {
        $bcode.push(Instruction::Branch($t));
        waffle_asm!(@ins $bcode => $($rest)*);
    };
    (@ins $bcode: expr =>) => {

    }
}

fn main() {
    if CONFIG.read().verbose {
        simple_logger::init().unwrap();
    }
    CONFIG.write().directories.push(PathBuf::from(format!(
        "{}/.waffle/std/",
        dirs::home_dir().unwrap().display()
    )));
    let c = Config::default();

    let contents = std::fs::read(&c.main_name).expect("ERROR!");
    let mut reader = BytecodeReader {
        bytes: std::io::Cursor::new(&contents),
    };

    let module = reader.read_module();
    let proc = Process::from_function(module.main_fn, &c).unwrap();
    RUNTIME.schedule_main_process(proc.clone());
    RUNTIME.start_pools();
}
