use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use super::{obj::ObjType, GcRef, VM};

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Null,
    Real(f64),
    Obj(GcRef),
}
impl Eq for Value {}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Real(l0), Self::Real(r0)) => l0 == r0,
            (Self::Obj(l0), Self::Obj(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let tag = core::mem::discriminant(self);
        Hash::hash(&tag, state);
        match self {
            Value::Bool(b) => Hash::hash(b, state),
            Value::Real(f) => Hash::hash(&f.to_bits(), state),
            Value::Obj(r) => Hash::hash(r, state),
            _ => {}
        }
    }
}

impl Value {
    /// Try to convert this into a real (f64).
    pub fn coerce_real(self) -> f64 {
        match self {
            Value::Bool(v) => {
                if v {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Real(v) => v,
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }

    pub fn coerce_bool(self) -> bool {
        match self {
            Value::Bool(v) => v,
            Value::Real(v) => v != 0.0,
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }

    pub fn coerce_str(self) -> String {
        match self {
            Value::Bool(v) => {
                if v {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Value::Real(v) => v.to_string(),
            Value::Obj(o) => match &o.inner().kind {
                ObjType::String(v) => v.clone().into_inner(),
                ObjType::Object(_) => todo!("typeerrors"),
            },
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }

    pub fn add(self, rhs: Value, gc: &VM) -> Value {
        match self {
            Value::Real(l) => (l + rhs.coerce_real()).into(),
            Value::Obj(gcref) => match &gcref.kind {
                ObjType::String(self_string) => {
                    Value::Obj(gc.alloc(self_string.concat(&rhs.coerce_str()).into()))
                }
                ObjType::Object(_) => todo!(),
            },
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }

    pub fn sub(self, rhs: Value, _gc: &VM) -> Value {
        match self {
            Value::Real(l) => (l - rhs.coerce_real()).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }

    pub fn mul(self, rhs: Value, _gc: &VM) -> Value {
        match self {
            Value::Real(l) => (l * rhs.coerce_real()).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
    pub fn div(self, rhs: Value, _gc: &VM) -> Value {
        match self {
            Value::Real(l) => (l / rhs.coerce_real()).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
    pub fn neg(self, _gc: &VM) -> Value {
        match self {
            Value::Real(l) => (-l).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }
    pub fn not(self, _gc: &VM) -> Value {
        match self {
            Value::Bool(l) => (!l).into(),
            _ => todo!("implement proper type errors here instead of panics"),
        }
    }

    pub fn falsey(&self) -> bool {
        match self {
            Value::Bool(b) => !*b,
            Value::Null => true,
            Value::Real(b) => *b != 0.0,
            Value::Obj(o) => true,
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value(")?;
        match self {
            Self::Bool(b) => write!(f, "Bool({})", b),
            Self::Null => write!(f, "null"),
            Self::Real(n) => write!(f, "Real({})", n),
            Self::Obj(a) => write!(f, "Obj({:?})", a.inner()),
        }?;
        write!(f, ")")
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Real(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}
