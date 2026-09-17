//! Native method mechanism: `Int`/`Float`/`Bool`/`Str` primitives, plus
//! two fallback buckets for `Value::Object` (generic `toString`-style
//! defaults, and per-class-name natives like `console`). No
//! implementations here, those live in `wollok-std`, registered into a
//! `Vm` via `wollok_std::install`. Not real inheritance, see
//! `docs/backlog.md` item 4.

use crate::program::Program;
use crate::value::Value;
use crate::vm::Vm;

/// A native method implementation: receiver by value, evaluated args,
/// plus `&Program` (needed for the default `toString` and for `Vm::send`).
pub type NativeFn = fn(&mut Vm, &Program, Value, &[Value]) -> Value;

/// What a native method can be registered against. No real `ClassTable`
/// entry for these, see `vm-design.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
  Int,
  Float,
  Bool,
  Str,
  /// Not a primitive at all — the fallback bucket for `Value::Object`
  /// receivers whose class doesn't define the selector itself. The
  /// closest thing this VM has to a root `Object` class today.
  Object,
}

impl PrimitiveKind {
  /// `None` for `Null`/`Object`, they don't bypass class-based dispatch.
  #[must_use]
  pub fn of(value: &Value) -> Option<Self> {
    match value {
      Value::Int(_) => Some(Self::Int),
      Value::Float(_) => Some(Self::Float),
      Value::Bool(_) => Some(Self::Bool),
      Value::Str(_) => Some(Self::Str),
      Value::Null | Value::Object(_) => None,
    }
  }
}

/// How many arguments a registered native method accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Arity {
  Exact(u8),
  /// Matches any argument count (e.g. `console.println`).
  Any,
}

impl Arity {
  fn matches(self, arity: u8) -> bool {
    match self {
      Arity::Exact(a) => a == arity,
      Arity::Any => true,
    }
  }
}

/// A `PrimitiveKind`, or a class name (a builtin singleton's `native
/// method`, resolved by name since its `ClassId` isn't known yet at
/// registration time).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
  Primitive(PrimitiveKind),
  ClassName(&'static str),
}

#[derive(Debug)]
struct Entry {
  key: Key,
  name: &'static str,
  arity: Arity,
  method: NativeFn,
}

/// Every native method, keyed by `(key, selector, arity)`.
///
/// ponytail: linear scan per lookup, fine at stdlib scale. Switch to a
/// nested `HashMap` if this ever grows into the hundreds.
#[derive(Debug, Default)]
pub struct NativeTable {
  entries: Vec<Entry>,
}

impl NativeTable {
  pub fn register(&mut self, kind: PrimitiveKind, name: &'static str, arity: u8, method: NativeFn) {
    self.entries.push(Entry {
      key: Key::Primitive(kind),
      name,
      arity: Arity::Exact(arity),
      method,
    });
  }

  /// Pins a native method to a class by name, for a builtin singleton's
  /// `native method` (`console`).
  pub fn register_for_class(
    &mut self,
    class_name: &'static str,
    name: &'static str,
    arity: u8,
    method: NativeFn,
  ) {
    self.entries.push(Entry {
      key: Key::ClassName(class_name),
      name,
      arity: Arity::Exact(arity),
      method,
    });
  }

  /// Same as [`Self::register_for_class`], but matches any argument count.
  pub fn register_variadic_for_class(
    &mut self,
    class_name: &'static str,
    name: &'static str,
    method: NativeFn,
  ) {
    self.entries.push(Entry {
      key: Key::ClassName(class_name),
      name,
      arity: Arity::Any,
      method,
    });
  }

  #[must_use]
  pub fn lookup(&self, kind: PrimitiveKind, selector: &str, arity: u8) -> Option<NativeFn> {
    self.lookup_key(Key::Primitive(kind), selector, arity)
  }

  #[must_use]
  pub fn lookup_class(&self, class_name: &str, selector: &str, arity: u8) -> Option<NativeFn> {
    self
      .entries
      .iter()
      .find(|e| {
        matches!(e.key, Key::ClassName(n) if n == class_name)
          && e.arity.matches(arity)
          && e.name == selector
      })
      .map(|e| e.method)
  }

  fn lookup_key(&self, key: Key, selector: &str, arity: u8) -> Option<NativeFn> {
    self
      .entries
      .iter()
      .find(|e| e.key == key && e.arity.matches(arity) && e.name == selector)
      .map(|e| e.method)
  }
}

/// Looks up and calls a native method on `receiver`. Only for true
/// primitives, an `Object` receiver never reaches this.
///
/// # Panics
/// If `receiver`'s kind has no method registered for `selector`/this arity.
pub fn dispatch(
  vm: &mut Vm,
  program: &Program,
  selector: &str,
  receiver: Value,
  args: &[Value],
) -> Value {
  let arity = u8::try_from(args.len()).expect("more than 255 args");
  let found =
    PrimitiveKind::of(&receiver).and_then(|kind| vm.natives.lookup(kind, selector, arity));
  match found {
    Some(method) => method(vm, program, receiver, args),
    None => panic!(
      "{} does not understand #{}",
      receiver_kind_name(&receiver),
      selector
    ),
  }
}

/// Human-readable receiver name for "X does not understand #y" panics.
/// Covers `Null`/`Object` too, unlike `PrimitiveKind`.
#[must_use]
pub fn receiver_kind_name(value: &Value) -> &'static str {
  match value {
    Value::Null => "Null",
    Value::Bool(_) => "Boolean",
    Value::Int(_) => "Integer",
    Value::Float(_) => "Float",
    Value::Str(_) => "String",
    Value::Object(_) => "Object",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn always_42(_vm: &mut Vm, _program: &Program, _receiver: Value, _args: &[Value]) -> Value {
    Value::from(42i64)
  }

  #[test]
  fn a_registered_method_is_found_by_kind_name_and_arity() {
    let mut table = NativeTable::default();
    table.register(PrimitiveKind::Int, "answer", 0, always_42);

    assert!(table.lookup(PrimitiveKind::Int, "answer", 0).is_some());
  }

  #[test]
  fn wrong_arity_or_wrong_kind_misses() {
    let mut table = NativeTable::default();
    table.register(PrimitiveKind::Int, "answer", 0, always_42);

    assert!(table.lookup(PrimitiveKind::Int, "answer", 1).is_none());
    assert!(table.lookup(PrimitiveKind::Bool, "answer", 0).is_none());
  }

  #[test]
  fn object_is_a_separate_bucket_from_every_primitive() {
    let mut table = NativeTable::default();
    table.register(PrimitiveKind::Object, "toString", 0, always_42);

    assert!(table.lookup(PrimitiveKind::Object, "toString", 0).is_some());
    assert!(table.lookup(PrimitiveKind::Int, "toString", 0).is_none());
  }

  #[test]
  fn class_keyed_methods_are_pinned_to_that_exact_class_name() {
    let mut table = NativeTable::default();
    table.register_for_class("console", "println", 1, always_42);

    assert!(table.lookup_class("console", "println", 1).is_some());
    assert!(
      table.lookup_class("Bird", "println", 1).is_none(),
      "a different class must not see console's native method"
    );
    assert!(
      table.lookup(PrimitiveKind::Object, "println", 1).is_none(),
      "a class-keyed method must not leak into the generic Object bucket"
    );
  }

  #[test]
  fn variadic_class_keyed_methods_match_any_arity() {
    let mut table = NativeTable::default();
    table.register_variadic_for_class("console", "println", always_42);

    assert!(table.lookup_class("console", "println", 0).is_some());
    assert!(table.lookup_class("console", "println", 1).is_some());
    assert!(table.lookup_class("console", "println", 5).is_some());
  }

  #[test]
  fn dispatch_calls_the_registered_method() {
    let mut vm = Vm::new();
    let program = Program::default();
    vm.natives
      .register(PrimitiveKind::Int, "answer", 0, always_42);

    assert_eq!(
      dispatch(&mut vm, &program, "answer", Value::from(1i64), &[]).as_int(),
      Some(42)
    );
  }

  #[test]
  #[should_panic(expected = "Integer does not understand #nope")]
  fn dispatch_panics_with_a_clear_message_when_nothing_matches() {
    let mut vm = Vm::new();
    let program = Program::default();
    dispatch(&mut vm, &program, "nope", Value::from(1i64), &[]);
  }

  #[test]
  fn receiver_kind_names_match_wollok_class_names() {
    assert_eq!(receiver_kind_name(&Value::from(1i64)), "Integer");
    assert_eq!(receiver_kind_name(&Value::from(1.0f64)), "Float");
    assert_eq!(receiver_kind_name(&Value::from(true)), "Boolean");
    assert_eq!(receiver_kind_name(&Value::Null), "Null");
  }
}
