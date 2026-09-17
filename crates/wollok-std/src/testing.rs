//! Shared test helpers for every native module in this crate.

/// # Panics
/// If `value` isn't a `Str` or its text doesn't match `expected`.
#[track_caller]
pub fn assert_str(vm: &wollok_vm::vm::Vm, value: wollok_vm::value::Value, expected: &str) {
  let idx = value.as_str_idx().expect("expected a String result");
  assert_eq!(vm.strings.get(idx), expected);
}

/// Builds a `#[test]` fn with a `Vm`/`Program` (and, `@isolated`, a bare
/// `NativeTable`) already set up. `$body` gets them as arguments.
#[macro_export]
macro_rules! test_fn {
  (@isolated $name:tt, $body:expr) => {
    #[test]
    fn $name() {
      let mut table = wollok_vm::native::NativeTable::default();
      install(&mut table);
      $body(
        &mut wollok_vm::vm::Vm::new(),
        wollok_vm::program::Program::default(),
        table,
      );
    }
  };

  ($name:tt, $body:expr) => {
    #[test]
    fn $name() {
      let mut vm = wollok_vm::vm::Vm::new();
      $crate::install(&mut vm);
      $body(&mut vm, wollok_vm::program::Program::default());
    }
  };
}
