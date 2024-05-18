pub mod lexer;
pub mod parser;

use crate::lexer::lexer as l;
use crate::parser::parser as p;

fn main() {
    let contents = "0";

    let mut l = l::Lexer::init(contents);
    let tokens = l.get_tokens();

    println!("{:?}\n\n", tokens);

    if l::Lexer::has_errors(&tokens) {
        println!("{:#?}\n\n", l::Lexer::get_first_error(&tokens).unwrap());
        return;
    }
}
