//! Defaults every heap object falls back to when its own class doesn't
//! define the selector itself — registered under `PrimitiveKind::Object`,
//! the closest thing this VM has today to a root `Object` class. **Not**
//! real inheritance: there's still no `inherits`/`with`/`super`
//! linearization (see `docs/backlog.md` item 4) — a class that *does*
//! define `toString` always wins, this is only consulted once `vm.rs`'s
//! `Send` (or `Vm::send`) finds nothing in the class's own vtable.

use wollok_vm::native::{NativeTable, PrimitiveKind};
use wollok_vm::program::Program;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

pub fn install(table: &mut NativeTable) {
    table.register(PrimitiveKind::Object, "toString", 0, to_string);
}

fn to_string(vm: &mut Vm, program: &Program, this: Value, _args: &[Value]) -> Value {
    let obj = this.as_object().expect("registered only for Value::Object receivers");
    let class_name = program.classes.name_of(vm.heap.class_of(obj));
    let text = format!("a {class_name}");
    Value::from(vm.strings.intern(&text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wollok_vm::heap::ClassId;

    #[test]
    fn default_to_string_names_the_class() {
        let mut table = NativeTable::default();
        install(&mut table);

        let mut vm = Vm::new();
        let mut program = Program::default();
        let class = program
            .classes
            .define("Bird", 0, None, Default::default());
        assert_eq!(class, ClassId(0));

        let obj = Value::from(vm.heap.alloc(class, vec![]));
        let f = table.lookup(PrimitiveKind::Object, "toString", 0).unwrap();
        let result = f(&mut vm, &program, obj, &[]);

        let idx = result.as_str_idx().unwrap();
        assert_eq!(vm.strings.get(idx), "a Bird");
    }
}
