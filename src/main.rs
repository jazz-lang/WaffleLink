use std::path::PathBuf;
use structopt::StructOpt;

use waffle::codegen::Codegen;

use cranelift_simplejit::SimpleJITBackend;
use cranelift_simplejit::SimpleJITBuilder;
//#[global_allocator]
//static A: mimallocator::Mimalloc = mimallocator::Mimalloc;

#[derive(StructOpt)]
#[structopt(name = "waffle", about = "compiler")]
pub struct Options {
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,
}

fn main() {
    let opts: Options = Options::from_args();

    let mut context = waffle::Context {
        files: vec![],
        import_search_paths: vec![],
        library: false,
        merged: None,
    };
    let start = time::PreciseTime::now();
    context.parse(opts.path.to_str().unwrap());
    use waffle::tycheck::TypeChecker;
    let mut checker = TypeChecker::new(&mut context);

    checker.run();
    let ty_info = checker.type_info.clone();
    let end = time::PreciseTime::now();

    println!(
        "Time wasted on parsing and typechecking: {} ms",
        start.to(end).num_milliseconds()
    );
    let complex = checker.complex.clone();

    let mut codegen: Codegen<SimpleJITBackend> = Codegen::<SimpleJITBackend>::new(
        ty_info,
        SimpleJITBuilder::new(),
        context.merged.unwrap().ast.clone(),
    );
    codegen.complex_types = complex;
    codegen.translate();

    let main_func = codegen.get_function("main").unwrap();

    let fun: fn() -> isize = unsafe { std::mem::transmute(main_func) };

    println!("{}", fun());
}
