extern crate cgc_single_threaded as cgc;
extern crate waffle2;
use cgc::api::*;
use parser::*;
use reader::*;
use value::*;
use waffle2::bytecompiler::*;
use waffle2::frontend::*;
use waffle2::fullcodegen::FullCodegen;
use waffle2::interpreter::callstack::CallFrame;
use waffle2::jit::JITResult;
use waffle2::runtime::*;
fn main() {
    //simple_logger::init().unwrap();
    let mut heap = {
        let mut rt = Runtime::new();
        let reader = Reader::from_string(
            "
var a = 3
var b = 4
return a + b
",
        );
        let mut ast = vec![];
        let mut p = Parser::new(reader, &mut ast);
        if let Err(e) = p.parse() {
            eprintln!("{}", e);
            return;
        }
        let code = match compile(&mut rt, &ast) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };
        let mut gen = FullCodegen::new(code.to_heap());
        gen.compile();
        let ncode = gen.finish(&mut rt, true);
        let func: extern "C" fn(&mut Runtime, Handle<CallFrame>) -> JITResult =
            unsafe { std::mem::transmute(ncode.instruction_start()) };
        let x = unsafe { &mut *(&mut rt as *mut Runtime) };
        let _ = rt
            .stack
            .push(x, Value::undefined(), code.to_heap(), Value::undefined());
        let current = rt.stack.current_frame();
        match func(&mut rt, current) {
            JITResult::Ok(val) => {
                println!("{}", waffle2::unwrap!(val.to_string(&mut rt)));
            }
            _ => unreachable!(),
        }
        /*let f = function_from_codeblock(&mut rt, code.to_heap(), "<main>");
        match rt.call(f, Value::undefined(), &[]) {
            Ok(x) => match x.to_string(&mut rt) {
                Ok(x) => println!("Result: {}", x),
                _ => unreachable!(),
            },
            Err(e) => println!("Err {}", waffle2::unwrap!(e.to_string(&mut rt))),
        }
        rt.perf.print_perf();*/
        rt.heap
    };

    heap.collect();
}
