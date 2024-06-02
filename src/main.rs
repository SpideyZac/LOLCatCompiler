pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod utils;

use compiler::target::Target;

use crate::compiler::ir::{IRFunction, IRFunctionEntry, IRStatement, IR};
use crate::compiler::target::vm::VM;
use crate::lexer::lexer as l;
use crate::lexer::tokens as t;
use crate::parser::parser as p;
use crate::utils::get_line;

fn main() {
    let contents = "HAI 1.2\nHOW IZ I sum ITZ NUMBER YR a ITZ NOOB AN YR b ITZ NUMBER\nFOUND YR SUM OF a AN b\nIF U SAY SO\nKTHXBYE";
    let lines = contents.split("\n").collect::<Vec<&str>>();

    let mut l = l::Lexer::init(contents);
    let tokens = l.get_tokens();

    if l::Lexer::has_errors(&tokens) {
        let error = l::Lexer::get_first_error(&tokens).unwrap();

        let (line, count) = get_line(&lines, error.start);

        match &error.token {
            t::Token::Illegal(e) => {
                println!("{}", lines[line]);
                let arrow =
                    " ".repeat(error.start - count) + "^".repeat(error.end - error.start).as_str();
                println!("{}", arrow);
                println!(
                    "Error: {} at line {}, column {}:{}",
                    e,
                    line + 1,
                    error.start - count + 1,
                    error.end - count + 1
                );
            }
            _ => {
                panic!("Unexpected error token");
            }
        }

        return;
    }

    let p = p::Parser::parse(tokens);

    if p.errors.len() > 0 {
        let reversed = p.errors.iter().rev().collect::<Vec<&p::ParserError>>();

        for (i, error) in reversed.iter().enumerate() {
            let (line, count) = get_line(&lines, error.token.start);

            println!("{}", lines[line]);
            let arrow = " ".repeat(error.token.start - count)
                + "^".repeat(error.token.end - error.token.start).as_str();
            println!("{}", arrow);
            println!(
                "Error: {} at line {}, column {}:{}",
                error.message,
                line + 1,
                error.token.start - count + 1,
                error.token.end - count + 1
            );

            if i != reversed.len() - 1 {
                println!("\nWhich was caused by:");
            }
        }

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
