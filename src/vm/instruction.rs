use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Instruction {
    Return = 1,
}
