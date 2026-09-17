pub use crate::class::ClassTable;
use crate::method::MethodTable;
use crate::selector::SelectorTable;
use crate::value::Value;

/// Fixed once compiled. `Vm` holds what changes at runtime (heap, caches,
/// and — since native methods like `toString` can produce a brand new
/// string — `strings` too; see `crate::strings::StringTable`).
#[derive(Debug, Default)]
pub struct Program {
    pub methods: MethodTable,
    pub classes: ClassTable,
    pub selectors: SelectorTable,
    pub consts: Vec<Value>,
}
