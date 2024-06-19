pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod utils;

use clap::Parser;
use std::fs;

use compiler::target::Target;

use crate::compiler::target as targ;
use crate::compiler::visit as v;
use crate::lexer::lexer as l;
use crate::lexer::tokens as t;
use crate::parser::parser as p;
use crate::utils::get_line;

#[derive(Parser)]
#[command(name = "Lol Cat Compiler")]
#[command(version = "0.1.0")]
#[command(about = "A fast and efficient compiler for the LOLCODE programming language.", long_about = None)]
#[command(author = "SpideyZac")]
struct Cli {
    input_file: String,
    #[arg(short = 'o', long = "output")]
    output_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let contents = fs::read_to_string(cli.input_file.clone());
    if let Result::Err(_) = contents {
        println!("Error: Could not read file '{}'", cli.input_file);
        std::process::exit(1);
    }
    let contents = contents.unwrap();
    let contents = contents.as_str();
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

        std::process::exit(1);
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

        std::process::exit(1);
    }

    let mut v = v::Visitor::new(p, 1000, 4000);
    let (ir, errors, hooks) = v.visit();

    for error in errors.iter() {
        let token = &error.token.token;

        let (line, count) = get_line(&lines, token.start);

        println!("{}", lines[line]);
        let arrow = " ".repeat(token.start - count) + "^".repeat(token.end - token.start).as_str();
        println!("{}", arrow);
        println!(
            "Error: {} at line {}, column {}:{}",
            error.message,
            line + 1,
            token.start - count + 1,
            token.end - count + 1
        );
    }
    if errors.len() > 0 {
        std::process::exit(1);
    }

    let target = targ::vm::VM {};

    let asm = ir.assemble(&target, hooks);
    let _ = target.compile(asm, cli.output_file).unwrap();
}
