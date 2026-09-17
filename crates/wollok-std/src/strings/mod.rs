//! `Str` native methods.
//!
//! Same pattern as `numbers`/`booleans`: write a `fn(&mut Vm, &Program,
//! Value, &[Value]) -> Value` and register it in [`install`]. The
//! receiver is a `Value::Str(StrIdx)`; get its text with
//! `vm.strings.get(idx)`. Building new text (concatenation, `toString`,
//! ...)? `vm.strings.intern(&s)` gives you back a fresh (deduplicated)
//! `StrIdx` to wrap in `Value::Str`.
//!
//! Need to stringify something that *isn't* already a `Str` — an `Int`, or
//! an arbitrary user object? That's not a `Value` conversion (`Value` has
//! no access to `Vm`/`Program` on purpose — see `docs/vm-design.md`) — use
//! `vm.send(program, value, "toString", vec![])` instead, which runs the
//! receiver's own `toString` (an override, a native one, or the `Object`
//! default — see `wollok-std`'s `objects` module) and hands back whatever
//! it returns.

use wollok_vm::native::{NativeTable, PrimitiveKind};
use wollok_vm::program::Program;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

pub fn install(table: &mut NativeTable) {
    table.register(PrimitiveKind::Str, "+", 1, concat);
}

fn text(vm: &Vm, this: Value) -> String {
    let idx = this.as_str_idx().expect("registered only for Value::Str receivers");
    vm.strings.get(idx).to_owned()
}

/// # Panics
/// If the argument isn't a `Str`.
fn concat(vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
    let lhs = text(vm, this);
    let rhs_idx = args[0]
        .as_str_idx()
        .unwrap_or_else(|| panic!("expected a String argument, got {:?}", args[0]));
    let rhs = vm.strings.get(rhs_idx);
    let combined = format!("{lhs}{rhs}");
    Value::from(vm.strings.intern(&combined))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concat_joins_two_strings() {
        let mut table = NativeTable::default();
        install(&mut table);
        let mut vm = Vm::new();
        let program = Program::default();

        let a = Value::from(vm.strings.intern("hola, "));
        let b = Value::from(vm.strings.intern("mundo"));

        let f = table.lookup(PrimitiveKind::Str, "+", 1).unwrap();
        let result = f(&mut vm, &program, a, &[b]);

        let idx = result.as_str_idx().unwrap();
        assert_eq!(vm.strings.get(idx), "hola, mundo");
    }
}
