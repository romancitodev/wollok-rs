use crate::dispatch::CacheSlotIdx;
use crate::heap::{ClassId, FieldIdx};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstIdx(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotIdx(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodNameIdx(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClosureIdx(pub u32);

/// Jump target as an absolute index into the instruction stream, decided
/// once by the compiler — never a relative offset the VM has to add up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstrIdx(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    PushConst(ConstIdx),
    PushNull,
    PushTrue,
    PushFalse,

    LoadLocal(SlotIdx),
    StoreLocal(SlotIdx),

    LoadField(FieldIdx),
    StoreField(FieldIdx),

    /// `cache_slot` is reserved by the compiler for every `Send` it
    /// emits, monomorphic-cache logic today — see dispatch.rs. Never
    /// added after the fact: that would mean recompiling everything
    /// already emitted.
    Send {
        method_name: MethodNameIdx,
        arg_count: u8,
        cache_slot: CacheSlotIdx,
    },
    SendSuper {
        method_name: MethodNameIdx,
        arg_count: u8,
        cache_slot: CacheSlotIdx,
    },

    Jump(InstrIdx),
    JumpIfFalse(InstrIdx),
    Return,

    NewInstance {
        class: ClassId,
        arg_count: u8,
    },
    NewArray(u32),
    NewSet(u32),
    NewClosure(ClosureIdx),

    PushTryHandler(InstrIdx),
    PopTryHandler,
    Throw,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send(cache_slot: u32) -> Instr {
        Instr::Send {
            method_name: MethodNameIdx(0),
            arg_count: 1,
            cache_slot: CacheSlotIdx(cache_slot),
        }
    }

    #[test]
    fn send_and_send_super_with_identical_fields_are_still_different_instructions() {
        let send = send(0);
        let send_super = Instr::SendSuper {
            method_name: MethodNameIdx(0),
            arg_count: 1,
            cache_slot: CacheSlotIdx(0),
        };
        assert_ne!(send, send_super);
    }

    #[test]
    fn send_equality_accounts_for_its_cache_slot() {
        // Two Sends to the same selector at different call-sites must NOT
        // be equal, because they carry different (independent) caches.
        assert_ne!(send(0), send(1));
        assert_eq!(send(0), send(0));
    }

    #[test]
    fn unit_variants_compare_equal_to_themselves() {
        assert_eq!(Instr::PushNull, Instr::PushNull);
        assert_eq!(Instr::Return, Instr::Return);
        assert_eq!(Instr::PopTryHandler, Instr::PopTryHandler);
        assert_ne!(Instr::PushTrue, Instr::PushFalse);
    }

    #[test]
    fn jump_targets_are_absolute_instruction_indices() {
        let jump = Instr::Jump(InstrIdx(42));
        let Instr::Jump(target) = jump else {
            unreachable!()
        };
        assert_eq!(target.0, 42);
    }

    #[test]
    fn new_instance_carries_class_and_arg_count() {
        let instr = Instr::NewInstance {
            class: ClassId(3),
            arg_count: 2,
        };
        let Instr::NewInstance { class, arg_count } = instr else {
            unreachable!()
        };
        assert_eq!(class, ClassId(3));
        assert_eq!(arg_count, 2);
    }
}
