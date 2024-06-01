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
                IRStatement::LoadBasePtr,
                IRStatement::Push(2.0),
                IRStatement::Subtract,
                IRStatement::Copy,
                IRStatement::LoadBasePtr,
                IRStatement::Push(3.0),
                IRStatement::Subtract,
                IRStatement::Copy,
                IRStatement::Add,
                IRStatement::SetReturnRegister,
                IRStatement::AccessReturnRegister,
                IRStatement::LoadBasePtr,
                IRStatement::Push(4.0),
                IRStatement::Subtract,
                IRStatement::Copy,
                IRStatement::Store(1),            // store 1 float on the heap
                IRStatement::EndStackFrame(2, 0), // add will destroy copied args and set return register destroys add (don't destory heap pointer)
            ],
        )],
        IRFunctionEntry::new(
            100, // 100 floats = 400 bytes
            400, // 400 bytes
            vec![
                IRStatement::Push(0.0), // garbage return address which won't be used
                IRStatement::EstablishStackFrame,
                IRStatement::Push(4.0), // 4 bytes for the return register (just testing heap) (keep this so that when we call free later it is arg1 and the heap pointer is arg2)
                IRStatement::Push(4.0), // 4 bytes for the return register (just testing heap)
                IRStatement::Allocate, // will push the address of the allocated memory to the stack as an arg
                IRStatement::Push(2.0), // b
                IRStatement::Push(1.0), // a
                IRStatement::Call("sum".to_string()), // call sum with a and b
                IRStatement::AccessReturnRegister, // push eax (return register) to the stack
                IRStatement::CallForeign("prn".to_string()), // print the result
                IRStatement::CallForeign("prend".to_string()), // print the endline
                IRStatement::LoadBasePtr,
                IRStatement::Push(2.0),
                IRStatement::Add, // we want to get a local variable not an arg
                IRStatement::Copy,
                IRStatement::Load(1), // load the float from the heap
                IRStatement::CallForeign("prn".to_string()), // print the result
                IRStatement::CallForeign("prend".to_string()), // print the endline
                IRStatement::Free,    // the original pointer is still on the stack
                IRStatement::EndStackFrame(0, 0),
            ],
        ),
    );

    let code = ir.assemble(&t);
    t.compile(code).expect("Failed to compile");
}
