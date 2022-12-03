use crate::{
    parser::ast::{AstNode, AstType, AstVisitor},
    vm::{chunk::Chunk, instruction::Instruction},
};

pub struct Compiler {
    chunk: Chunk,
}
impl Compiler {
    pub fn compile(ast: &AstNode) -> Chunk {
        let mut compiler = Compiler {
            chunk: Chunk::new(),
        };
        compiler.visit_node(ast);
        compiler.chunk
    }
}
impl AstVisitor<()> for Compiler {
    fn visit_node(&mut self, node: &AstNode) {
        macro_rules! write_byte {
            ($b:expr) => {
                self.chunk.write($b, node.token.line as usize);
            };
        }
        match &node.kind {
            AstType::Real(n) => {
                let constant = self.chunk.add_constant((*n).into());
                write_byte!(Instruction::Constant.into());
                write_byte!(constant);
            }
            AstType::Bool(_) => todo!(),
            AstType::Null => todo!(),
            AstType::Add(l, r) => {
                self.visit_node(l);
                self.visit_node(r);

                write_byte!(Instruction::Add.into());
            }
            AstType::Subtract(l, r) => {
                self.visit_node(l);
                self.visit_node(r);

                write_byte!(Instruction::Sub.into());
            }
            AstType::Multiply(l, r) => {
                self.visit_node(l);
                self.visit_node(r);

                write_byte!(Instruction::Mul.into());
            }
            AstType::Divide(l, r) => {
                self.visit_node(l);
                self.visit_node(r);

                write_byte!(Instruction::Div.into());
            }
            AstType::Negate(i) => {
                self.visit_node(i);

                write_byte!(Instruction::Negate.into());
            }
            AstType::Not(i) => {
                self.visit_node(i);

                write_byte!(Instruction::Not.into());
            }
            AstType::Grouping(b) => {
                self.visit_node(b);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compiler::visitor::Compiler,
        parser::{ast::AstNode, tokenizer::Tokenizer, ParserResult},
        vm::{instruction::Instruction, value::Value, InterpretResult, VM},
    };

    fn parse_expr<S: AsRef<str>>(source: S) -> ParserResult<AstNode> {
        let tokens = Tokenizer::new(source.as_ref())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let ast = AstNode::parse(tokens, source.as_ref().chars().collect());

        ast
    }

    #[test]
    fn basic() {
        let expr = parse_expr("1 + 2 * 3 - 4 / -5").unwrap();
        println!("{:#?}", expr);
        let mut compiled = Compiler::compile(&expr);
        compiled.disassemble("compiled");
        compiled.write(Instruction::Return.into(), 1);
        let (result, _vm) = VM::interpret(&compiled);
        assert_eq!(result, InterpretResult::Ok(Value::Real(7.8)));
    }
}
