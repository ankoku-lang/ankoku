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

struct Local {
    name: String,
    depth: usize,
}

pub struct Compiler {
    chunk: Chunk,
    constant_pool: FxHashMap<Value, usize>,
    scope_depth: usize,
    locals: Vec<Local>,
}
impl Compiler {
    pub fn compile(stmts: &[Stmt], vm: &VM) -> Chunk {
        let mut compiler = Compiler {
            chunk: Chunk::new(),
            constant_pool: HashMap::default(),
            scope_depth: 0,
            locals: Vec::new(),
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

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while !self.locals.is_empty() && self.locals[self.locals.len() - 1].depth > self.scope_depth
        {
            self.chunk
                .write(Instruction::Pop.into(), self.chunk.last_byte_line());
            self.locals.pop();
        }
        for local in &self.locals {
            debug_assert!(
                local.depth <= self.scope_depth,
                "{} {}",
                local.depth,
                self.scope_depth
            );
        }
    }

    fn add_local<S: Into<String>>(&mut self, name: S) {
        if self.locals.len() > u8::MAX as usize {
            panic!("too many locals in function") // TODO: compiler errors
        }
        self.locals.push(Local {
            name: name.into(),
            depth: self.scope_depth,
        });
    }
    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            println!("i = {}, {:?}, {:?}", i, local.name, local.depth);
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    fn emit_jump(&mut self, instruction: Instruction) -> usize {
        self.chunk
            .write(instruction.into(), self.chunk.last_byte_line());

        // Jump instructions all take a 32 bit uint

        self.chunk.write(0xFF, self.chunk.last_byte_line());
        self.chunk.write(0xFF, self.chunk.last_byte_line());
        self.chunk.write(0xFF, self.chunk.last_byte_line());
        self.chunk.write(0xFF, self.chunk.last_byte_line());

        self.chunk.code.len() - 4
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.chunk
            .write(Instruction::Jump.into(), self.chunk.last_byte_line());
        let offset = loop_start;
        if offset > u32::MAX as usize {
            panic!("Too much code to loop on.");
        }

        self.chunk
            .write(((offset >> 24) & 0xff) as u8, self.chunk.last_byte_line());
        self.chunk
            .write(((offset >> 16) & 0xff) as u8, self.chunk.last_byte_line());
        self.chunk
            .write(((offset >> 8) & 0xff) as u8, self.chunk.last_byte_line());
        self.chunk
            .write((offset & 0xff) as u8, self.chunk.last_byte_line());
    }

    fn patch_jump(&mut self, jmp_offset: usize) {
        let jump = self.chunk.code.len(); // TODO: WHY IS THIS BROKEN??!

        if jump > u32::MAX as usize {
            panic!("Too much code to jump over.");
        }

        self.chunk.code[jmp_offset] = ((jump >> 24) & 0xff) as u8;
        self.chunk.code[jmp_offset + 1] = ((jump >> 16) & 0xff) as u8;
        self.chunk.code[jmp_offset + 2] = ((jump >> 8) & 0xff) as u8;
        self.chunk.code[jmp_offset + 3] = (jump & 0xff) as u8;
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
                if self.scope_depth == 0 {
                    let constant = self.get_constant(Value::Obj(
                        vm.alloc(Obj::new(ObjType::String(AnkokuString::new(name.clone())))),
                    ));
                    write_byte!(Instruction::DefineGlobal.into());
                    write_byte!(constant as u8);
                } else {
                    for local in self.locals.iter().rev() {
                        if local.depth < self.scope_depth {
                            break;
                        }

                        if *name == local.name {
                            panic!("already variable named {:?} in this scope", name);
                        }
                    }
                    self.add_local(name);
                }
            }
            StmtType::Block(block) => {
                self.begin_scope();
                for b in block {
                    self.visit_stmt(b, vm);
                }
                self.end_scope();
            }
            StmtType::If(condition, body, else_body) => {
                self.visit_node(condition, vm);

                let jump = self.emit_jump(Instruction::JumpIfFalse);

                write_byte!(Instruction::Pop.into());

                self.visit_stmt(body, vm);

                let else_jump = self.emit_jump(Instruction::Jump);

                self.patch_jump(jump);
                write_byte!(Instruction::Pop.into());
                if let Some(else_body) = else_body {
                    self.visit_stmt(else_body, vm);
                }
                self.patch_jump(else_jump);
            }
            StmtType::While(cond, body) => {
                let loop_start = self.chunk.code.len();

                self.visit_node(cond, vm);

                let exit_jump = self.emit_jump(Instruction::JumpIfFalse);
                write_byte!(Instruction::Pop.into());
                self.visit_stmt(body, vm);
                self.emit_loop(loop_start);

                self.patch_jump(exit_jump);
                write_byte!(Instruction::Pop.into());
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
            ExprType::Bool(n) => {
                self.write_constant((*n).into());
            }
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
            ExprType::Var(s) => {
                if let Some(local) = self.resolve_local(s) {
                    write_byte!(Instruction::GetLocal.into());
                    write_byte!(local as u8);
                } else {
                    let constant = self.get_constant(Value::Obj(
                        vm.alloc(Obj::new(ObjType::String(AnkokuString::new(s.to_string())))), // intern this too
                    ));

                    write_byte!(Instruction::GetGlobal.into());
                    write_byte!(constant as u8);
                }
            }
            ExprType::Assign(name, value) => {
                self.visit_node(value, vm);

                if let Some(local) = self.resolve_local(name) {
                    write_byte!(Instruction::SetLocal.into());
                    write_byte!(local as u8);
                } else {
                    let constant = self.get_constant(Value::Obj(vm.alloc(Obj::new(
                        ObjType::String(AnkokuString::new(name.to_string())),
                    ))));

                    write_byte!(Instruction::SetGlobal.into());
                    write_byte!(constant as u8);
                }
            }
            ExprType::String(s) => {
                self.write_constant(Value::Obj(
                    vm.alloc(Obj::new(ObjType::String(AnkokuString::new(s.to_string())))), // intern this too
                ));
            }
            ExprType::And(l, r) => {
                self.visit_node(l, vm);
                let end_jump = self.emit_jump(Instruction::JumpIfFalse);

                write_byte!(Instruction::Pop.into());
                self.visit_node(r, vm);

                self.patch_jump(end_jump);
            }
            ExprType::Or(l, r) => {
                self.visit_node(l, vm);
                let else_jump = self.emit_jump(Instruction::JumpIfFalse);
                let end_jump = self.emit_jump(Instruction::Jump);

                self.patch_jump(else_jump);
                write_byte!(Instruction::Pop.into());

                self.visit_node(r, vm);
                self.patch_jump(end_jump);
            }
            ExprType::Greater(l, r) => {
                self.visit_node(l, vm);
                self.visit_node(r, vm);

                write_byte!(Instruction::Greater.into());
            }
            ExprType::Less(l, r) => {
                self.visit_node(l, vm);
                self.visit_node(r, vm);

                write_byte!(Instruction::Less.into());
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
