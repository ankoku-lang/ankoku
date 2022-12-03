use self::{chunk::Chunk, value::Value};

pub mod chunk;
pub mod instruction;
pub mod obj;
pub mod value;

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl VM<'_> {
    pub fn interpret(chunk: &Chunk) -> (InterpretResult, VM) {
        let mut vm = VM {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(128),
        };

        (vm.run(), vm)
    }

    pub(crate) fn stack_push(&mut self, value: Value) {
        assert!(self.stack.len() < 1024, "stack overflow");
        self.stack.push(value);
    }

    pub(crate) fn stack_pop(&mut self) -> Value {
        self.stack.pop().expect("nothing to pop")
    }

    pub fn run(&mut self) -> InterpretResult {
        #[cfg(feature = "debug-mode")]
        println!("== VM ==");
        macro_rules! read_byte {
            () => {{
                self.ip += 1;
                self.chunk.code[self.ip - 1]
            }};
        }
        macro_rules! read_constant {
            () => {
                self.chunk.constants[read_byte!() as usize].clone()
            };
        }

        macro_rules! binop {
			($t:tt) => {
				let b = self.stack_pop();
				let a = self.stack_pop();
				self.stack_push(a $t b);
			};
		}

        macro_rules! unop {
			($t:tt) => {
				let a = self.stack_pop();
				self.stack_push($t a);
			};
		}

        loop {
            #[cfg(feature = "debug-mode")]
            {
                print!("STACK:    ");
                for value in &self.stack {
                    print!("[ {:?} ]", value);
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }
            let instruction = read_byte!();

            match instruction {
                // Return
                1 => {
                    return InterpretResult::Ok(self.stack_pop());
                }
                // Constant
                2 => {
                    let constant = read_constant!();
                    println!("{:?}", constant);
                    self.stack_push(constant);
                }
                // Negate
                3 => {
                    let v = self.stack_pop();
                    self.stack_push(-v);
                }
                // Add
                4 => {
                    binop!(+);
                }
                // Sub
                5 => {
                    binop!(-);
                }
                // Mul
                6 => {
                    binop!(*);
                }
                // Div
                7 => {
                    binop!(/);
                }
                // Not
                8 => {
                    unop!(!);
                }
                _ => unimplemented!(),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InterpretResult {
    Ok(Value),
    CompileError,
    RuntimeError,
}

#[cfg(test)]
mod tests {
    use crate::vm::{chunk::Chunk, instruction::Instruction, value::Value, InterpretResult, VM};

    #[test]
    fn returns() {
        let mut chunk = Chunk::new();

        let constant = chunk.add_constant(1.2.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant, 1);

        let constant = chunk.add_constant(3.4.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant, 1);

        chunk.write(Instruction::Add.into(), 1);

        let constant = chunk.add_constant(5.6.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant, 1);

        chunk.write(Instruction::Div.into(), 1);

        chunk.write(Instruction::Negate.into(), 1);

        chunk.write(Instruction::Return.into(), 1);

        chunk.disassemble("test");
        // if this assertion fails, try using approx: https://docs.rs/approx
        assert_eq!(
            VM::interpret(&chunk).0,
            InterpretResult::Ok(Value::Real(-((1.2 + 3.4) / 5.6)))
        );
    }
}
