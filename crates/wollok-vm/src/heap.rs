use crate::value::Value;

/// Index into the heap arena, not a pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjRef(u32);

impl core::fmt::Display for ObjRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<object {}>", self.0)
    }
}

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

    /// Only way to mutate a field — routes through the write barrier hook.
    pub fn write_field(&mut self, obj: ObjRef, field: FieldIdx, value: Value) {
        self.write_barrier(obj, value);
        self.objects[obj.0 as usize].fields[field.0 as usize] = value;
    }

    #[allow(clippy::unused_self)]
    fn write_barrier(&mut self, _obj: ObjRef, _value: Value) {
        // no-op until there's a generational GC
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

    #[test]
    fn zero_field_object_allocates_fine() {
        let mut heap = Heap::new();
        let obj = heap.alloc(ClassId(5), vec![]);
        assert_eq!(heap.class_of(obj), ClassId(5));
    }

    #[test]
    fn writing_one_field_never_touches_its_neighbors() {
        let mut heap = Heap::new();
        let obj = heap.alloc(
            ClassId(1),
            vec![Value::from(1i64), Value::from(2i64), Value::from(3i64)],
        );

        heap.write_field(obj, FieldIdx(1), Value::from(999i64));

        assert_eq!(heap.read_field(obj, FieldIdx(0)).as_int(), Some(1));
        assert_eq!(heap.read_field(obj, FieldIdx(1)).as_int(), Some(999));
        assert_eq!(heap.read_field(obj, FieldIdx(2)).as_int(), Some(3));
    }

    #[test]
    fn writing_one_object_never_touches_another_objects_fields() {
        let mut heap = Heap::new();
        let a = heap.alloc(ClassId(1), vec![Value::from(1i64)]);
        let b = heap.alloc(ClassId(1), vec![Value::from(2i64)]);

        heap.write_field(a, FieldIdx(0), Value::from(-1i64));

        assert_eq!(heap.read_field(a, FieldIdx(0)).as_int(), Some(-1));
        assert_eq!(
            heap.read_field(b, FieldIdx(0)).as_int(),
            Some(2),
            "writing object a must not leak into object b"
        );
    }

    #[test]
    fn class_of_stays_correct_across_interleaved_allocations_of_different_classes() {
        let mut heap = Heap::new();
        let a = heap.alloc(ClassId(1), vec![]);
        let b = heap.alloc(ClassId(2), vec![]);
        let c = heap.alloc(ClassId(1), vec![]);

        assert_eq!(heap.class_of(a), ClassId(1));
        assert_eq!(heap.class_of(b), ClassId(2));
        assert_eq!(heap.class_of(c), ClassId(1));
    }

    #[test]
    fn earlier_refs_stay_valid_after_more_allocations() {
        // ObjRef is an index into the arena, not a pointer, so it must
        // keep pointing at the same object even after the backing Vec
        // grows (and potentially reallocates) further.
        let mut heap = Heap::new();
        let first = heap.alloc(ClassId(1), vec![Value::from(111i64)]);

        for i in 0..64 {
            heap.alloc(ClassId(2), vec![Value::from(i)]);
        }

        assert_eq!(heap.class_of(first), ClassId(1));
        assert_eq!(heap.read_field(first, FieldIdx(0)).as_int(), Some(111));
    }

    #[test]
    fn obj_ref_is_stable_as_a_hash_map_key() {
        use std::collections::HashMap;

        let mut heap = Heap::new();
        let a = heap.alloc(ClassId(1), vec![]);
        let b = heap.alloc(ClassId(1), vec![]);

        let mut seen = HashMap::new();
        seen.insert(a, "a");
        seen.insert(b, "b");

        assert_eq!(seen.get(&a), Some(&"a"));
        assert_eq!(seen.get(&b), Some(&"b"));
    }
}
