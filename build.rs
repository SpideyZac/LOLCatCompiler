extern crate cc;
extern crate bindgen;

use std::fs;
use std::env;

fn parse_dir(path: &str, builder: &mut cc::Build) {
    for file in fs::read_dir(path).unwrap() {
        let f = file.unwrap();
        let path = f.path();
        let p = path.to_str().unwrap();

        if path.is_dir() {
            parse_dir(p, builder);
        } else {
            if p.ends_with(".c") {
                builder.file(p);
            }
        }
    }
}

fn main() {
    env::set_var("CRATE_CC_NO_DEFAULTS", "1");
    env::set_var("CC", "gcc");

    let mut builder = cc::Build::new();
    builder.std("c99");
    builder.flag("-g");
    builder.flag("-Wall");
    builder.flag("-Wextra");
    builder.flag("-Wpedantic");
    parse_dir("./deps/qbe-1.2", &mut builder);

    builder.compile("qbe");

    let bindings = bindgen::Builder::default()
        .header("./deps/qbe-1.2/wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}