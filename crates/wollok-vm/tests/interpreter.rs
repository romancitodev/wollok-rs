//! End-to-end interpreter tests: hand-assembled bytecode (there's no
//! compiler yet) run through the real Vm, proving Program/Vm/heap/dispatch
//! actually work together — not just each piece in isolation.

use std::collections::HashMap;

use wollok_vm::bytecode::{ConstIdx, Instr, InstrIdx, SlotIdx};
use wollok_vm::heap::FieldIdx;
use wollok_vm::method::Method;
use wollok_vm::program::Program;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

/// Builds a "Counter" class: one field, `setValue(v)` (stores it) and
/// `value()` (reads it back). Returns the class id and both method's
/// selectors, so tests can drive it directly.
fn counter_class(
    program: &mut Program,
) -> (
    wollok_vm::heap::ClassId,
    wollok_vm::bytecode::MethodNameIdx,
    wollok_vm::bytecode::MethodNameIdx,
) {
    let set_value = program.selectors.intern("setValue", 1);
    let value = program.selectors.intern("value", 0);

    let set_value_method = program.methods.define(Method {
        selector: set_value,
        arity: 1,
        extra_locals: 0,
        code: vec![
            Instr::LoadLocal(SlotIdx(0)),
            Instr::StoreField(FieldIdx(0)),
            Instr::PushNull,
            Instr::Return,
        ],
    });

    let value_method = program.methods.define(Method {
        selector: value,
        arity: 0,
        extra_locals: 0,
        code: vec![Instr::LoadField(FieldIdx(0)), Instr::Return],
    });

    let mut vtable = HashMap::new();
    vtable.insert(set_value, set_value_method);
    vtable.insert(value, value_method);
    let class = program.classes.define("Counter", 1, vtable);

    (class, set_value, value)
}

#[test]
fn set_a_field_then_read_it_back_through_two_separate_sends() {
    let mut vm = Vm::new();
    let mut program = Program::default();
    let (counter_class, set_value, value) = counter_class(&mut program);

    program.consts.push(Value::from(42i64));

    let set_value_slot = vm.caches.reserve_slot();
    let value_slot = vm.caches.reserve_slot();

    // main(): const c = new Counter(); c.setValue(42); return c.value()
    let main = program.methods.define(Method {
        selector: program.selectors.intern("main", 0),
        arity: 0,
        extra_locals: 1, // local 0 = the counter instance
        code: vec![
            Instr::NewInstance {
                class: counter_class,
                arg_count: 0,
            },
            Instr::StoreLocal(SlotIdx(0)),
            Instr::LoadLocal(SlotIdx(0)),
            Instr::PushConst(ConstIdx(0)), // 42
            Instr::Send {
                method_name: set_value,
                arg_count: 1,
                cache_slot: set_value_slot,
            },
            Instr::Pop, // discard setValue's null result
            Instr::LoadLocal(SlotIdx(0)),
            Instr::Send {
                method_name: value,
                arg_count: 0,
                cache_slot: value_slot,
            },
            Instr::Return,
        ],
    });

    let result = vm.run_method(&program, main, Value::Null, vec![]);
    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn the_call_site_inline_cache_gets_populated_after_the_first_send() {
    let mut vm = Vm::new();
    let mut program = Program::default();
    let (counter_class, _set_value, value) = counter_class(&mut program);

    let value_slot = vm.caches.reserve_slot();
    let get_value = program.methods.define(Method {
        selector: value,
        arity: 0,
        extra_locals: 1,
        code: vec![
            Instr::NewInstance {
                class: counter_class,
                arg_count: 0,
            },
            Instr::StoreLocal(SlotIdx(0)),
            Instr::LoadLocal(SlotIdx(0)),
            Instr::Send {
                method_name: value,
                arg_count: 0,
                cache_slot: value_slot,
            },
            Instr::Return,
        ],
    });

    assert_eq!(
        vm.caches.get(value_slot).lookup(counter_class),
        None,
        "cache should start empty"
    );

    vm.run_method(&program, get_value, Value::Null, vec![]);

    assert!(
        vm.caches.get(value_slot).lookup(counter_class).is_some(),
        "the Send should have populated the cache for Counter"
    );
}

#[test]
fn the_same_call_site_dispatches_correctly_to_two_different_classes() {
    // Proves the polymorphic cache path end-to-end, not just in
    // dispatch.rs's own unit tests: two classes both implementing
    // `describe()`, sent through the SAME call-site.
    let mut vm = Vm::new();
    let mut program = Program::default();

    let describe = program.selectors.intern("describe", 0);

    let a_method = program.methods.define(Method {
        selector: describe,
        arity: 0,
        extra_locals: 0,
        code: vec![Instr::PushConst(ConstIdx(0)), Instr::Return],
    });
    let b_method = program.methods.define(Method {
        selector: describe,
        arity: 0,
        extra_locals: 0,
        code: vec![Instr::PushConst(ConstIdx(1)), Instr::Return],
    });

    program.consts.push(Value::from(1i64));
    program.consts.push(Value::from(2i64));

    let mut vtable_a = HashMap::new();
    vtable_a.insert(describe, a_method);
    let class_a = program.classes.define("A", 0, vtable_a);

    let mut vtable_b = HashMap::new();
    vtable_b.insert(describe, b_method);
    let class_b = program.classes.define("B", 0, vtable_b);

    let describe_slot = vm.caches.reserve_slot();

    // main(): return [new A().describe(), new B().describe(), new A().describe()]
    // (no array literal instruction yet, so just call three times and
    // check each result directly instead of collecting them)
    let call_describe_on = |class| {
        vec![
            Instr::NewInstance {
                class,
                arg_count: 0,
            },
            Instr::Send {
                method_name: describe,
                arg_count: 0,
                cache_slot: describe_slot,
            },
            Instr::Return,
        ]
    };

    let run_a = program.methods.define(Method {
        selector: program.selectors.intern("runA", 0),
        arity: 0,
        extra_locals: 0,
        code: call_describe_on(class_a),
    });
    let run_b = program.methods.define(Method {
        selector: program.selectors.intern("runB", 0),
        arity: 0,
        extra_locals: 0,
        code: call_describe_on(class_b),
    });

    assert_eq!(
        vm.run_method(&program, run_a, Value::Null, vec![]).as_int(),
        Some(1)
    );
    assert_eq!(
        vm.run_method(&program, run_b, Value::Null, vec![]).as_int(),
        Some(2)
    );
    // Back to A after B: the polymorphic cache must still have it (a
    // monomorphic cache would have evicted A the moment B was seen).
    assert_eq!(
        vm.run_method(&program, run_a, Value::Null, vec![]).as_int(),
        Some(1)
    );
}

#[test]
fn jump_if_false_branches_correctly_both_ways() {
    let mut vm = Vm::new();
    let mut program = Program::default();

    program.consts.push(Value::from(111i64)); // then-branch value
    program.consts.push(Value::from(222i64)); // else-branch value

    // choose(flag): if (flag) 111 else 222 -- built directly with
    // Jump/JumpIfFalse, no `if` sugar exists at this level.
    let choose = program.methods.define(Method {
        selector: program.selectors.intern("choose", 1),
        arity: 1,
        extra_locals: 0,
        code: vec![
            Instr::LoadLocal(SlotIdx(0)),    // 0: push flag
            Instr::JumpIfFalse(InstrIdx(4)), // 1: if false, jump to else
            Instr::PushConst(ConstIdx(0)),   // 2: then-branch
            Instr::Return,                   // 3
            Instr::PushConst(ConstIdx(1)),   // 4: else-branch
            Instr::Return,                   // 5
        ],
    });

    let then_result = vm.run_method(&program, choose, Value::Null, vec![Value::from(true)]);
    assert_eq!(then_result.as_int(), Some(111));

    let else_result = vm.run_method(&program, choose, Value::Null, vec![Value::from(false)]);
    assert_eq!(else_result.as_int(), Some(222));
}

#[test]
#[should_panic(expected = "does not understand")]
fn sending_an_unimplemented_selector_panics_with_a_clear_message() {
    let mut vm = Vm::new();
    let mut program = Program::default();
    let (counter_class, ..) = counter_class(&mut program);

    let missing_slot = vm.caches.reserve_slot();
    let unknown = program.selectors.intern("flyToTheMoon", 0);

    let caller = program.methods.define(Method {
        selector: program.selectors.intern("caller", 0),
        arity: 0,
        extra_locals: 1,
        code: vec![
            Instr::NewInstance {
                class: counter_class,
                arg_count: 0,
            },
            Instr::StoreLocal(SlotIdx(0)),
            Instr::LoadLocal(SlotIdx(0)),
            Instr::Send {
                method_name: unknown,
                arg_count: 0,
                cache_slot: missing_slot,
            },
            Instr::Return,
        ],
    });

    vm.run_method(&program, caller, Value::Null, vec![]);
}
