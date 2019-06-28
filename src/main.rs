use std::path::PathBuf;
use structopt::StructOpt;

use waffle::codegen::Codegen;

use cranelift::codegen::isa;
use cranelift::codegen::settings::{self, Configurable};
use cranelift_faerie::FaerieBackend;
use cranelift_faerie::FaerieBuilder;
use cranelift_faerie::FaerieProduct;
use cranelift_simplejit::SimpleJITBackend;
use cranelift_module::default_libcall_names;

use cranelift_simplejit::SimpleJITBuilder;
use std::str::FromStr;
use target_lexicon::triple;
#[derive(StructOpt)]
#[structopt(name = "waffle", about = "compiler")]
pub struct Options {
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,

    #[structopt(short = "l", help = "Link with library")]
    pub libraries: Vec<String>,
    #[structopt(long = "aot", help = "Use JIT compilation instead of AOT compilation")]
    pub jit: bool,
    #[structopt(
        short = "c",
        help = "Compiler will output object file or C code file (you can not use that config without --aot or --emit-c"
    )]
    pub compile_only: bool,
    #[structopt(short = "o", help = "Set output filename")]
    pub output: Option<String>,
    #[structopt(long = "dump-ir", help = "Dump Cranelift IR to stdout")]
    pub dump_ir: bool,
    #[structopt(long = "target", help = "Set target triple")]
    pub target: Option<String>,
    #[structopt(long = "emit-c", help = "Generate C code")]
    pub emit_c: bool,
    #[structopt(long = "show-time", help = "Display compilation time")]
    pub time: bool,
    #[structopt(
        short = "O",
        long = "opt-level",
        help = "Optimization level ( possible values: 0,1,2,3 )"
    )]
    pub opt_level: Option<usize>,
    #[structopt(long = "cc", help = "Specify C compiler for linking/compiling C files",parse(from_str))]
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
    };
    let start = time::PreciseTime::now();
    context.parse(opts.path.to_str().unwrap());
    use waffle::tycheck::TypeChecker;
    let mut checker = TypeChecker::new(&mut context);

    checker.run();
    let ty_info = checker.type_info.clone();

    let complex = checker.complex.clone();

    if opts.emit_c {
        let mut cgen = waffle::cgen::CCodeGen::new();
        cgen.complex_types = complex;
        cgen.ty_info = ty_info;
        cgen.buffer = "
#include <stddef.h>
typedef unsigned long ulong;
typedef unsigned int uint;
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
        let end = time::PreciseTime::now();
        if opts.time {
            println!(
                "Compilation time {} ms (without C file compiling)",
                start.to(end).num_milliseconds()
            );
        }
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

        return;
    }

    if opts.dump_ir {
        unsafe {
            waffle::DUMP_IR = true;
        }
    }
    if !opts.jit {
        let triple_ = if opts.target.is_some() {
            opts.target.unwrap().clone()
        } else {
            "x86_64-unknown-unknown-elf".to_owned()
        };
        let mut flag_builder = settings::builder();
        flag_builder.enable("is_pic").unwrap();
        flag_builder.set("opt_level", "fastest").unwrap();
        let t_ref = &triple_;
        let isa_builder = isa::lookup(triple!(t_ref)).unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder));
        let mut codegen: Codegen<FaerieBackend> = Codegen::<FaerieBackend>::new(
            ty_info,
            FaerieBuilder::new(
                isa,
                "waffle".to_owned(),
                cranelift_faerie::FaerieTrapCollection::Disabled,
                default_libcall_names(),
            )
            .unwrap(),
            context.merged.unwrap().ast.clone(),
        );
        codegen.complex_types = complex;
        codegen.translate();
        let product: FaerieProduct = codegen.module.finish();

        let file = std::fs::File::create("output.o").expect("faile");
        product.write(file).unwrap();

        if !opts.compile_only {
            linker(
                if opts.cc.is_some() {
                    opts.cc.as_ref().unwrap()
                } else {
                    "clang"
                },
                "output.o",
                &opts.libraries,
                opts.opt_level.unwrap_or(2),
                if opts.output.is_some() {
                    opts.output.as_ref().unwrap()
                } else {
                    "output.exe"
                },
            );
        }
        let end = time::PreciseTime::now();
        if opts.time {
            println!("Compilation time {} ms", start.to(end).num_milliseconds());
        }
    } else {
        if !opts.libraries.is_empty() {
            panic!("Linking with libraries not supported in JIT mode");
        }
        let mut codegen: Codegen<SimpleJITBackend> = Codegen::<SimpleJITBackend>::new(
            ty_info,
            SimpleJITBuilder::new(default_libcall_names()),
            context.merged.unwrap().ast.clone(),
        );
        // Load runtime
        unsafe {
            let c_str: std::ffi::CString = std::ffi::CString::new("libwaffle_runtime.so").unwrap();
            let handle = libc::dlopen(c_str.as_ptr(), libc::RTLD_LAZY);
            if handle.is_null() {
                panic!("Could not load language runtime");
            }
        }
        

        codegen.complex_types = complex;
        codegen.translate();

        let func = codegen.get_function("main").unwrap();

        let function: fn() -> isize = unsafe { std::mem::transmute(func) };
        let end = time::PreciseTime::now();
        if opts.time {
            println!("Compilation time {} ms\n", start.to(end).num_milliseconds());
        }
        println!("Result: {}", function());
    }
}

extern "C" {
    fn system(s: *const i8) -> i32;
}

fn linker(cc: &str, filename: &str, libs: &Vec<String>, opt_level: usize, output: &str) {
    let mut linker = String::from(&format!(
        "{} -O{} -lc -lpthread -lwaffle_runtime {} -o {}  ",
        cc, opt_level, filename, output
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
