//! `console`, a builtin singleton compiled from real Wollok source (`SRC`),
//! with `println` as a native method resolved by class name (no `ClassId`
//! juggling, see `wollok_vm::native`).

use wollok_vm::program::Program;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

pub const SRC: &str = "object console {\n  native method println(obj)\n}\n";

pub fn install(vm: &mut Vm) {
  vm.builtin_sources.push(SRC);
  vm.natives
    .register_variadic_for_class("console", "println", println);
}

fn println(vm: &mut Vm, program: &Program, _this: Value, args: &[Value]) -> Value {
  let mut text = String::new();
  for &arg in args {
    text.push_str(&crate::util::stringify(vm, program, arg));
  }
  println!("{text}");
  Value::Null
}
