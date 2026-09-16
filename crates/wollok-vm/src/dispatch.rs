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

    #[test]
    fn a_fresh_cache_has_no_entry() {
        let cache = InlineCache::default();
        assert_eq!(cache.lookup(ClassId(1)), None);
        assert_eq!(cache.lookup(ClassId(2)), None);
    }

    #[test]
    fn monomorphic_cache_is_overwritten_by_the_latest_class_only() {
        // Today's cache is strictly monomorphic: a polymorphic call-site
        // (receiver class changes between calls) doesn't grow the cache,
        // it just replaces the entry. Locking this in explicitly because
        // it's exactly the behavior a future polymorphic cache would
        // change on purpose.
        let mut cache = InlineCache::default();
        cache.store(ClassId(1), MethodRef(1));
        assert_eq!(cache.lookup(ClassId(1)), Some(MethodRef(1)));

        cache.store(ClassId(2), MethodRef(2));
        assert_eq!(
            cache.lookup(ClassId(1)),
            None,
            "storing a new class must evict the old one in a monomorphic cache"
        );
        assert_eq!(cache.lookup(ClassId(2)), Some(MethodRef(2)));
    }

    #[test]
    fn storing_the_same_class_again_updates_the_method() {
        // Covers redefinition/relinking: same class, resolved method
        // changes (e.g. the method got recompiled).
        let mut cache = InlineCache::default();
        cache.store(ClassId(1), MethodRef(1));
        cache.store(ClassId(1), MethodRef(2));
        assert_eq!(cache.lookup(ClassId(1)), Some(MethodRef(2)));
    }

    #[test]
    fn reserved_slots_are_assigned_in_order() {
        let mut table = InlineCacheTable::default();
        let slots: Vec<_> = (0..5).map(|_| table.reserve_slot()).collect();
        let indices: Vec<u32> = slots.iter().map(|s| s.0).collect();
        assert_eq!(indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn mutating_one_slot_never_touches_another() {
        let mut table = InlineCacheTable::default();
        let a = table.reserve_slot();
        let b = table.reserve_slot();
        let c = table.reserve_slot();

        table.get_mut(a).store(ClassId(1), MethodRef(10));
        table.get_mut(c).store(ClassId(2), MethodRef(30));

        assert_eq!(table.get(a).lookup(ClassId(1)), Some(MethodRef(10)));
        assert_eq!(
            table.get(b).lookup(ClassId(1)),
            None,
            "b was never stored into"
        );
        assert_eq!(table.get(c).lookup(ClassId(2)), Some(MethodRef(30)));
    }
}
