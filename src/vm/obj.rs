use std::hash::Hasher;

use crate::util::fxhash::FxHasher;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Obj {
    pub kind: ObjType,
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

#[cfg(test)]
mod tests {
    use super::AnkokuString;

    #[test]
    fn hashes() {
        let s = "Hello world";
        let s = AnkokuString::new(s.to_string());

        let b = "Hello world";
        let b = AnkokuString::new(b.to_string());
        assert_eq!(s.hash, b.hash);
        assert_eq!(s, b);
    }

    #[test]
    fn naughty_strings() {
        for ns in naughty_strings::BLNS {
            println!("{}", ns);
        }
    }
}
