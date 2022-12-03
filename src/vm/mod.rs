use self::chunk::Chunk;

pub mod chunk;
pub mod instruction;

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
}

impl VM<'_> {
    pub fn interpret(chunk: &Chunk) -> InterpretResult {
        let mut vm = VM { chunk, ip: 0 };

        vm.run()
    }

    pub fn run(&mut self) -> InterpretResult {
        let mut instr: *const u8 = unsafe {
            (self.chunk.code.get(self.ip).expect("IP out of bounds") as *const u8).sub(1)
        };

        let mut read_byte = || unsafe {
            instr = instr.add(1);
            *instr
        };

        loop {
            let instruction = read_byte();
            match instruction {
                // Return
                1 => {
                    return InterpretResult::Ok;
                }

                _ => unimplemented!(),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

#[cfg(test)]
mod tests {
    use crate::vm::{chunk::Chunk, instruction::Instruction, InterpretResult, VM};

    #[test]
    fn returns() {
        let mut chunk = Chunk::new();
        chunk.write(Instruction::Return.into());
        assert_eq!(VM::interpret(&chunk), InterpretResult::Ok);
    }
}
