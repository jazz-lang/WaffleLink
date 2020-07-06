use capstone::prelude::*;
use linkbuffer::*;
use masm::*;
use x86_assembler::*;
use x86masm::*;
pub extern "C" fn foo(x: *const u8, y: i32) -> i32 {
    println!("{:p} {}", x, y);
    42
}

fn main() {
    let mut masm = MacroAssemblerX86::new(true);
    masm.function_prologue(0);
    let s = "Hello,World! %i\n\0";
    masm.prepare_call_with_arg_count(2);
    masm.pass_ptr_as_arg(s.as_ptr() as usize, 0);
    masm.pass_int32_as_arg(42, 1);
    let c = masm.call(2);
    masm.function_epilogue();
    masm.ret();
    let code = masm.finalize();
    let mut memory = Memory::new();
    let ptr = memory.allocate(code.len(), 8).unwrap();
    unsafe {
        std::ptr::copy_nonoverlapping(code.as_ptr(), ptr, code.len());
        let buffer = LinkBuffer::<MacroAssemblerX86>::new(ptr);
        buffer.link_call(c, foo as *const u8);
        memory.set_readable_and_executable();
        let f: fn() -> i32 = std::mem::transmute(ptr);
        println!("{}", f());
        let code = std::slice::from_raw_parts(ptr, code.len());
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("Failed to create Capstone object");
        let insns = cs.disasm_all(code, ptr as _);
        for i in insns.unwrap().iter() {
            println!("{}", i);
        }
    }
}
