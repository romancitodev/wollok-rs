use crate::bytecode::{GlobalIdx, Instr};
use crate::dispatch::{InlineCacheTable, MethodRef};
use crate::frame::Frame;
use crate::heap::Heap;
use crate::native;
use crate::program::Program;
use crate::strings::StringTable;
use crate::value::Value;

#[derive(Debug, Default)]
pub struct Vm {
  pub heap: Heap,
  pub caches: InlineCacheTable,
  /// One slot per top-level singleton `object`, reserved at compile time.
  pub globals: Vec<Value>,
  /// Runtime string storage, see `crate::strings::StringTable`.
  pub strings: StringTable,
  /// Native methods, filled in by `wollok_std::install` right after
  /// `Vm::new()`.
  pub natives: native::NativeTable,
  /// Wollok source for builtin singletons (`console`, ...), compiled
  /// alongside the user's own source. See `wollok-std`'s `console` module.
  pub builtin_sources: Vec<&'static str>,
}

impl Vm {
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// # Panics
  /// Panics if more than `u32::MAX` globals get compiled.
  pub fn reserve_global(&mut self) -> GlobalIdx {
    let idx = self.globals.len();
    self.globals.push(Value::Null);
    GlobalIdx(u32::try_from(idx).expect("more globals than u32::MAX"))
  }

  /// Runs one method to completion and returns its result. Recurses on
  /// every `Send`.
  ///
  /// # Panics
  /// On any bytecode invariant violation, and on the instructions not
  /// wired yet (`NewArray`, `NewSet`, `NewClosure`, try/catch, `SendSuper`).
  pub fn run_method(
    &mut self,
    program: &Program,
    method_ref: MethodRef,
    receiver: Value,
    args: Vec<Value>,
  ) -> Value {
    let method = program.methods.get(method_ref);

    let mut locals = args;
    locals.resize(locals.len() + method.extra_locals as usize, Value::Null);
    let mut frame = Frame::new(receiver, locals);

    loop {
      let instr = &method.code[frame.ip];
      frame.ip += 1;

      match instr {
        Instr::PushConst(idx) => frame.push(program.consts[idx.0 as usize]),
        Instr::PushNull => frame.push(Value::Null),
        Instr::PushTrue => frame.push(Value::from(true)),
        Instr::PushFalse => frame.push(Value::from(false)),
        Instr::PushSelf => frame.push(frame.receiver),

        Instr::LoadLocal(slot) => frame.push(frame.locals[slot.0 as usize]),
        Instr::StoreLocal(slot) => {
          let value = frame.pop();
          frame.locals[slot.0 as usize] = value;
        }

        Instr::LoadField(field) => {
          let obj = frame
            .receiver
            .as_object()
            .expect("LoadField outside a method running on an object receiver");
          frame.push(self.heap.read_field(obj, *field));
        }
        Instr::StoreField(field) => {
          let value = frame.pop();
          let obj = frame
            .receiver
            .as_object()
            .expect("StoreField outside a method running on an object receiver");
          self.heap.write_field(obj, *field, value);
        }

        Instr::LoadGlobal(idx) => frame.push(self.globals[idx.0 as usize]),
        Instr::StoreGlobal(idx) => {
          let value = frame.pop();
          self.globals[idx.0 as usize] = value;
        }

        Instr::Pop => {
          frame.pop();
        }

        Instr::Send {
          method_name,
          arg_count,
          cache_slot,
        } => {
          let mut call_args = vec![Value::Null; *arg_count as usize];
          for slot in call_args.iter_mut().rev() {
            *slot = frame.pop();
          }
          let receiver_v = frame.pop();

          let result = if let Some(obj) = receiver_v.as_object() {
            let class = self.heap.class_of(obj);
            if let Some(method_ref) = self.caches.get(*cache_slot).lookup(class) {
              self.run_method(program, method_ref, receiver_v, call_args)
            } else if let Some(method_ref) = program.classes.lookup(class, *method_name) {
              self.caches.get_mut(*cache_slot).store(class, method_ref);
              self.run_method(program, method_ref, receiver_v, call_args)
            } else {
              // No vtable entry: try a native pinned to this class,
              // then the generic Object fallback. Not inheritance,
              // see docs/backlog.md item 4.
              let selector_name = program.selectors.name_of(*method_name);
              let found = self
                .natives
                .lookup_class(program.classes.name_of(class), selector_name, *arg_count)
                .or_else(|| {
                  self
                    .natives
                    .lookup(native::PrimitiveKind::Object, selector_name, *arg_count)
                });
              match found {
                Some(method) => method(self, program, receiver_v, &call_args),
                None => panic!(
                  "{} does not understand #{}",
                  program.classes.name_of(class),
                  selector_name
                ),
              }
            }
          } else {
            let selector_name = program.selectors.name_of(*method_name);
            native::dispatch(self, program, selector_name, receiver_v, &call_args)
          };
          frame.push(result);
        }

        Instr::Jump(target) => frame.ip = target.0 as usize,
        Instr::JumpIfFalse(target) => {
          let value = frame.pop();
          let cond = value.as_bool().expect("JumpIfFalse on a non-boolean value");
          if !cond {
            frame.ip = target.0 as usize;
          }
        }
        Instr::Return => return frame.pop(),

        Instr::NewInstance { class, arg_count } => {
          // constructor params not supported yet: args discarded
          for _ in 0..*arg_count {
            frame.pop();
          }
          let field_count = program.classes.field_count(*class);
          let obj = self
            .heap
            .alloc(*class, vec![Value::Null; field_count as usize]);
          if let Some(ctor) = program.classes.ctor(*class) {
            self.run_method(program, ctor, Value::from(obj), vec![]);
          }
          frame.push(Value::from(obj));
        }

        other => todo!("instruction not implemented yet: {other:?}"),
      }
    }
  }

  /// Sends `selector` to `receiver`, for native methods calling back
  /// into user code. No cache, always resolves from scratch.
  ///
  /// # Panics
  /// If `receiver` doesn't understand `selector`/this arity.
  pub fn send(
    &mut self,
    program: &Program,
    receiver: Value,
    selector: &str,
    args: Vec<Value>,
  ) -> Value {
    let arity = u8::try_from(args.len()).expect("more than 255 args");

    if let Some(obj) = receiver.as_object() {
      let class = self.heap.class_of(obj);
      let overridden = program
        .selectors
        .id_of(selector, arity)
        .and_then(|id| program.classes.lookup(class, id));
      if let Some(method_ref) = overridden {
        return self.run_method(program, method_ref, receiver, args);
      }
      let found = self
        .natives
        .lookup_class(program.classes.name_of(class), selector, arity)
        .or_else(|| {
          self
            .natives
            .lookup(native::PrimitiveKind::Object, selector, arity)
        });
      return match found {
        Some(method) => method(self, program, receiver, &args),
        None => panic!(
          "{} does not understand #{}",
          program.classes.name_of(class),
          selector
        ),
      };
    }

    native::dispatch(self, program, selector, receiver, &args)
  }
}
