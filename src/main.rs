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
    simple_logger::init().unwrap();
    let mut heap = {
        let mut rt = Runtime::new();
        let reader = Reader::from_string(
            "
function foo() {
    return 1
}

var i = 0
while i < 100000 {
    foo()
    i = i + 1
}

return i
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
        /*let mut gen = FullCodegen::new(code.to_heap());
        gen.compile(false);
        let ncode = gen.finish(&mut rt, true);
        let func: extern "C" fn(&mut Runtime, Handle<CallFrame>,u32) -> JITResult =
            unsafe { std::mem::transmute(ncode.instruction_start()) };
        let x = unsafe { &mut *(&mut rt as *mut Runtime) };
        let _ = rt
            .stack
            .push(x, Value::undefined(), code.to_heap(), Value::undefined());
        rt.stack.current_frame().entries.push(Value::undefined());
        let current = rt.stack.current_frame();
        let s = std::time::Instant::now();
        let res = func(&mut rt, current);
        let e = s.elapsed();
        let ms = e.as_millis();
        let ns = e.as_nanos();
        println!("JIT code executed in: {}ms or {}ns", ms, ns);
        match res {
            JITResult::Ok(val) => {
                println!("{}", waffle2::unwrap!(val.to_string(&mut rt)));
            }
            _ => unreachable!(),
        }

        */let f = function_from_codeblock(&mut rt, code.to_heap(), "<main>");
        let res = rt.call(f, Value::undefined(), &[]);
        match res {
            Ok(x) => match x.to_string(&mut rt) {
                Ok(x) => println!("Result: {}", x),
                _ => unreachable!(),
            },
            Err(e) => println!("Err {}", waffle2::unwrap!(e.to_string(&mut rt))),
        }
        rt.perf.print_perf();
        rt.heap
    };

    heap.collect();
}
