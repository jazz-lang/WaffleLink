extern crate cgc_single_threaded as cgc;
extern crate waffle2;
use parser::*;
use reader::*;
use value::*;
use waffle2::bytecompiler::*;
use waffle2::frontend::*;
use waffle2::runtime::*;
fn main() {
    //simple_logger::init().unwrap();
    let mut heap = {
        let mut rt = Runtime::new();
        let reader = Reader::from_string(
            "
function fib(n) {
    var prev = 0
    var next = 1
    var i = 0
    while i < n {
        var temp = next
        next = prev + next
        prev = temp
        i = i + 1
    }
  
    return prev
}
fib(100)
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
        let f = function_from_codeblock(&mut rt, code.to_heap(), "<main>");
        match rt.call(f, Value::undefined(), &[]) {
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
