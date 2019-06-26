use std::path::PathBuf;
use structopt::StructOpt;

use waffle::codegen::Codegen;

use cranelift::codegen::isa;
use cranelift::codegen::settings::{self, Configurable};
use cranelift_faerie::FaerieBackend;
use cranelift_faerie::FaerieBuilder;
use cranelift_faerie::FaerieProduct;
use cranelift_simplejit::SimpleJITBackend;

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
    #[structopt(long = "aot", help = "Use AOT compilation instead of JIT compilation")]
    pub aot: bool,
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
}

fn main() {
    let opts: Options = Options::from_args();

    let mut context = waffle::Context {
        files: vec![],
        import_search_paths: vec![],
        library: false,
        merged: None,
    };
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

typedef size_t usize;
typedef size_t isize;

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
        file.write_all(cgen.buffer.as_bytes()).unwrap();
        if !opts.compile_only {
            linker(
                &output,
                &opts.libraries,
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
    if opts.aot {
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
                cranelift_faerie::FaerieBuilder::default_libcall_names(),
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
                "output.o",
                &opts.libraries,
                if opts.output.is_some() {
                    opts.output.as_ref().unwrap()
                } else {
                    "output.exe"
                },
            );
        }
    } else {
        let mut codegen: Codegen<SimpleJITBackend> = Codegen::<SimpleJITBackend>::new(
            ty_info,
            SimpleJITBuilder::new(),
            context.merged.unwrap().ast.clone(),
        );
        codegen.complex_types = complex;
        codegen.translate();

        let func = codegen.get_function("main").unwrap();

        let function: fn() -> isize = unsafe { std::mem::transmute(func) };

        println!("Result: {}", function());
    }
}

extern "C" {
    fn system(s: *const i8) -> i32;
}

fn linker(filename: &str, libs: &Vec<String>, output: &str) {
    let mut linker = String::from(&format!("gcc -lc -lpthread {} -o {} ", filename, output));
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
