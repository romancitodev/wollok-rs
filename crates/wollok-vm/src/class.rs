use std::collections::HashMap;

use crate::bytecode::MethodNameIdx;
use crate::dispatch::MethodRef;
use crate::heap::ClassId;

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub field_count: u32,
    /// Flat selector -> method table, already resolved from the class's
    /// linearization (`inherits`/`with`) once at class-definition time —
    /// so even an inline-cache miss is one hashmap lookup, never a walk
    /// up the hierarchy.
    vtable: HashMap<MethodNameIdx, MethodRef>,
}

#[derive(Debug, Default)]
pub struct ClassTable {
    classes: Vec<ClassDef>,
}

impl ClassTable {
    /// # Panics
    /// Panics if more than `u32::MAX` classes get defined.
    pub fn define(
        &mut self,
        name: impl Into<String>,
        field_count: u32,
        vtable: HashMap<MethodNameIdx, MethodRef>,
    ) -> ClassId {
        let idx = self.classes.len();
        self.classes.push(ClassDef {
            name: name.into(),
            field_count,
            vtable,
        });
        ClassId(u32::try_from(idx).expect("more than u32::MAX classes"))
    }

    #[must_use]
    pub fn lookup(&self, class: ClassId, selector: MethodNameIdx) -> Option<MethodRef> {
        self.classes[class.0 as usize]
            .vtable
            .get(&selector)
            .copied()
    }

    #[must_use]
    pub fn field_count(&self, class: ClassId) -> u32 {
        self.classes[class.0 as usize].field_count
    }

    #[must_use]
    pub fn name_of(&self, class: ClassId) -> &str {
        &self.classes[class.0 as usize].name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_a_defined_selector() {
        let mut vtable = HashMap::new();
        vtable.insert(MethodNameIdx(0), MethodRef(10));
        let mut classes = ClassTable::default();
        let class = classes.define("Point", 2, vtable);

        assert_eq!(classes.lookup(class, MethodNameIdx(0)), Some(MethodRef(10)));
        assert_eq!(classes.field_count(class), 2);
        assert_eq!(classes.name_of(class), "Point");
    }

    #[test]
    fn lookup_misses_for_an_undefined_selector() {
        let classes_table = {
            let mut classes = ClassTable::default();
            classes.define("Empty", 0, HashMap::new());
            classes
        };
        assert_eq!(classes_table.lookup(ClassId(0), MethodNameIdx(99)), None);
    }

    #[test]
    fn distinct_classes_have_independent_vtables() {
        let mut vtable_a = HashMap::new();
        vtable_a.insert(MethodNameIdx(0), MethodRef(1));
        let mut vtable_b = HashMap::new();
        vtable_b.insert(MethodNameIdx(0), MethodRef(2));

        let mut classes = ClassTable::default();
        let a = classes.define("A", 0, vtable_a);
        let b = classes.define("B", 0, vtable_b);

        assert_eq!(classes.lookup(a, MethodNameIdx(0)), Some(MethodRef(1)));
        assert_eq!(classes.lookup(b, MethodNameIdx(0)), Some(MethodRef(2)));
    }
}
