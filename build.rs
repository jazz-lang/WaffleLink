use std::fs;
use std::process;
fn main() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let builtins = format!("{}/src/builtins/", manifest);
    let paths = fs::read_dir(&builtins).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if let Some(ext) = path.extension().map(|s| s.to_str().unwrap()) {
            if ext == ".jzl" {
                process::Command::new("jlightc")
                    .arg(path.to_str().unwrap().to_owned())
                    .arg(&format!(
                        "-o {}.bc",
                        path.file_name().unwrap().to_str().unwrap()
                    ))
                    .spawn()
                    .unwrap();
            }
        }
    }
}
