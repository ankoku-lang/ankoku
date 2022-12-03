use std::{hash::Hasher, ptr::NonNull};

use crate::util::fxhash::FxHasher;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Obj {
    pub kind: ObjType,
    pub(crate) next: Option<NonNull<Obj>>,
}

impl Obj {
    pub fn new(kind: ObjType) -> Self {
        Self { kind, next: None }
    }
}

impl From<AnkokuString> for Obj {
    fn from(v: AnkokuString) -> Self {
        Obj::new(ObjType::String(v))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ObjType {
    String(AnkokuString),
}

#[derive(Clone, Debug)]
pub struct AnkokuString {
    inner: String,
    hash: usize,
}

impl AnkokuString {
    pub fn new(str: String) -> Self {
        AnkokuString {
            hash: AnkokuString::hash(str.as_bytes()),
            inner: str,
        }
    }
    fn hash(bytes: &[u8]) -> usize {
        let mut f = FxHasher::default();

        f.write(bytes);

        f.finish() as usize
    }

    pub fn concat(&self, other: &str) -> AnkokuString {
        let mut s = self.inner.clone();
        s.push_str(other);
        AnkokuString::new(s)
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn into_inner(self) -> String {
        self.inner
    }
}
impl PartialEq for AnkokuString {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}
impl Eq for AnkokuString {}
