//! Wollok's standard library — the actual native method implementations
//! for `Int`/`Float`/`Bool`/`Str`, plus the `Object` defaults (`objects`).
//! `wollok-vm` only defines the mechanism (`wollok_vm::native::NativeTable`);
//! everything here is a plain `fn(&mut Vm, &Program, Value, &[Value]) ->
//! Value`, one per method, grouped by primitive in its own module.
//!
//! Call [`install`] once, right after building a fresh `Vm` and before
//! compiling/running any Wollok source — `main.rs` does this.

pub mod booleans;
pub mod numbers;
pub mod objects;
pub mod strings;

use wollok_vm::vm::Vm;

pub fn install(vm: &mut Vm) {
    numbers::install(&mut vm.natives);
    booleans::install(&mut vm.natives);
    strings::install(&mut vm.natives);
    objects::install(&mut vm.natives);
}
