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

        compiler
            .chunk
            .write(Instruction::Return as u8, compiler.chunk.last_byte_line());

        compiler.chunk
    }

    fn get_constant(&mut self, value: Value) -> usize {
        self.constant_pool
            .get(&value)
            .copied()
            .unwrap_or_else(|| self.chunk.add_constant(value))
    }
    fn write_constant(&mut self, value: Value) {
        let constant = self.get_constant(value);

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
            StmtType::Var(name, value) => {
                self.visit_node(value, vm);
                let constant = self.get_constant(Value::Obj(
                    vm.alloc(Obj::new(ObjType::String(AnkokuString::new(name.clone())))),
                ));
                write_byte!(Instruction::DefineGlobal.into());
                write_byte!(constant as u8);
            }
        }
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
                    write_byte!(Instruction::ObjectSet.into());
                }
            }
            ExprType::Identifier(s) => {
                let constant = self.get_constant(Value::Obj(
                    vm.alloc(Obj::new(ObjType::String(AnkokuString::new(s.to_string())))), // intern this too
                ));

                write_byte!(Instruction::GetGlobal.into());
                write_byte!(constant as u8);
            }
            ExprType::Assign(name, value) => {
                self.visit_node(value, vm);
                let constant = self.get_constant(Value::Obj(vm.alloc(Obj::new(ObjType::String(
                    AnkokuString::new(name.to_string()),
                )))));

                write_byte!(Instruction::SetGlobal.into());
                write_byte!(constant as u8);
            }
            ExprType::String(s) => {
                self.write_constant(Value::Obj(
                    vm.alloc(Obj::new(ObjType::String(AnkokuString::new(s.to_string())))), // intern this too
                ));
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compiler::Compiler,
        parser::{stmt::Stmt, tokenizer::Tokenizer, ParserError},
        vm::{InterpretResult, VM},
    };

    fn parse_stmts<S: AsRef<str>>(source: S) -> (Vec<Stmt>, Vec<ParserError>) {
        let tokens = Tokenizer::new(source.as_ref())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", tokens);
        let (stmts, errors) = Stmt::parse(tokens, source.as_ref().chars().collect());
        println!("{:#?}", stmts);

        (stmts, errors)
    }

    fn parse_stmts_unwrap<S: AsRef<str>>(source: S) -> Vec<Stmt> {
        let (stmts, errors) = parse_stmts(source);
        if !errors.is_empty() {
            for err in errors {
                println!("{:?}", err);
            }
            panic!("errors; see stdout")
        } else {
            stmts
        }
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
        let stmt = parse_stmts_unwrap("print 1 + 2 * 3 - 4 / -5; print 15;");
        let mut vm = VM::new();
        let compiled = Compiler::compile(&stmt, &vm);
        compiled.disassemble("compiled");
        let result = vm.interpret(compiled);
        assert_eq!(result, InterpretResult::Ok);
    }
    #[test]
    fn objects() {
        let stmt = parse_stmts_unwrap("print { a = 1, b = 2 }; print 1;");
        let mut vm = VM::new();
        let compiled = Compiler::compile(&stmt, &vm);
        compiled.disassemble("compiled");
        let result = vm.interpret(compiled);
        assert_eq!(result, InterpretResult::Ok);
    }
    #[test]
    fn variables() {
        let stmt = parse_stmts_unwrap("var a = 12; print a; a = 13; print a;");
        let mut vm = VM::new();
        let compiled = Compiler::compile(&stmt, &vm);
        compiled.disassemble("compiled");
        let result = vm.interpret(compiled);
        assert_eq!(result, InterpretResult::Ok);
    }
}
