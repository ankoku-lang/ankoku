//! The runtime uses a different approach to errors than the parsing and compiler stuff, so it's a seperate file.

use std::backtrace::Backtrace;
#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorType,
    pub internal_bt: Backtrace,
}
#[derive(Debug)]
pub enum RuntimeErrorType {
    TypeError {
        expected: RuntimeType,
        kind: TypeErrorType,
    },
    UndefinedVariable {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeErrorType {
    GlobalNameMustBeString,
    ObjectSetMustBeObject,
    KeyMustBeString,
}
// TODO: proper type system
#[allow(dead_code)] // for now
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeType {
    Real,
    Number,
    String,
    Object,
    Null,
}
