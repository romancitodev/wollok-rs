use crate::bytecode::{Instr, MethodNameIdx};
use crate::dispatch::MethodRef;

#[derive(Debug, Clone)]
pub struct Method {
    pub selector: MethodNameIdx,
    pub arity: u8,
    /// Locals beyond the params (which occupy slots `0..arity`).
    pub extra_locals: u32,
    pub code: Vec<Instr>,
}

/// Compiled method bodies, indexed by `MethodRef`.
#[derive(Debug, Default)]
pub struct MethodTable {
    methods: Vec<Method>,
}

impl MethodTable {
    /// # Panics
    /// Panics if more than `u32::MAX` methods get compiled.
    pub fn define(&mut self, method: Method) -> MethodRef {
        let idx = self.methods.len();
        self.methods.push(method);
        MethodRef(u32::try_from(idx).expect("more than u32::MAX methods"))
    }

    #[must_use]
    pub fn get(&self, method_ref: MethodRef) -> &Method {
        &self.methods[method_ref.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn method(selector: u32) -> Method {
        Method {
            selector: MethodNameIdx(selector),
            arity: 0,
            extra_locals: 0,
            code: vec![Instr::PushNull, Instr::Return],
        }
    }

    #[test]
    fn defined_methods_are_retrievable_by_their_ref() {
        let mut table = MethodTable::default();
        let m_ref = table.define(method(1));
        assert_eq!(table.get(m_ref).selector, MethodNameIdx(1));
    }

    #[test]
    fn distinct_definitions_get_distinct_refs() {
        let mut table = MethodTable::default();
        let a = table.define(method(1));
        let b = table.define(method(2));
        assert_ne!(a, b);
        assert_eq!(table.get(a).selector, MethodNameIdx(1));
        assert_eq!(table.get(b).selector, MethodNameIdx(2));
    }
}
