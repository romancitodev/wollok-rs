//! `Int` native methods. `Float` has none yet — same pattern, new module.
//!
//! To add one: write a `fn(&mut Vm, &Program, Value, &[Value]) -> Value`
//! below and register it in [`install`].

use wollok_vm::native::{NativeTable, PrimitiveKind};
use wollok_vm::program::Program;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

pub fn install(table: &mut NativeTable) {
  table.register(PrimitiveKind::Int, "+", 1, plus);
  table.register(PrimitiveKind::Int, "-", 1, minus);
  table.register(PrimitiveKind::Int, "*", 1, times);
  table.register(PrimitiveKind::Int, "/", 1, div);
  table.register(PrimitiveKind::Int, "%", 1, modulo);
  table.register(PrimitiveKind::Int, "<", 1, lt);
  table.register(PrimitiveKind::Int, "<=", 1, le);
  table.register(PrimitiveKind::Int, ">", 1, gt);
  table.register(PrimitiveKind::Int, ">=", 1, ge);
  table.register(PrimitiveKind::Int, "==", 1, eq);
  table.register(PrimitiveKind::Int, "!=", 1, ne);
}

fn receiver(this: Value) -> i64 {
  this
    .as_int()
    .expect("registered only for Value::Int receivers")
}

/// # Panics
/// If the argument isn't an `Int` — arithmetic/ordering across types is a
/// real type error, unlike `==`/`!=` below, which compare cleanly instead.
fn arg(args: &[Value]) -> i64 {
  args[0]
    .as_int()
    .unwrap_or_else(|| panic!("expected an Integer argument, got {:?}", args[0]))
}

fn plus(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) + arg(args))
}

fn minus(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) - arg(args))
}

fn times(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) * arg(args))
}

fn div(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) / arg(args))
}

fn modulo(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) % arg(args))
}

fn lt(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) < arg(args))
}

fn le(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) <= arg(args))
}

fn gt(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) > arg(args))
}

fn ge(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(receiver(this) >= arg(args))
}

// `==`/`!=` compare against `args[0].as_int()` directly (an `Option<i64>`)
// instead of going through `arg()`: a type mismatch is just "not equal",
// not a panic — `5 == "hola"` is `false`, it doesn't blow up.
fn eq(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(Some(receiver(this)) == args[0].as_int())
}

fn ne(_vm: &mut Vm, _program: &Program, this: Value, args: &[Value]) -> Value {
  Value::from(Some(receiver(this)) != args[0].as_int())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn table() -> NativeTable {
    let mut table = NativeTable::default();
    install(&mut table);
    table
  }

  #[test]
  fn arithmetic() {
    let table = table();
    let mut vm = Vm::new();
    let program = Program::default();
    let mut call = |name, a: i64, b: i64| {
      let f = table.lookup(PrimitiveKind::Int, name, 1).unwrap();
      f(&mut vm, &program, Value::from(a), &[Value::from(b)])
    };

    assert_eq!(call("+", 2, 3).as_int(), Some(5));
    assert_eq!(call("-", 5, 3).as_int(), Some(2));
    assert_eq!(call("*", 4, 3).as_int(), Some(12));
    assert_eq!(call("/", 7, 2).as_int(), Some(3));
    assert_eq!(call("%", 7, 2).as_int(), Some(1));
  }

  #[test]
  fn comparisons() {
    let table = table();
    let mut vm = Vm::new();
    let program = Program::default();
    let f = table.lookup(PrimitiveKind::Int, "<", 1).unwrap();
    assert_eq!(
      f(&mut vm, &program, Value::from(2i64), &[Value::from(3i64)]).as_bool(),
      Some(true)
    );
  }

  #[test]
  fn equality_across_types_is_false_not_a_panic() {
    let table = table();
    let mut vm = Vm::new();
    let program = Program::default();
    let f = table.lookup(PrimitiveKind::Int, "==", 1).unwrap();
    assert_eq!(
      f(&mut vm, &program, Value::from(5i64), &[Value::from(true)]).as_bool(),
      Some(false)
    );
  }
}
