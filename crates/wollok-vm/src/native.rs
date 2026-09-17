//! The *mechanism* for native (Rust-implemented) methods on primitive
//! receivers — `Int`, `Float`, `Bool`, `Str` — plus a small fallback
//! bucket, `PrimitiveKind::Object`, for defaults every heap object gets
//! when its own class doesn't override them (`toString`, ...). Primitives
//! are never heap objects (see "Principio: los primitivos nunca son
//! objetos de heap" in `docs/vm-design.md`), so there's no
//! vtable/inline-cache lookup for them: `Send` tries [`dispatch`] first,
//! whenever the receiver isn't a `Value::Object`. For `Object` receivers,
//! `Send` still tries the class's own vtable first — `PrimitiveKind::Object`
//! is only ever consulted once that misses (see `vm.rs`), it is **not**
//! real inheritance/linearization (`inherits`/`with`/`super` chains are
//! still unimplemented — see `docs/backlog.md` item 4).
//!
//! This module deliberately has **no method implementations** — those
//! live in the `wollok-std` crate, one function per method, registered
//! into a fresh `Vm`'s [`NativeTable`] via `wollok_std::install(&mut vm)`
//! before anything runs. `wollok-vm` only needs to know how to call a
//! registered method, not what any of them do — that split is what lets
//! the standard library grow (and get reorganized, tested, reviewed) on
//! its own, without ever touching this crate.

use crate::program::Program;
use crate::value::Value;
use crate::vm::Vm;

/// A single native method's implementation. Takes the receiver by value
/// (primitives are `Copy`) and the already-evaluated arguments. Gets
/// `&Program` too — a default `toString` on `Object` needs it (to find
/// the receiver's class name), and a method that wants to call back into
/// user code (via `Vm::send`) needs it to resolve/run that code.
pub type NativeFn = fn(&mut Vm, &Program, Value, &[Value]) -> Value;

/// The "kinds" a native method can be registered against — the fiction
/// that `Int`/`Float`/`Bool`/`Str` "have a class" (see `vm-design.md`),
/// without an actual entry in `ClassTable`.
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
  /// `None` for `Null`/`Object` — they don't bypass class-based dispatch
  /// the way a true primitive does (see `vm.rs`'s `Send`). Use
  /// `PrimitiveKind::Object` explicitly at the one call site that wants
  /// the object-defaults bucket instead.
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

#[derive(Debug)]
struct Entry {
  kind: PrimitiveKind,
  name: &'static str,
  arity: u8,
  method: NativeFn,
}

/// Every native method available at runtime, keyed by `(kind, selector,
/// arity)` — `wollok-std` (or anyone else) fills this in once via
/// [`NativeTable::register`], `Send` only ever calls [`NativeTable::lookup`].
///
/// ponytail: linear scan per lookup. Fine for a stdlib with a few dozen
/// methods per kind; if this ever grows into the hundreds, switch to
/// `HashMap<PrimitiveKind, HashMap<(&'static str, u8), NativeFn>>` (a flat
/// `HashMap` keyed by the full tuple doesn't work — `&'static str` doesn't
/// implement `Borrow` against a shorter-lived lookup `&str` once it's
/// nested inside a tuple).
#[derive(Debug, Default)]
pub struct NativeTable {
  entries: Vec<Entry>,
}

impl NativeTable {
  pub fn register(&mut self, kind: PrimitiveKind, name: &'static str, arity: u8, method: NativeFn) {
    self.entries.push(Entry {
      kind,
      name,
      arity,
      method,
    });
  }

  #[must_use]
  pub fn lookup(&self, kind: PrimitiveKind, selector: &str, arity: u8) -> Option<NativeFn> {
    self
      .entries
      .iter()
      .find(|e| e.kind == kind && e.arity == arity && e.name == selector)
      .map(|e| e.method)
  }
}

/// Looks up and calls a native method on `receiver`. Only for true
/// primitives — an `Object` receiver never reaches this (see `vm.rs`'s
/// `Send`, which tries the class's own vtable first and only falls back to
/// `PrimitiveKind::Object` itself, not through here).
///
/// # Panics
/// If `receiver`'s kind has no method registered for `selector`/this
/// arity — same "doesn't understand" contract as a `Send` to an object
/// whose class has no matching method.
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

/// Human-readable receiver name for "X does not understand #y" panics —
/// covers `Null`/`Object` too, unlike `PrimitiveKind`, since both can also
/// reach a `Send` that finds nothing.
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
