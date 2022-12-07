// TODO: flesh this out, probably move to separate crate
#[cfg(feature = "cli")]
fn main() {
    use std::process::exit;

    use ankoku::{
        compiler::Compiler,
        parser::{stmt::Stmt, tokenizer::Tokenizer},
        util::error::{cli::CLIErrorReporter, ErrorReporter},
        vm::{instruction::Instruction, VM},
    };

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("usage: ankoku <file.ak>");
        exit(1);
    }
    let input = &args[1];
    let source = std::fs::read_to_string(input).unwrap();

    let tokens = Tokenizer::new(source.as_ref())
        .map(|v| v.unwrap())
        .collect::<Vec<_>>();

    let (ast, errors) = Stmt::parse(tokens, source.chars().collect());
    if !errors.is_empty() {
        let reporter = CLIErrorReporter;
        for err in errors {
            #[cfg(feature = "debug-mode")]
            println!("{:?}", err);
            reporter.report(err);
        }
        return;
    }
    let mut vm = VM::new();
    let mut compiled = Compiler::compile(&ast, &vm);
    compiled.disassemble("compiled");
    compiled.write(Instruction::Return.into(), 1);
    vm.interpret(compiled);
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("must build with feature 'cli' to build the cli");
}
