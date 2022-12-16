#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Instruction {
    Return = 1,
    Constant = 2,
    Negate = 3,
    Add = 4,
    Sub = 5,
    Mul = 6,
    Div = 7,
    Not = 8,
    Pop = 9,
    NewObject = 10, // TODO: this should have a type when classes implemented
    ObjectSet = 11,
    DefineGlobal = 12,
    GetGlobal = 13,
    SetGlobal = 14,
    GetLocal = 15,
    SetLocal = 16,
    JumpIfFalse = 17,
    Jump = 18,
    Greater = 19,
    Less = 20,
    Print = 100, // FIXME: TEMP, will be removed when functions work
}

impl From<u8> for Instruction {
    fn from(v: u8) -> Self {
        use Instruction::*;

        match v {
            1 => Return,
            2 => Constant,
            3 => Negate,
            4 => Add,
            5 => Sub,
            6 => Mul,
            7 => Div,
            8 => Not,
            9 => Pop,
            11 => ObjectSet,
            10 => NewObject,
            12 => DefineGlobal,
            13 => GetGlobal,
            14 => SetGlobal,
            15 => GetLocal,
            16 => SetLocal,
            17 => JumpIfFalse,
            18 => Jump,
            19 => Greater,
            20 => Less,
            100 => Print,
            _ => panic!("not an instruction: {:?}", v),
        }
    }
}
impl From<Instruction> for u8 {
    fn from(v: Instruction) -> Self {
        v as u8
    }
}
