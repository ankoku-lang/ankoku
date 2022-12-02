#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::with_capacity(8),
        }
    }
    pub fn write(&mut self, byte: u8) {
        self.code.push(byte);
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
        chunk.write(Instruction::Return.into());
        println!("{:?}", chunk);
        assert!(false)
    }
}
