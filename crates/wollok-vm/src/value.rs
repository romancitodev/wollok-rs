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
}
