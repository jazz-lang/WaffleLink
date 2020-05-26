extern crate waffle2;
use parser::*;
use reader::*;
use waffle2::bytecode::def::*;
use waffle2::frontend::*;
fn main() {
    simple_logger::init().unwrap();
    let _heap = {
        let reader = Reader::from_string(
            "
function foo(x) {
    var y = x.x
    var z = x.x
    return y+z
} 
return 0
",
        );
        let mut ast = vec![];
        let mut p = Parser::new(reader, &mut ast);
        if let Err(e) = p.parse() {
            eprintln!("{}", e);
            return;
        }
        println!("{}", std::mem::size_of::<Ins>());
    };

    //heap.collect();
}
