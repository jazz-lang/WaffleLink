extern crate waffle2;
use parser::*;
use reader::*;
use value::*;
use waffle2::bytecompiler::*;
use waffle2::frontend::*;
use waffle2::fullcodegen::FullCodegen;
use waffle2::heap::api::*;
use waffle2::interpreter::callstack::CallFrame;
use waffle2::jit::JITResult;
use waffle2::runtime::*;
fn main() {
    let mut heap = {
        let mut rt = Runtime::new(Configs::default());
        let reader = Reader::from_string(
            "
function fac(x) {
    if x < 2 {
        return 1
    } else {
        fac(x - 1) * x
    }
}
log(fac(5))
return 0
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
        let f = function_from_codeblock(&mut rt, code.clone(), "<main>");
        let x = std::time::Instant::now();
        //let mut xx = &mut f;

        let res = rt.call(f, Value::undefined(), &[]);
        //let yy = *xx;
        let e = x.elapsed();
        let ms = e.as_millis();
        let ns = e.as_nanos();
        println!("Executed in {} ms or {} ns", ms, ns);
        match res {
            Ok(x) => {
                match x.to_string(&mut rt) {
                    Ok(x) => println!("Result: {}", x),
                    _ => unreachable!(),
                };
            }
            Err(e) => println!("Err {}", waffle2::unwrap!(e.to_string(&mut rt))),
        }
        #[cfg(feautre = "perf")]
        {
            rt.perf.print_perf();
        }
        rt.heap
    };

    //heap.collect();
}
