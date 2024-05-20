#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod lexer;
pub mod parser;

use crate::lexer::lexer as l;
use crate::parser::parser as p;

fn main() {
    let contents = "HAI 1.2, I HAS A x ITZ YARN \n\nR \"hello world\",GIMMEH x,VISIBLE \"hello \" \"world::)\",KTHXBYE";

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
