use std::process::exit;

use ankoku::{
    compiler::Compiler,
    parser::{stmt::Stmt, tokenizer::Tokenizer},
    util::error::{AnkokuError, ErrorReporter},
    vm::{instruction::Instruction, VM},
};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("usage: ankoku <file>");
        println!("a basic cli for ankoku");
        exit(1);
    }
    let input = &args[1];
    let source = std::fs::read_to_string(input).unwrap();

    let tokens = Tokenizer::new(source.as_ref())
        .map(|v| {
            if let Err(v) = v {
                let reporter = CLIErrorReporter;
                reporter.report(v);
                panic!("tokenizer error") // TODO: "Don't Panic"
            } else {
                v.unwrap()
            }
        })
        .collect::<Vec<_>>();

    let (ast, errors) = Stmt::parse(tokens, source.chars().collect());
    if !errors.is_empty() {
        let reporter = CLIErrorReporter;
        for err in errors {
            reporter.report(err);
        }
        return;
    }
    let mut vm = VM::new();
    let mut compiled = Compiler::compile(&ast, &vm);
    compiled.disassemble("CLI compiled chunk");
    compiled.write(Instruction::Return.into(), 1);
    vm.interpret(compiled);
}

use owo_colors::OwoColorize;

pub struct CLIErrorReporter;

impl ErrorReporter for CLIErrorReporter {
    fn report<E: AnkokuError>(&self, err: E) {
        if let Some((line, col, content)) = err.line_col() {
            println!(
                "{} {:04}: {}",
                "error".bright_red().bold(),
                format!("AK{}", err.code()).bold(),
                err.msg()
            );
            // println!("{} todo filename", "-->".bold().bright_cyan());

            let bottom_highlight = || {
                format!(
                    "{}{}",
                    " ".repeat(col - 1),
                    "^".repeat(err.length().unwrap_or(1)).bold().yellow(),
                )
            };
            // 4 digits ought to be enough for anyone
            if line < 100 {
                println!("{}", "    |".bold().bright_cyan());
                println!(
                    "{} {}",
                    format!(" {:2} |", line).bold().bright_cyan(),
                    content
                );
                println!("{} {}", "    |".bold().bright_cyan(), bottom_highlight());
            } else if line < 1000 {
                println!("{}", "     |".bold().bright_cyan());
                println!(
                    "{} {}",
                    format!(" {:3} |", line).bold().bright_cyan(),
                    content
                );
                println!("{} {}", "     |".bold().bright_cyan(), bottom_highlight());
            } else if line < 10000 {
                println!("{}", "      |".bold().bright_cyan());
                println!(
                    "{} {}",
                    format!(" {:4} |", line).bold().bright_cyan(),
                    content
                );
                println!("{} {}", "      |".bold().bright_cyan(), bottom_highlight());
            }
        } else {
            println!("error has no line");
        }
    }
}
