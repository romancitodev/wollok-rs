use std::collections::HashMap;

use crate::bytecode::StrIdx;

/// Runtime string storage, interned by content (two `"pepita"` literals, or
/// a literal and a `toString`/`+` result with the same bytes, share a
/// `StrIdx`). Lives on `Vm`, not `Program`: unlike every other table a
/// `Program` owns, this one grows at runtime — string-producing native
/// methods (`toString`, `+` on strings) need somewhere to put their result,
/// and `Program` is immutable once compiled. See "Principio: los
/// primitivos nunca son objetos de heap" in `docs/vm-design.md`.
#[derive(Debug, Default)]
pub struct StringTable {
    ids: HashMap<String, StrIdx>,
    entries: Vec<String>,
}

impl StringTable {
    /// # Panics
    /// Panics if more than `u32::MAX` distinct strings ever exist at once.
    pub fn intern(&mut self, s: &str) -> StrIdx {
        if let Some(&id) = self.ids.get(s) {
            return id;
        }
        let id = StrIdx(u32::try_from(self.entries.len()).expect("more than u32::MAX strings"));
        self.entries.push(s.to_owned());
        self.ids.insert(s.to_owned(), id);
        id
    }

    #[must_use]
    pub fn get(&self, idx: StrIdx) -> &str {
        &self.entries[idx.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interning_the_same_content_twice_returns_the_same_id() {
        let mut table = StringTable::default();
        let a = table.intern("pepita");
        let b = table.intern("pepita");
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_content_gets_distinct_ids() {
        let mut table = StringTable::default();
        let a = table.intern("pepita");
        let b = table.intern("alpiste");
        assert_ne!(a, b);
    }

    #[test]
    fn get_round_trips_the_content() {
        let mut table = StringTable::default();
        let id = table.intern("hola, soy pepita!");
        assert_eq!(table.get(id), "hola, soy pepita!");
    }
}
