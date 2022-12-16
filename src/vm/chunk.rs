use crate::vm::instruction::Instruction;

use super::value::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::with_capacity(8),
            constants: Vec::with_capacity(8),
            lines: Vec::new(),
        }
    }
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub(crate) fn last_byte_line(&self) -> usize {
        if !self.lines.is_empty() {
            self.lines[self.lines.len() - 1]
        } else {
            1
        }
    }

    pub fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        assert!(offset < self.code.len());
        print!("{:04} ", offset);

        let instruction: Instruction = self.code[offset].try_into().expect("Invalid instruction");
        match instruction {
            Instruction::Return
            | Instruction::Negate
            | Instruction::Add
            | Instruction::Sub
            | Instruction::Mul
            | Instruction::Div
            | Instruction::Less
            | Instruction::Greater
            | Instruction::Not
            | Instruction::Pop
            | Instruction::Print
            | Instruction::NewObject
            | Instruction::ObjectSet => {
                println!("{:?}", instruction);
                offset + 1
            }
            Instruction::GetLocal | Instruction::SetLocal => {
                println!("{:?} {}", instruction, self.code[offset + 1],);
                offset + 2
            }
            // one 32-bit operand
            Instruction::JumpIfFalse | Instruction::Jump => {
                println!(
                    "{:?} {}",
                    instruction,
                    ((self.code[offset + 1] as usize) << 24)
                        | ((self.code[offset + 2] as usize) << 16)
                        | ((self.code[offset + 3] as usize) << 8)
                        | (self.code[offset + 4] as usize)
                );
                offset + 5
            }
            Instruction::DefineGlobal
            | Instruction::GetGlobal
            | Instruction::SetGlobal
            | Instruction::Constant => {
                println!(
                    "{:?} {} {:?}",
                    instruction,
                    self.code[offset + 1],
                    self.constants[self.code[offset + 1] as usize]
                );
                offset + 2
            }
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::instruction::Instruction;

    use super::Chunk;

    #[test]
    fn basic() {
        let mut chunk = Chunk::new();
        chunk.write(Instruction::Return.into(), 1);

        let constant = chunk.add_constant(1.0.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant as u8, 1);

        chunk.disassemble("test");

        assert_eq!(chunk.code, vec![1, 2, constant as u8]);
    }
}
