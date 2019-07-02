use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use waffle::cgen::Gen;
#[derive(StructOpt)]
#[structopt(name = "waffle", about = "compiler")]
pub struct Options {
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,

    #[structopt(short = "l", help = "Link with library")]
    pub libraries: Vec<String>,
    #[structopt(long = "jit", help = "Use JIT compilation instead of AOT compilation")]
    pub jit: bool,
    #[structopt(
        short = "c",
        help = "Compiler will output object file or C code file (you can not use that config without --aot or --emit-c"
    )]
    pub compile_only: bool,
    #[structopt(short = "o", help = "Set output filename")]
    pub output: Option<String>,
    #[structopt(
        short = "O",
        long = "opt-level",
        help = "Optimization level ( possible values: 0,1,2,3 )"
    )]
    pub opt_level: Option<usize>,
    #[structopt(
        long = "cc",
        help = "Specify C compiler for linking/compiling C files",
        parse(from_str)
    )]
    pub cc: Option<String>,
}

fn main() {
    let opts: Options = Options::from_args();
    if opts.opt_level.is_some() {
        assert!(opts.opt_level.unwrap() <= 3);
    }
    let mut context = waffle::Context {
        files: vec![],
        import_search_paths: vec![],
        library: false,
        merged: None,
        path: String::new(),
        imported_std: false
    };
    context.parse(opts.path.to_str().unwrap());
    use waffle::check::TypeChecker;

    let mut checker = TypeChecker::new(&mut context);
    checker.run();

    let ty_info = checker.type_info.clone();
    let call_info = checker.call_info.clone();
    let complex = checker.complex.clone();

    let mut cgen = Gen::new();
    cgen.ty_info = ty_info;
    cgen.call_info = call_info;
    cgen.complex_types = complex.clone();

    cgen.buffer = "
#include <stddef.h>
#include <inttypes.h>
typedef unsigned long ulong;
typedef uint32_t uint;
typedef unsigned short ushort;
typedef double float64;
typedef float float32;
typedef size_t usize;
typedef size_t isize;
typedef unsigned char ubyte;
typedef char byte;
#ifndef true 
typedef unsigned char bool;
#define true 1
#define false 0
#endif
"
    .to_owned();
    cgen.gen_toplevel(&context.merged.unwrap().ast);
    let output = "output.c";
    if !std::path::Path::new("output.c").exists() {
        std::fs::File::create("output.c").unwrap();
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(&output)
        .unwrap();
    use std::io::Write;
    file.set_len(0).unwrap();
    file.write_all(cgen.buffer.as_bytes()).unwrap();

    if !opts.compile_only {
        linker(
            if opts.cc.is_some() {
                opts.cc.as_ref().unwrap()
            } else {
                "clang"
            },
            &output,
            &opts.libraries,
            opts.opt_level.unwrap_or(2),
            if opts.output.is_some() {
                opts.output.as_ref().unwrap()
            } else {
                "output.exe"
            },
        );
    }
}

extern "C" {
    fn system(s: *const i8) -> i32;
}

fn linker(cc: &str, filename: &str, libs: &Vec<String>, opt_level: usize, output: &str) {
    let lib = if cfg!(windows) { "" } else { "-lc -lpthread" };
    let mut linker = String::from(&format!(
        "{} -O{} {} {} -o {}  ",
        cc, opt_level, lib, filename, output
    ));
    for lib in libs.iter() {
        linker.push_str(&format!(" -l{} ", lib));
    }

    let cstr = std::ffi::CString::new(linker);

    unsafe {
        let exit_code = system(cstr.unwrap().as_ptr());

        if exit_code == -1 {
            panic!("Linking failed");
        }
    }
}
