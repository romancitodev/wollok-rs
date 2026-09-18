//! `Bool` native methods. `&&`/`||` are compiled as jumps, not `Send`
//! (see `wollok-compiler`), so they never reach here.

use wollok_vm::native::{NativeTable, PrimitiveKind};
use wollok_vm::program::Program;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

pub fn install(table: &mut NativeTable) {
  table.register(PrimitiveKind::Bool, "negate", 0, negate);
  table.register(PrimitiveKind::Bool, "toString", 0, to_string);
}

fn to_string(vm: &mut Vm, _program: &Program, this: Value, _args: &[Value]) -> Value {
  let s = this
    .as_bool()
    .expect("registered only for Value::Bool receivers")
    .to_string();
  Value::from(vm.strings.intern(&s))
}

fn negate(_vm: &mut Vm, _program: &Program, this: Value, _args: &[Value]) -> Value {
  Value::from(
    !this
      .as_bool()
      .expect("registered only for Value::Bool receivers"),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn negate_flips_the_receiver() {
    let mut table = NativeTable::default();
    install(&mut table);
    let mut vm = Vm::new();
    let program = Program::default();

    let f = table.lookup(PrimitiveKind::Bool, "negate", 0).unwrap();
    assert_eq!(
      f(&mut vm, &program, Value::from(true), &[]).as_bool(),
      Some(false)
    );
  }
}
