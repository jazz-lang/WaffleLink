extern crate cgc_single_threaded as cgc;
extern crate waffle2;
use cgc::api::*;
use cgc::heap::*;
use parser::*;
use reader::*;
use value::*;
use waffle2::bytecompiler::*;
use waffle2::frontend::*;
use waffle2::runtime::*;
fn main() {
    let reader = Reader::from_string(
        "

var a = 3
var b = 4
a = a + b
",
    );
    let mut ast = vec![];
    let mut p = Parser::new(reader, &mut ast);
    if let Err(e) = p.parse() {
        eprintln!("{}", e);
        return;
    }
    let mut rt = Runtime::new();
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
