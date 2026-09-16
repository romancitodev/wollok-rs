use crate::value::Value;

/// Index into the heap arena. Objects live in a `Vec`, not behind loose
/// pointers, so allocation is a push and a future mark-sweep GC can just
/// walk the arena instead of chasing pointers around the process heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjRef(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldIdx(pub u32);

#[derive(Debug, Clone)]
pub struct HeapObject {
    pub class: ClassId,
    pub fields: Vec<Value>,
}

#[derive(Debug, Default)]
pub struct Heap {
    objects: Vec<HeapObject>,
}

impl Heap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// # Panics
    /// Panics if the heap ever grows past `u32::MAX` live objects.
    pub fn alloc(&mut self, class: ClassId, fields: Vec<Value>) -> ObjRef {
        let idx = self.objects.len();
        self.objects.push(HeapObject { class, fields });
        ObjRef(u32::try_from(idx).expect("heap grew past u32::MAX objects"))
    }

    #[must_use]
    pub fn class_of(&self, obj: ObjRef) -> ClassId {
        self.objects[obj.0 as usize].class
    }

    #[must_use]
    pub fn read_field(&self, obj: ObjRef, field: FieldIdx) -> Value {
        self.objects[obj.0 as usize].fields[field.0 as usize]
    }

    /// The only way to mutate a field. Every heap mutation routes through
    /// here on purpose: `write_barrier` is a no-op today, but a future
    /// generational GC needs a hook at every mutation site, and adding one
    /// after the fact means re-auditing the whole codebase instead of one
    /// function.
    pub fn write_field(&mut self, obj: ObjRef, field: FieldIdx, value: Value) {
        self.write_barrier(obj, value);
        self.objects[obj.0 as usize].fields[field.0 as usize] = value;
    }

    #[allow(clippy::unused_self)]
    fn write_barrier(&mut self, _obj: ObjRef, _value: Value) {
        // no-op: mark-sweep doesn't need this. A generational GC would
        // record `_obj` in a remembered set here when `_value` points at
        // a younger-generation object.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_read_write_roundtrip() {
        let mut heap = Heap::new();
        let obj = heap.alloc(ClassId(1), vec![Value::from(1i64), Value::from(2i64)]);

        assert_eq!(heap.class_of(obj), ClassId(1));
        assert_eq!(heap.read_field(obj, FieldIdx(0)).as_int(), Some(1));

        heap.write_field(obj, FieldIdx(0), Value::from(99i64));
        assert_eq!(heap.read_field(obj, FieldIdx(0)).as_int(), Some(99));
        assert_eq!(heap.read_field(obj, FieldIdx(1)).as_int(), Some(2));
    }

    #[test]
    fn distinct_allocations_get_distinct_refs() {
        let mut heap = Heap::new();
        let a = heap.alloc(ClassId(1), vec![]);
        let b = heap.alloc(ClassId(1), vec![]);
        assert_ne!(a, b);
    }
}
