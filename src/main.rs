use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt)]
#[structopt(name = "waffle", about = "compiler")]
pub struct Options {
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,
}

fn walk_directories(path: &str, files: &mut Vec<String>) {
    let walk = WalkDir::new(path);
    for (i, entry) in walk.into_iter().enumerate() {
        if i == 0 {
            continue;
        }
        let entry: &walkdir::DirEntry = &entry.unwrap();

        if entry.file_type().is_file() {
            let path = entry.path().to_str().unwrap().to_owned();
            if path.ends_with(".waffle") {
                files.push(path);
            }
        } else if entry.file_type().is_dir() {
            walk_directories(entry.path().to_str().unwrap(), files);
        }
    }
}

use waffle::{parser::Parser, reader::Reader, Context, File};

fn main() {
    let opts: Options = Options::from_args();

    let mut files = vec![];

    if opts.path.is_dir() {
        walk_directories(opts.path.to_str().unwrap(), &mut files);
    } else {
        files.push(opts.path.to_str().unwrap().to_owned());
    }

    let mut context = Context {
        files: vec![],
        import_search_paths: vec![],
        library: false,
        merged: None,
    };

    context.parse(opts.path.to_str().unwrap());
    use waffle::tycheck::TypeChecker;
    let mut checker = TypeChecker::new(&mut context);

    checker.run();
    /*use rayon::iter::*;
    let failed = std::sync::atomic::AtomicBool::new(false);
    let fail_count = std::sync::atomic::AtomicI32::new(0);
    context.files = files
        .par_iter()
        .map(|file_path| {
            let path = std::path::Path::new(file_path);
            let mut ast_file = File {
                ast: vec![],
                src: String::new(),
                name: String::new(),
                path: String::new(),
                main: path.file_name().unwrap() == "main.waffle",
            };

            let reader = Reader::from_file(file_path);
            match reader {
                Ok(reader) => {
                    let mut parser = Parser::new(reader, &mut ast_file);
                    let parse_result = parser.parse();
                    match parse_result {
                        Ok(_) => (),
                        Err(e) => {
                            if ast_file.main {
                                eprintln!("{}\n", e);
                                failed.store(true, std::sync::atomic::Ordering::Relaxed);
                                fail_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            } else {
                                eprintln!("{}\n", e);
                                failed.store(true, std::sync::atomic::Ordering::Relaxed);
                                fail_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    failed.store(true, std::sync::atomic::Ordering::Relaxed);
                    fail_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
            ast_file
        })
        .collect::<Vec<_>>();

    if failed.load(std::sync::atomic::Ordering::Relaxed) {
        eprintln!(
            "Compilation failed\nFails count: {}",
            fail_count.load(std::sync::atomic::Ordering::Relaxed)
        );
        std::process::exit(-1);
    }
    */
}
