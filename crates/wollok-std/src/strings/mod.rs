//! `Str` native methods. Same pattern as `numbers`/`booleans`. Stringify a
//! non-`Str` argument with `crate::util::stringify`.

use wollok_vm::native::{NativeTable, PrimitiveKind};
use wollok_vm::program::Program;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

pub fn install(table: &mut NativeTable) {
  table.register(PrimitiveKind::Str, "+", 1, concat);
}

fn value_to_string(vm: &Vm, this: Value) -> String {
  let idx = this
    .as_str_idx()
    .expect("registered only for Value::Str receivers");
  vm.strings.get(idx).to_owned()
}

fn concat(vm: &mut Vm, program: &Program, this: Value, args: &[Value]) -> Value {
  let lhs = value_to_string(vm, this);
  let [rhs] = args else {
    panic!("expected exactly one argument, got {args:?}");
  };
  let rhs = crate::util::stringify(vm, program, *rhs);
  let combined = format!("{lhs}{rhs}");
  Value::from(vm.strings.intern(&combined))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_fn;
  use crate::testing::assert_str;

  test_fn!(@isolated concat_joins_two_strings, |vm: &mut Vm, program: Program, table: NativeTable| {
    let a = Value::from(vm.strings.intern("hola, "));
    let b = Value::from(vm.strings.intern("mundo"));

    let f = table.lookup(PrimitiveKind::Str, "+", 1).unwrap();
    let result = f(vm, &program, a, &[b]);

    assert_str(vm, result, "hola, mundo");
  });

  test_fn!(
    concat_joins_a_string_and_number,
    |vm: &mut Vm, program: Program| {
      let a = Value::from(vm.strings.intern("The answer is "));
      let b = Value::from(42i64);

      let f = vm.natives.lookup(PrimitiveKind::Str, "+", 1).unwrap();
      let result = f(vm, &program, a, &[b]);

      assert_str(vm, result, "The answer is 42");
    }
  );
}
