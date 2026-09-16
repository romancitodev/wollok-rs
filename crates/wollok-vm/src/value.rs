use crate::heap::ObjRef;

/// A Wollok runtime value. Never match on this directly outside this
/// module — go through the accessors below, so that if the internal
/// representation ever changes (packed/boxed), the change stays here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Object(ObjRef),
}

impl Value {
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(n) => Some(*n),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_object(&self) -> Option<ObjRef> {
        match self {
            Value::Object(r) => Some(*r),
            _ => None,
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Float(n)
    }
}

impl From<ObjRef> for Value {
    fn from(r: ObjRef) -> Self {
        Value::Object(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_match_the_constructed_variant() {
        assert_eq!(Value::from(42i64).as_int(), Some(42));
        assert_eq!(Value::from(3.5f64).as_float(), Some(3.5));
        assert_eq!(Value::from(true).as_bool(), Some(true));
        assert!(Value::Null.is_null());
    }

    #[test]
    fn accessors_return_none_for_the_wrong_variant() {
        let v = Value::from(42i64);
        assert_eq!(v.as_bool(), None);
        assert_eq!(v.as_float(), None);
        assert_eq!(v.as_object(), None);
        assert!(!v.is_null());
    }

    #[test]
    fn every_variant_only_answers_its_own_accessor() {
        let bool_v = Value::from(false);
        assert_eq!(bool_v.as_bool(), Some(false));
        assert_eq!(bool_v.as_int(), None);
        assert_eq!(bool_v.as_float(), None);
        assert_eq!(bool_v.as_object(), None);

        let float_v = Value::from(1.0f64);
        assert_eq!(float_v.as_float(), Some(1.0));
        assert_eq!(float_v.as_int(), None);
        assert_eq!(float_v.as_bool(), None);
        assert_eq!(float_v.as_object(), None);

        assert_eq!(Value::Null.as_bool(), None);
        assert_eq!(Value::Null.as_int(), None);
        assert_eq!(Value::Null.as_float(), None);
        assert_eq!(Value::Null.as_object(), None);
    }

    #[test]
    fn int_and_float_are_distinct_variants_even_when_numerically_equal() {
        // 1 (Int) and 1.0 (Float) must NOT compare equal: Wollok
        // distinguishes them, so the VM can't let Rust's numeric
        // coercion blur that at the Value level.
        assert_ne!(Value::from(1i64), Value::from(1.0f64));
    }

    #[test]
    fn same_variant_same_payload_is_equal() {
        assert_eq!(Value::from(7i64), Value::from(7i64));
        assert_eq!(Value::from(2.5f64), Value::from(2.5f64));
        assert_eq!(Value::from(true), Value::from(true));
        assert_eq!(Value::Null, Value::Null);
        assert_ne!(Value::from(7i64), Value::from(8i64));
        assert_ne!(Value::from(true), Value::from(false));
    }

    #[test]
    fn negative_and_zero_ints_round_trip() {
        assert_eq!(Value::from(0i64).as_int(), Some(0));
        assert_eq!(Value::from(-1i64).as_int(), Some(-1));
        assert_eq!(Value::from(i64::MIN).as_int(), Some(i64::MIN));
        assert_eq!(Value::from(i64::MAX).as_int(), Some(i64::MAX));
    }

    #[test]
    fn object_variant_round_trips_the_exact_ref() {
        use crate::heap::{ClassId, Heap};
        let mut heap = Heap::new();
        let obj = heap.alloc(ClassId(0), vec![]);
        let v = Value::from(obj);
        assert_eq!(v.as_object(), Some(obj));
    }

    #[test]
    fn value_is_copy_and_independent_after_copying() {
        let a = Value::from(1i64);
        let b = a; // Copy, not a move
        assert_eq!(a, b);
        assert_eq!(a.as_int(), Some(1));
    }
}
