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
