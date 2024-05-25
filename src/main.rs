pub mod lexer;
pub mod parser;

use crate::lexer::lexer as l;
use crate::parser::parser as p;

use rust_embed::{Embed, EmbeddedFile};
use std::io::Write;

#[derive(Embed)]
#[folder = "deps/"]
struct Assets;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        std::fs::remove_dir_all("runtime-deps").expect("Failed to remove runtime-deps directory");
    }
}

fn main() {
    let _cleanup = Cleanup;

    let mut as_exe: Option<EmbeddedFile> = None;
    let mut ld_exe: Option<EmbeddedFile> = None;
    let mut qbe_exe: Option<EmbeddedFile> = None;
    for (i, file) in Assets::iter().enumerate() {
        match i {
            0 => as_exe = Some(Assets::get(&file).unwrap()),
            1 => ld_exe = Some(Assets::get(&file).unwrap()),
            2 => qbe_exe = Some(Assets::get(&file).unwrap()),
            _ => panic!("Unknown file: {:?}", file),
        }
    }

    std::fs::DirBuilder::new()
        .create(std::path::Path::new("runtime-deps"))
        .unwrap();
    for (i, file) in Assets::iter().enumerate() {
        let mut file = std::fs::File::create(format!("runtime-deps/{}", file)).unwrap();

        match i {
            0 => file
                .write_all(as_exe.clone().unwrap().data.as_ref())
                .unwrap(),
            1 => file
                .write_all(ld_exe.clone().unwrap().data.as_ref())
                .unwrap(),
            2 => file
                .write_all(qbe_exe.clone().unwrap().data.as_ref())
                .unwrap(),
            _ => panic!("Unknown file: {:?}", file),
        }
    }

    let contents = "HAI 1.2,HOW IZ I sum ITZ NUMBER YR a ITZ NUMBER AN YR b ITZ NUMBER,FOUND YR SUM OF a AN b,IF U SAY SO,KTHXBYE";

    let mut l = l::Lexer::init(contents);
    let tokens = l.get_tokens();

    println!("{:?}\n\n", tokens);

    if l::Lexer::has_errors(&tokens) {
        println!("{:#?}\n\n", l::Lexer::get_first_error(&tokens).unwrap());
        return;
    }

    let p = p::Parser::parse(tokens);
    println!("{:?}\n\n", p.ast);

    for error in p.errors.iter() {
        println!("{:#?}", error);
    }
    if p.errors.len() > 0 {
        println!("\n\n");
        return;
    }
}
