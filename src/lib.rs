#![feature(box_syntax)]
#![feature(const_fn)]
#![feature(const_transmute)]

pub static mut DUMP_IR: bool = false;

#[macro_use]
pub mod macros;
pub mod ast;
pub mod cgen;
pub mod codegen;
pub mod err;
pub mod lexer;
pub mod parser;
pub mod reader;
pub mod tycheck;
#[derive(Clone)]
pub struct File {
    pub ast: Vec<ast::Element>,
    pub main: bool,
    pub path: String,
    pub src: String,
    pub name: String,
}

unsafe impl Send for File {}
unsafe impl Sync for File {}

use std::collections::HashSet;

pub struct Context {
    pub import_search_paths: Vec<String>,
    pub files: Vec<File>,
    pub library: bool,
    pub merged: Option<File>,
    pub path: String
}

use ast::Element;
use walkdir::WalkDir;
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
use parser::Parser;
use reader::Reader;
use rayon::iter::*;
impl Context {
    pub fn parse(&mut self, path: &str) {
        let spath = std::path::Path::new(path);
        let mut files = vec![];
        if spath.is_dir() {
            walk_directories(path, &mut files);
        } else {
            files.push(path.to_owned());
        }
        if spath.is_dir() {
            self.path = path.to_owned();
        } else {
            let tmp_path = std::path::Path::new(path);
            self.path = tmp_path.parent().unwrap().to_str().unwrap().to_owned();
        }
        let failed = std::sync::atomic::AtomicBool::new(false);
        let fail_count = std::sync::atomic::AtomicI32::new(0);
        self.files = files
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
                                eprintln!("{}\n", e);
                                failed.store(true, std::sync::atomic::Ordering::Relaxed);
                                fail_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
        self.process();
    }

    pub fn process(&mut self) {
        let mut merge_file = File {
            ast: vec![],
            main: true,
            src: String::new(),
            path: String::new(),
            name: String::from("<<merged>>"),
        };

        let mut in_module = HashSet::<String>::new();

        for file in self.files.iter() {
            merge_file.ast.extend_from_slice(&file.ast);
            merge_file.src = format!("{}\n{}", merge_file.src, file.src);
            in_module.insert(file.path.clone());
        }

        let mut imports = std::collections::HashSet::new();
        merge_file.ast.iter().for_each(|x| {
            if let Element::Import(ref name) = x {
                imports.insert(name.name.clone());
            }
        });
        
        let home_path = env!("HOME").to_owned();
        let home_path = home_path + "/.jazz";
        let home_path = std::path::Path::new(&home_path);
        let files = imports
            .par_iter()
            .map(|import| {
                let mut ctx = Context {
                    merged: None,
                    library: false,
                    files: vec![],
                    import_search_paths: vec![],
                    path: self.path.clone()
                };
                let mut full_path = String::new();
                full_path.push_str(home_path.to_str().unwrap());
                full_path.push('/');
                full_path.push_str(import);
                if std::path::Path::new(&full_path).exists() {
                    ctx.parse(&full_path);
                } else {
                
                
                let mut other_path = String::new();
                other_path.push_str(&self.path);
                other_path.push('/');
                other_path.push_str(import);

                ctx.parse(&other_path);
                }
                assert!(ctx.merged.is_some());

                ctx.merged.unwrap().clone()
            })
            .collect::<Vec<_>>();;

        for file in files.into_iter() {
            
            merge_file.ast.extend_from_slice(&file.ast);
        }

        self.merged = Some(merge_file);
    }
}
