pub mod compiler;
pub mod lexer;
pub mod parser;

use compiler::target::Target;

use crate::compiler::ir::{IRFunction, IRFunctionEntry, IRStatement, IR};
use crate::compiler::target::vm::VM;
use crate::lexer::lexer as l;
use crate::parser::parser as p;

fn main() {
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

    let t = VM {};

    let ir = IR::new(
        vec![IRFunction::new(
            "sum".to_string(),
            vec![
                IRStatement::EstablishStackFrame,
                IRStatement::Push(2.0),
                IRStatement::Copy,
                IRStatement::Push(3.0),
                IRStatement::Copy,
                IRStatement::Add,
                IRStatement::SetReturnRegister,
                IRStatement::AccessReturnRegister,
                IRStatement::Push(4.0),
                IRStatement::Copy,
                IRStatement::Store(1),
                IRStatement::EndStackFrame(2, 0),
            ],
        )],
        IRFunctionEntry::new(
            100,
            400,
            vec![
                IRStatement::Push(4.0),
                IRStatement::Push(4.0),
                IRStatement::Allocate,
                IRStatement::Push(2.0),
                IRStatement::Push(1.0),
                IRStatement::Call("sum".to_string()),
                IRStatement::AccessReturnRegister,
                IRStatement::CallForeign("prn".to_string()),
                IRStatement::CallForeign("prend".to_string()),
                IRStatement::Push(-2.0),
                IRStatement::Copy,
                IRStatement::Load(1),
                IRStatement::CallForeign("prn".to_string()),
                IRStatement::CallForeign("prend".to_string()),
                IRStatement::Free,
            ],
        ),
    );

    let code = ir.assemble(&t);
    t.compile(code).expect("Failed to compile");
}
