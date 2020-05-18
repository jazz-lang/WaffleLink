use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

fn jit_loop(c: &mut Criterion) {
    let mut rt = Runtime::new(Configs::default().no_jit());
    let reader = Reader::from_string(
        "
var i = 0 
while i < 10000000 {
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

    let f = function_from_codeblock(&mut rt, code.clone(), "<main>");
    c.bench_function("interp-loop", |b| {
        b.iter(|| {
            let _ = rt.call(f, Value::undefined(), &[]);
        })
    });
    let mut rt = Runtime::new(Configs::default());
    let reader = Reader::from_string(
        "
var i = 0 
while i < 10000000 {
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

    let f = function_from_codeblock(&mut rt, code.clone(), "<main>");
    c.bench_function("jit-loop", |b| {
        b.iter(|| {
            let _ = rt.call(f, Value::undefined(), &[]);
        })
    });
}
fn interp_loop(c: &mut Criterion) {}

criterion_group! (
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = jit_loop
);

criterion_main!(benches);
