use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
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
}
