use std::{fmt::Debug, hash::Hasher, ptr::NonNull};

use crate::util::fxhash::FxHasher;

use super::table::HashTable;

#[derive(Clone, PartialEq)]
pub struct Obj {
    pub kind: ObjType,
    pub(crate) next: Option<NonNull<Obj>>,
    pub(crate) marked: bool,
}

impl Obj {
    pub fn new(kind: ObjType) -> Self {
        Self {
            kind,
            next: None,
            marked: false,
        }
    }
}
impl Debug for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
impl Drop for Obj {
    fn drop(&mut self) {
        #[cfg(feature = "gc-debug-super-slow")]
        println!("{:?} dropped", self);
    }
}

impl From<AnkokuString> for Obj {
    fn from(v: AnkokuString) -> Self {
        Obj::new(ObjType::String(v))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ObjType {
    String(AnkokuString),
    Object(Object),
}

/// Not an [Obj], an [Object]. Objects are a language feature, basically a hashtable, but [Obj]s are a VM implementation of heap-allocated objects.
#[derive(Clone, PartialEq)]
pub struct Object {
    pub table: HashTable,
}
impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.table.fmt(f)
    }
}
impl Object {
    pub fn new() -> Self {
        Self {
            table: HashTable::new(),
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct AnkokuString {
    inner: String,
    hash: usize,
}

impl Debug for AnkokuString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
impl AnkokuString {
    pub fn new(str: String) -> Self {
        AnkokuString {
            hash: AnkokuString::hash_bytes(str.as_bytes()),
            inner: str,
        }
    }

    #[inline(always)]
    pub fn hash(&self) -> usize {
        self.hash
    }

    fn hash_bytes(bytes: &[u8]) -> usize {
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
