use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Null,
    Real(f64),
}

impl Value {
    /// Try to convert this into a real (f64).
    pub fn coerce_real(&self) -> f64 {
        match self {
            Value::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Real(v) => *v,
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }

    pub fn coerce_bool(&self) -> bool {
        match self {
            Value::Bool(v) => *v,
            Value::Real(v) => *v != 0.0,
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", b),
            Self::Null => write!(f, "null"),
            Self::Real(n) => write!(f, "{}", n),
        }
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Real(v)
    }
}

impl Add<Value> for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Self::Output {
        match self {
            Value::Real(l) => (l + rhs.coerce_real()).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
}
impl Sub<Value> for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Self::Output {
        match self {
            Value::Real(l) => (l - rhs.coerce_real()).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
}

impl Mul<Value> for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Self::Output {
        match self {
            Value::Real(l) => (l * rhs.coerce_real()).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
}

impl Div<Value> for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> Self::Output {
        match self {
            Value::Real(l) => (l / rhs.coerce_real()).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
}
impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::Real(l) => (-l).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
}
