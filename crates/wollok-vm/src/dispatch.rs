use crate::heap::ClassId;

/// Points at a compiled method. Opaque placeholder until methods actually
/// have compiled bodies to point to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodRef(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheSlotIdx(pub u32);

/// Real-world call-sites almost never see more than a handful of distinct
/// receiver classes (Hölzle/Chambers/Ungar, ECOOP '91) — 4 slots covers
/// that without needing a hashmap.
const POLYMORPHIC_CACHE_SIZE: usize = 4;

/// One polymorphic inline cache per `Send`/`SendSuper` call-site: up to
/// `POLYMORPHIC_CACHE_SIZE` (class, method) pairs, checked linearly (cheap
/// specifically because the array is tiny — no hashing, fits in a cache
/// line). A call site that only ever sees one class behaves exactly like
/// the classic monomorphic cache; one that alternates between a handful of
/// classes doesn't evict on every single call the way a 1-slot cache would.
#[derive(Debug, Clone, Copy)]
pub struct InlineCache {
    entries: [Option<(ClassId, MethodRef)>; POLYMORPHIC_CACHE_SIZE],
    /// Ring-buffer write cursor, used only once every slot is full — new
    /// classes evict the oldest entry, not some fancier "least used" pick.
    /// With this few slots and this little real-world polymorphism, a
    /// smarter policy isn't worth tracking.
    next_write: usize,
}

impl Default for InlineCache {
    fn default() -> Self {
        Self {
            entries: [None; POLYMORPHIC_CACHE_SIZE],
            next_write: 0,
        }
    }
}

impl InlineCache {
    #[must_use]
    pub fn lookup(&self, class: ClassId) -> Option<MethodRef> {
        self.entries.iter().find_map(|entry| {
            entry
                .filter(|(cached_class, _)| *cached_class == class)
                .map(|(_, method)| method)
        })
    }

    pub fn store(&mut self, class: ClassId, method: MethodRef) {
        if let Some(slot) = self
            .entries
            .iter_mut()
            .find(|entry| matches!(entry, Some((c, _)) if *c == class))
        {
            *slot = Some((class, method));
            return;
        }

        if let Some(slot) = self.entries.iter_mut().find(|entry| entry.is_none()) {
            *slot = Some((class, method));
        } else {
            self.entries[self.next_write] = Some((class, method));
            self.next_write = (self.next_write + 1) % POLYMORPHIC_CACHE_SIZE;
        }
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
    fn a_second_distinct_class_does_not_evict_the_first() {
        // The whole point of going polymorphic: alternating between a
        // couple of classes at the same call-site must not thrash.
        let mut cache = InlineCache::default();
        cache.store(ClassId(1), MethodRef(1));
        cache.store(ClassId(2), MethodRef(2));

        assert_eq!(cache.lookup(ClassId(1)), Some(MethodRef(1)));
        assert_eq!(cache.lookup(ClassId(2)), Some(MethodRef(2)));
    }

    #[test]
    fn storing_the_same_class_again_updates_the_method_in_place() {
        // Covers redefinition/relinking: same class, resolved method
        // changes (e.g. the method got recompiled) — must not consume a
        // second slot.
        let mut cache = InlineCache::default();
        cache.store(ClassId(1), MethodRef(1));
        cache.store(ClassId(1), MethodRef(2));
        assert_eq!(cache.lookup(ClassId(1)), Some(MethodRef(2)));
    }

    #[test]
    fn all_four_slots_are_usable_at_once() {
        let mut cache = InlineCache::default();
        for i in 0..4 {
            cache.store(ClassId(i), MethodRef(i * 10));
        }
        for i in 0..4 {
            assert_eq!(cache.lookup(ClassId(i)), Some(MethodRef(i * 10)));
        }
    }

    #[test]
    fn a_fifth_distinct_class_evicts_the_oldest_one() {
        let mut cache = InlineCache::default();
        for i in 0..4 {
            cache.store(ClassId(i), MethodRef(i));
        }
        // Slot 0 (ClassId(0)) was written first, so it's the one a
        // round-robin eviction picks once every slot is full.
        cache.store(ClassId(4), MethodRef(4));

        assert_eq!(
            cache.lookup(ClassId(0)),
            None,
            "the oldest entry should have been evicted"
        );
        assert_eq!(cache.lookup(ClassId(1)), Some(MethodRef(1)));
        assert_eq!(cache.lookup(ClassId(2)), Some(MethodRef(2)));
        assert_eq!(cache.lookup(ClassId(3)), Some(MethodRef(3)));
        assert_eq!(cache.lookup(ClassId(4)), Some(MethodRef(4)));
    }

    #[test]
    fn freeing_a_slot_is_reused_before_evicting_anything() {
        // Update-in-place (same class again) must not advance the
        // round-robin cursor — only a genuinely new class filling the
        // last empty slot should start the eviction rotation.
        let mut cache = InlineCache::default();
        for i in 0..3 {
            cache.store(ClassId(i), MethodRef(i));
        }
        cache.store(ClassId(0), MethodRef(99)); // update in place, slot 3 still empty

        cache.store(ClassId(3), MethodRef(3)); // fills the last empty slot
        assert_eq!(cache.lookup(ClassId(0)), Some(MethodRef(99)));
        assert_eq!(cache.lookup(ClassId(3)), Some(MethodRef(3)));
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
