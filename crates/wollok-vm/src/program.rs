use crate::class::ClassTable;
use crate::method::MethodTable;
use crate::selector::SelectorTable;
use crate::value::Value;

/// Everything compiled once, ahead of execution, and never mutated while
/// running: method bodies, class vtables, the constant pool, and interned
/// selectors. Kept separate from `Vm` (heap + inline caches, which DO
/// change while running) so the interpreter loop can hold a method
/// borrowed from here across recursive calls without fighting the borrow
/// checker over `&mut self` on a single do-everything struct.
#[derive(Debug, Default)]
pub struct Program {
    pub methods: MethodTable,
    pub classes: ClassTable,
    pub selectors: SelectorTable,
    pub consts: Vec<Value>,
}
