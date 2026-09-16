use std::collections::HashMap;

use crate::bytecode::MethodNameIdx;

/// Wollok allows overloading by arity (`do()` and `do(a)` are different
/// methods), so a selector is name+arity, not just the name.
type SelectorKey = (String, u8);

/// Interns (name, arity) pairs once so every `Send` only ever compares a
/// `u32` (`MethodNameIdx`), never a string — string comparison on every
/// message send would undercut the whole point of inline caching.
#[derive(Debug, Default)]
pub struct SelectorTable {
    ids: HashMap<SelectorKey, MethodNameIdx>,
    entries: Vec<SelectorKey>,
}

impl SelectorTable {
    /// # Panics
    /// Panics if more than `u32::MAX` distinct selectors get interned.
    pub fn intern(&mut self, name: &str, arity: u8) -> MethodNameIdx {
        let key = (name.to_owned(), arity);
        if let Some(&id) = self.ids.get(&key) {
            return id;
        }
        let id =
            MethodNameIdx(u32::try_from(self.entries.len()).expect("more than u32::MAX selectors"));
        self.entries.push(key.clone());
        self.ids.insert(key, id);
        id
    }

    #[must_use]
    pub fn name_of(&self, id: MethodNameIdx) -> &str {
        &self.entries[id.0 as usize].0
    }

    #[must_use]
    pub fn arity_of(&self, id: MethodNameIdx) -> u8 {
        self.entries[id.0 as usize].1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interning_the_same_name_and_arity_twice_returns_the_same_id() {
        let mut table = SelectorTable::default();
        let a = table.intern("do", 0);
        let b = table.intern("do", 0);
        assert_eq!(a, b);
    }

    #[test]
    fn same_name_different_arity_are_different_selectors() {
        // do() and do(a) are different methods in Wollok.
        let mut table = SelectorTable::default();
        let arity0 = table.intern("do", 0);
        let arity1 = table.intern("do", 1);
        assert_ne!(arity0, arity1);
    }

    #[test]
    fn name_and_arity_round_trip() {
        let mut table = SelectorTable::default();
        let id = table.intern("area", 2);
        assert_eq!(table.name_of(id), "area");
        assert_eq!(table.arity_of(id), 2);
    }

    #[test]
    fn distinct_names_get_distinct_ids() {
        let mut table = SelectorTable::default();
        let a = table.intern("area", 0);
        let b = table.intern("perimetro", 0);
        assert_ne!(a, b);
    }
}
