pub mod lexer;

use crate::lexer::lexer as l;

fn main() {
    let contents = "HAI 1.2\nI HAS A var ITZ 0\nIM IN YR loop UPPIN YR var TIL BOTH SAEM var AN 10,VISIBLE var \" Yessir\"\nIM OUTTA YR loop\nKTHXBYE";

    let mut l = l::Lexer::init(contents);
    let tokens = l.get_tokens();

    println!("{:?}\n\n", tokens);

    if l::Lexer::has_errors(&tokens) {
        println!("{:#?}\n\n", l::Lexer::get_first_error(&tokens).unwrap());
        return;
    }
}
