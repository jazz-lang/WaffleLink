extern crate cgc_single_threaded as cgc;
extern crate waffle2;
use parser::*;
use reader::*;
use value::*;
use waffle2::bytecompiler::*;
use waffle2::frontend::*;
use waffle2::runtime::*;
fn main() {
    let mut rt = Runtime::new();
    //simple_logger::init().unwrap();
    {
        let reader = Reader::from_string(
            "
function foo(x) {
    if x {
        return 1
    } else {
        return 0
    }
}
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
                Ok(x) => println!("{}", x),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    rt.heap.collect();
}
