use ankoku::{
    compiler::visitor::Compiler,
    parser::{ast::AstNode, tokenizer::Tokenizer},
    vm::{instruction::Instruction, VM},
};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let input = &args[1];
    let source = std::fs::read_to_string(input).unwrap();

    let tokens = Tokenizer::new(source.as_ref())
        .map(|v| v.unwrap())
        .collect::<Vec<_>>();

    let ast = AstNode::parse(tokens, source.chars().collect()).expect("parse error");
    let mut compiled = Compiler::compile(&ast);
    compiled.disassemble("compiled");
    compiled.write(Instruction::Return.into(), 1);
    let mut vm = VM::new();
    vm.interpret(compiled);
}
