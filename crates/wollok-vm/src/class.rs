// use std::collections::HashMap;

use hashbrown::HashMap;

use crate::bytecode::{MethodNameIdx, MethodSlot};
use crate::dispatch::MethodRef;
use crate::heap::ClassId;

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub field_count: u32,
    /// Runs field initializers on a freshly allocated instance. `None`
    /// when the class has no fields to initialize.
    ctor: Option<MethodRef>,
    vtable: VTable,
}

#[derive(Debug, Clone)]
pub struct VTable {
    slots: HashMap<MethodNameIdx, MethodSlot>,
    entries: Box<[MethodRef]>,
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
        ctor: Option<MethodRef>,
        methods: HashMap<MethodNameIdx, MethodRef>,
    ) -> ClassId {
        let idx = self.classes.len();

        let mut slots = HashMap::with_capacity(methods.len());
        let mut entries = Vec::with_capacity(methods.len());

        for (slot, (selector, method)) in methods.into_iter().enumerate() {
            let slot =
                MethodSlot(u32::try_from(slot).expect("more than u32::MAX methods in a class"));

            slots.insert(selector, slot);
            entries.push(method);
        }

        let vtable = VTable {
            slots,
            entries: entries.into_boxed_slice(),
        };

        self.classes.push(ClassDef {
            name: name.into(),
            field_count,
            ctor,
            vtable,
        });
        ClassId(u32::try_from(idx).expect("more than u32::MAX classes"))
    }

    #[must_use]
    pub fn ctor(&self, ClassId(id): ClassId) -> Option<MethodRef> {
        self.classes[id as usize].ctor
    }

    #[must_use]
    pub fn lookup(&self, ClassId(id): ClassId, selector: MethodNameIdx) -> Option<MethodRef> {
        let vtable = &self.classes[id as usize].vtable;

        let slot = vtable.slots.get(&selector)?;
        vtable.entries.get(slot.0 as usize).copied()
    }

    #[must_use]
    pub fn field_count(&self, ClassId(id): ClassId) -> u32 {
        self.classes[id as usize].field_count
    }

    #[must_use]
    pub fn name_of(&self, ClassId(id): ClassId) -> &str {
        &self.classes[id as usize].name
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
        let class = classes.define("Point", 2, None, vtable);

        assert_eq!(classes.lookup(class, MethodNameIdx(0)), Some(MethodRef(10)));
        assert_eq!(classes.field_count(class), 2);
        assert_eq!(classes.name_of(class), "Point");
    }

    #[test]
    fn lookup_misses_for_an_undefined_selector() {
        let classes_table = {
            let mut classes = ClassTable::default();
            classes.define("Empty", 0, None, HashMap::new());
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
        let a = classes.define("A", 0, None, vtable_a);
        let b = classes.define("B", 0, None, vtable_b);

        assert_eq!(classes.lookup(a, MethodNameIdx(0)), Some(MethodRef(1)));
        assert_eq!(classes.lookup(b, MethodNameIdx(0)), Some(MethodRef(2)));
    }
}
