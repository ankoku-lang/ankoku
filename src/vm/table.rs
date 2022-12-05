use std::fmt::Debug;

use super::{obj::AnkokuString, value::Value};

#[derive(Clone, PartialEq)]
pub struct HashTable {
    entries: Vec<Entry>,
    count: usize,
}
const TABLE_MAX_LOAD: f32 = 0.75;
impl Debug for HashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashTable {{")?;
        for (k, v) in self.entries() {
            write!(f, " {} = {:?}", k.as_str(), v)?;
        }
        Ok(())
    }
}
impl HashTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            count: 0,
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.entries.iter().filter_map(|v| {
            if v.key.is_some() {
                Some(&v.value)
            } else {
                None
            }
        })
    }

    pub fn entries(&self) -> impl Iterator<Item = (&AnkokuString, &Value)> {
        self.entries
            .iter()
            .filter_map(|v| v.key.as_ref().map(|k| (k, &v.value)))
    }
    pub fn len(&self) -> usize {
        self.count
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn find_entry(entries: &[Entry], key: usize) -> usize {
        let mut index = key % entries.len();
        let mut entry: &Entry;
        let mut tombstone: Option<usize> = None;
        loop {
            entry = &entries[index];
            if entry.key.is_none() {
                if entry.value == Value::Null {
                    // Empty entry.
                    return if let Some(t) = tombstone { t } else { index };
                } else {
                    // We found a tombstone.
                    if tombstone.is_none() {
                        tombstone = Some(index);
                    }
                }
            } else if let Some(k) = &entry.key {
                if k.hash() == key {
                    return index;
                }
            }
            index = (index + 1) % entries.len();
        }
    }
    pub fn get(&self, key: &AnkokuString) -> Option<&Value> {
        if self.count == 0 {
            None
        } else {
            let entry = &self.entries[HashTable::find_entry(&self.entries, key.hash())];
            entry.key.as_ref()?;
            Some(&entry.value)
        }
    }
    pub fn set(&mut self, key: AnkokuString, value: Value) -> bool {
        if (self.count + 1) as f32 > self.entries.len() as f32 * TABLE_MAX_LOAD {
            let capacity = if self.entries.len() < 8 {
                8
            } else {
                self.entries.len() * 2
            };

            let mut entries = Vec::with_capacity(capacity);

            for _ in 0..capacity {
                entries.push(Entry {
                    key: None,
                    value: Value::Null,
                });
            }

            for i in 0..self.entries.len() {
                let entry = &self.entries[i];
                if entry.key.is_none() {
                    continue;
                }
                let dest = HashTable::find_entry(&entries, entry.key.as_ref().unwrap().hash());

                entries[dest].key = entry.key.clone();
                entries[dest].value = entry.value.clone();
            }

            self.entries = entries;
        }
        let entry = HashTable::find_entry(&self.entries, key.hash());
        let is_new_key = self.entries[entry].key.is_none();
        if is_new_key {
            self.count += 1;
        }
        let entry = &mut self.entries[entry];
        entry.key = Some(key);
        entry.value = value;
        is_new_key
    }

    pub fn add_all(&mut self, from: &HashTable) {
        for i in 0..from.entries.len() {
            let entry = &from.entries[i];
            if let Some(k) = entry.key.clone() {
                self.set(k, entry.value.clone());
            }
        }
    }

    pub fn delete(&mut self, key: usize) -> bool {
        if self.count == 0 {
            false
        } else {
            let entry = HashTable::find_entry(&self.entries, key);
            if self.entries[entry].key.is_none() {
                return false;
            }
            self.entries[entry].key = None;
            self.entries[entry].value = Value::Bool(true);
            true
        }
    }
}

impl Default for HashTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Entry {
    key: Option<AnkokuString>,
    value: Value,
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::vm::{obj::AnkokuString, value::Value};

    use super::HashTable;

    #[test]
    fn basic() {
        let thingy = Value::Bool(true);
        let key = AnkokuString::new("hello_world".into());

        let mut table = HashTable::new();
        table.set(key.clone(), thingy.clone());

        assert_eq!(table.get(&key), Some(&thingy));
    }

    #[test]
    fn stress_test() {
        let start = Instant::now();
        let mut table = HashTable::new();

        for i in 0..10000 {
            let thingy = Value::Bool(true);
            let key = AnkokuString::new(format!("i{}", i));
            table.set(key, thingy.clone());
        }
        println!("stress test inserts took {:?}", start.elapsed());
    }
}
