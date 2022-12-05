use std::collections::HashMap;

use crate::{
    parser::{
        expr::{AstVisitor, Expr, ExprType},
        stmt::{Stmt, StmtType},
    },
    util::fxhash::FxHashMap,
    vm::{
        chunk::Chunk,
        instruction::Instruction,
        obj::{AnkokuString, Obj, ObjType},
        value::Value,
        VM,
    },
};

pub struct Compiler {
    chunk: Chunk,
    constant_pool: FxHashMap<Value, usize>,
}
impl Compiler {
    pub fn compile(stmts: &[Stmt], vm: &VM) -> Chunk {
        let mut compiler = Compiler {
            chunk: Chunk::new(),
            constant_pool: HashMap::default(),
        };
        for stmt in stmts {
            compiler.visit_stmt(stmt, vm);
        }
        compiler.chunk
    }

    fn write_constant(&mut self, value: Value) {
        let constant = self
            .constant_pool
            .get(&value)
            .copied()
            .unwrap_or_else(|| self.chunk.add_constant(value));

        self.chunk
            .write(Instruction::Constant.into(), self.chunk.last_byte_line());
        self.chunk
            .write(constant as u8, self.chunk.last_byte_line());
    }
}
impl AstVisitor<(), ()> for Compiler {
    fn visit_stmt(&mut self, stmt: &Stmt, vm: &VM) {
        macro_rules! write_byte {
            ($b:expr) => {
                self.chunk.write($b, self.chunk.last_byte_line());
            };
        }

        match &stmt.kind {
            StmtType::Expr(e) => {
                self.visit_node(e, vm);
                write_byte!(Instruction::Pop as u8);
            }
            StmtType::Print(e) => {
                self.visit_node(e, vm);
                write_byte!(Instruction::Print as u8);
            }
        }

        write_byte!(Instruction::Return as u8);
    }

    fn visit_node(&mut self, node: &Expr, vm: &VM) {
        macro_rules! write_byte {
            ($b:expr) => {
                self.chunk.write($b, node.token.line as usize);
            };
        }
        match &node.kind {
            ExprType::Real(n) => {
                self.write_constant((*n).into());
            }
            ExprType::Bool(_) => todo!(),
            ExprType::Null => todo!(),
            ExprType::Add(l, r) => {
                self.visit_node(l, vm);
                self.visit_node(r, vm);

                write_byte!(Instruction::Add.into());
            }
            ExprType::Subtract(l, r) => {
                self.visit_node(l, vm);
                self.visit_node(r, vm);

                write_byte!(Instruction::Sub.into());
            }
            ExprType::Multiply(l, r) => {
                self.visit_node(l, vm);
                self.visit_node(r, vm);

                write_byte!(Instruction::Mul.into());
            }
            ExprType::Divide(l, r) => {
                self.visit_node(l, vm);
                self.visit_node(r, vm);

                write_byte!(Instruction::Div.into());
            }
            ExprType::Negate(i) => {
                self.visit_node(i, vm);

                write_byte!(Instruction::Negate.into());
            }
            ExprType::Not(i) => {
                self.visit_node(i, vm);

                write_byte!(Instruction::Not.into());
            }
            ExprType::Grouping(b) => {
                self.visit_node(b, vm);
            }
            ExprType::Object(table) => {
                write_byte!(Instruction::NewObject.into());

                for (key, value) in table {
                    self.write_constant(Value::Obj(vm.alloc(Obj::new(ObjType::String(
                        AnkokuString::new(key.to_string()),
                    )))));
                    self.visit_node(value, vm);
                    println!("{:?}", Instruction::ObjectSet);
                    write_byte!(Instruction::ObjectSet.into());
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compiler::Compiler,
        parser::{stmt::Stmt, tokenizer::Tokenizer, ParserResult},
        vm::{InterpretResult, VM},
    };

    fn parse_stmts<S: AsRef<str>>(source: S) -> ParserResult<Vec<Stmt>> {
        let tokens = Tokenizer::new(source.as_ref())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", tokens);
        let stmts = Stmt::parse(tokens, source.as_ref().chars().collect());
        println!("{:#?}", stmts);

        stmts
    }

    // #[test]
    // fn basic() {
    //     let expr = parse_expr("1 + 2 * 3 - 4 / -5").unwrap();
    //     let mut compiled = Compiler::compile(&expr);
    //     compiled.disassemble("compiled");
    //     let mut vm = VM::new();
    //     let result = vm.interpret(compiled);
    //     assert_eq!(result, InterpretResult::Ok);
    //     assert_eq!(vm.stack_pop(), Value::Real(7.8));
    // }

    #[test]
    fn statements() {
        let stmt = parse_stmts("print 1 + 2 * 3 - 4 / -5; print 15;").unwrap();
        let mut vm = VM::new();
        let compiled = Compiler::compile(&stmt, &vm);
        compiled.disassemble("compiled");
        let result = vm.interpret(compiled);
        assert_eq!(result, InterpretResult::Ok);
    }
    #[test]
    fn objects() {
        let stmt = parse_stmts("print { a = 1, b = 2 }; print 1;").unwrap();
        let mut vm = VM::new();
        let compiled = Compiler::compile(&stmt, &vm);
        compiled.disassemble("compiled");
        let result = vm.interpret(compiled);
        assert_eq!(result, InterpretResult::Ok);
    }
}
