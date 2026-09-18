use wollok_vm::program::Program;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

/// Text for `value`: itself if it's already a `Str`, otherwise whatever
/// its `toString` returns.
pub(crate) fn stringify(vm: &mut Vm, program: &Program, value: Value) -> String {
  if let Some(idx) = value.as_str_idx() {
    return vm.strings.get(idx).to_owned();
  }
  let result = vm.send(program, value, "toString", vec![]);
  let idx = result
    .as_str_idx()
    .unwrap_or_else(|| panic!("expected a String result from toString, got {result:?}"));
  vm.strings.get(idx).to_owned()
}
