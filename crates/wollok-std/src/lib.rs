//! Wollok's standard library. `wollok-vm` only defines the native method
//! mechanism, the implementations live here, one module per primitive.
//! Call [`install`] once right after building a fresh `Vm`.

pub mod booleans;
pub mod console;
pub mod numbers;
pub mod objects;
pub mod strings;
#[cfg(test)]
pub mod testing;
mod util;

use wollok_vm::vm::Vm;

pub fn install(vm: &mut Vm) {
  numbers::install(&mut vm.natives);
  booleans::install(&mut vm.natives);
  strings::install(&mut vm.natives);
  objects::install(&mut vm.natives);
  console::install(vm);
}
