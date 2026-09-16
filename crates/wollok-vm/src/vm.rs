use crate::bytecode::Instr;
use crate::dispatch::{InlineCacheTable, MethodRef};
use crate::frame::Frame;
use crate::heap::Heap;
use crate::program::Program;
use crate::value::Value;

/// Runtime state that changes while executing: the heap and the inline
/// caches. Everything that's fixed once compiled lives in `Program`
/// instead (see program.rs for why they're split).
#[derive(Debug, Default)]
pub struct Vm {
    pub heap: Heap,
    pub caches: InlineCacheTable,
}

impl Vm {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs one method activation to completion and returns its result.
    /// Recurses (through the real Rust call stack) on every `Send` — fine
    /// for now; a VM-managed call stack is a later concern, not a
    /// correctness one.
    ///
    /// # Panics
    /// Panics on any bytecode invariant violation (stack underflow, a
    /// `Send` whose receiver isn't an object yet — primitives don't have
    /// methods of their own until native dispatch exists, a selector the
    /// receiver's class doesn't implement) and on a handful of
    /// instructions (`NewArray`, `NewSet`, `NewClosure`, try/catch,
    /// `SendSuper`) that aren't wired yet.
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
                    let obj = receiver_v.as_object().unwrap_or_else(|| {
                        panic!(
                            "cannot send #{} to a non-object value yet (no native/primitive methods)",
                            program.selectors.name_of(*method_name)
                        )
                    });
                    let class = self.heap.class_of(obj);

                    let resolved =
                        if let Some(method_ref) = self.caches.get(*cache_slot).lookup(class) {
                            method_ref
                        } else {
                            let method_ref = program
                                .classes
                                .lookup(class, *method_name)
                                .unwrap_or_else(|| {
                                    panic!(
                                        "{} does not understand #{}",
                                        program.classes.name_of(class),
                                        program.selectors.name_of(*method_name)
                                    )
                                });
                            self.caches.get_mut(*cache_slot).store(class, method_ref);
                            method_ref
                        };

                    let result = self.run_method(program, resolved, receiver_v, call_args);
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
                    // Constructors aren't wired yet (the parser doesn't
                    // even have `constructor(...)` yet either) — args are
                    // consumed off the stack but discarded, fields start
                    // as Null.
                    for _ in 0..*arg_count {
                        frame.pop();
                    }
                    let field_count = program.classes.field_count(*class);
                    let obj = self
                        .heap
                        .alloc(*class, vec![Value::Null; field_count as usize]);
                    frame.push(Value::from(obj));
                }

                other => todo!("instruction not implemented yet: {other:?}"),
            }
        }
    }
}
