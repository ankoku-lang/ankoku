use super::{obj::AnkokuString, value::Value};

pub struct HashTable {
    entries: Vec<Entry>,
    count: usize,
}
const TABLE_MAX_LOAD: f32 = 0.75;

impl HashTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            count: 0,
        }
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
            if entry.key == 0 {
                if entry.value == Value::Null {
                    // Empty entry.
                    return if let Some(t) = tombstone { t } else { index };
                } else {
                    // We found a tombstone.
                    if tombstone == None {
                        tombstone = Some(index);
                    }
                }
            } else if entry.key == key {
                return index;
            }
            index = (index + 1) % entries.len();
        }
    }
    pub fn get(&self, key: usize) -> Option<&Value> {
        if self.count == 0 {
            None
        } else {
            let entry = &self.entries[HashTable::find_entry(&self.entries, key)];
            if entry.key == 0 {
                return None;
            }
            Some(&entry.value)
        }
    }
    pub fn set(&mut self, key: usize, value: Value) -> bool {
        if (self.count + 1) as f32 > self.entries.len() as f32 * TABLE_MAX_LOAD {
            let capacity = if self.entries.len() < 8 {
                8
            } else {
                self.entries.len() * 2
            };

            let mut entries = Vec::with_capacity(capacity);

            for _ in 0..capacity {
                entries.push(Entry {
                    key: 0,
                    value: Value::Null,
                });
            }

            for i in 0..self.entries.len() {
                let entry = &self.entries[i];
                if entry.key == 0 {
                    continue;
                }
                let dest = HashTable::find_entry(&entries, entry.key);

                entries[dest].key = entry.key;
                entries[dest].value = entry.value.clone();
            }

            self.entries = entries;
        }
        let entry = HashTable::find_entry(&self.entries, key);
        let is_new_key = self.entries[entry].key == 0;
        if is_new_key {
            self.count += 1;
        }
        let entry = &mut self.entries[entry];
        entry.key = key;
        entry.value = value;
        is_new_key
    }

    pub fn add_all(&mut self, from: &HashTable) {
        for i in 0..from.entries.len() {
            let entry = &from.entries[i];
            if entry.key != 0 {
                self.set(entry.key, entry.value.clone());
            }
        }
    }

    pub fn delete(&mut self, key: usize) -> bool {
        if self.count == 0 {
            false
        } else {
            let entry = HashTable::find_entry(&self.entries, key);
            if self.entries[entry].key == 0 {
                return false;
            }
            self.entries[entry].key = 0;
            self.entries[entry].value = Value::Bool(true);
            true
        }
    }
}

impl From<AnkokuString> for usize {
    fn from(v: AnkokuString) -> Self {
        v.hash()
    }
}

impl From<&AnkokuString> for usize {
    fn from(v: &AnkokuString) -> Self {
        v.hash()
    }
}

impl Default for HashTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Entry {
    key: usize,
    value: Value,
}

impl Entry {
    pub fn new(key: usize, value: Value) -> Self {
        Self { key, value }
    }
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
        table.set(key.hash(), thingy.clone());

        assert_eq!(table.get(key.hash()), Some(&thingy));
    }

    #[test]
    fn stress_test() {
        let start = Instant::now();
        let mut table = HashTable::new();

        for i in 0..10000 {
            let thingy = Value::Bool(true);
            let key = AnkokuString::new(format!("i{}", i));
            table.set(key.hash(), thingy.clone());
        }
        println!("stress test inserts took {:?}", start.elapsed());
    }
}
