use crate::heap::ClassId;

/// Points at a compiled method. Opaque placeholder until methods actually
/// have compiled bodies to point to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodRef(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheSlotIdx(pub u32);

/// One entry per `Send`/`SendSuper` call-site. Monomorphic only for now
/// (a single cached class+method) — the common case for real code. A
/// polymorphic cache (small array instead of one slot) is a mechanical
/// extension of this, not a rewrite, so it stays deferred until profiling
/// says it's actually needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct InlineCache {
    entry: Option<(ClassId, MethodRef)>,
}

impl InlineCache {
    #[must_use]
    pub fn lookup(&self, class: ClassId) -> Option<MethodRef> {
        self.entry
            .filter(|(cached_class, _)| *cached_class == class)
            .map(|(_, method)| method)
    }

    pub fn store(&mut self, class: ClassId, method: MethodRef) {
        self.entry = Some((class, method));
    }
}

/// Every inline cache slot the compiler has handed out, indexed by
/// `CacheSlotIdx`. The compiler reserves a slot for every `Send` it emits,
/// regardless of whether that call-site turns out hot — retrofitting slots
/// into already-compiled bytecode would mean recompiling everything.
#[derive(Debug, Default)]
pub struct InlineCacheTable {
    slots: Vec<InlineCache>,
}

impl InlineCacheTable {
    /// # Panics
    /// Panics if more than `u32::MAX` call-sites get compiled.
    pub fn reserve_slot(&mut self) -> CacheSlotIdx {
        let idx = self.slots.len();
        self.slots.push(InlineCache::default());
        CacheSlotIdx(u32::try_from(idx).expect("more inline cache slots than u32::MAX"))
    }

    #[must_use]
    pub fn get(&self, slot: CacheSlotIdx) -> &InlineCache {
        &self.slots[slot.0 as usize]
    }

    pub fn get_mut(&mut self, slot: CacheSlotIdx) -> &mut InlineCache {
        &mut self.slots[slot.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_only_for_the_cached_class() {
        let mut cache = InlineCache::default();
        assert_eq!(cache.lookup(ClassId(1)), None);

        cache.store(ClassId(1), MethodRef(42));
        assert_eq!(cache.lookup(ClassId(1)), Some(MethodRef(42)));
        assert_eq!(
            cache.lookup(ClassId(2)),
            None,
            "different class = cache miss"
        );
    }

    #[test]
    fn table_hands_out_distinct_slots() {
        let mut table = InlineCacheTable::default();
        let a = table.reserve_slot();
        let b = table.reserve_slot();
        assert_ne!(a, b);

        table.get_mut(a).store(ClassId(1), MethodRef(7));
        assert_eq!(table.get(a).lookup(ClassId(1)), Some(MethodRef(7)));
        assert_eq!(table.get(b).lookup(ClassId(1)), None);
    }
}
