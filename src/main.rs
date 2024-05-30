pub mod compiler;
pub mod lexer;
pub mod parser;

use crate::compiler::ir::{IRFunction, IRFunctionEntry, IRStatement, IR};
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

    let ir = IR::new(
        vec![IRFunction::new(
            "sum".to_string(),
            vec![
                IRStatement::EstablishStackFrame(12), // 4 bytes for a, 4 bytes for b, 4 bytes for return value
                IRStatement::LoadBasePtr,             // Load the base pointer
                // base_ptr + 12 = b
                // base_ptr + 8  = a
                // base_ptr + 4  = next instruction
                // base_ptr      = previous base pointer
                // base_ptr - 4  = return value
                IRStatement::Push(8.0),            // Push 8.0 for a
                IRStatement::Add,                  // Add 8.0 to base_ptr to get a
                IRStatement::Copy, // Copy a to front of stack (deletes the add stack frame instruction)
                IRStatement::LoadBasePtr, // Load the base pointer
                IRStatement::Push(12.0), // Push 12.0 for b
                IRStatement::Add,  // Add 12.0 to base_ptr to get b
                IRStatement::Copy, // Copy b to front of stack (deletes the add stack frame instruction)
                IRStatement::Add, // Add a and b (because they are at the front of the stack as copy deletes the add left overs)
                IRStatement::SetReturnRegister, // Set the return register to the result of the addition
                IRStatement::EndStackFrame(8, 12), // End the stack frame (8 bytes for arguments, 12 bytes for local scope)
            ],
        )],
        IRFunctionEntry::new(
            512,
            512,
            vec![
                IRStatement::Push(2.0), // b
                IRStatement::Push(1.0), // a
                IRStatement::Call("sum".to_string()),
                IRStatement::AccessReturnRegister,
            ],
        ),
    );
}
