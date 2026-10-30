//! Bounded actor/event pool identity model derived from B71B70/B71C50/
//! B71CC0/B71D20/B71D50/B71C00. Source identity groundwork, not gameplay integration.
//!
//! The source pool is unchecked and stores a raw 32-bit handle in each slot.
//! This model adds explicit occupancy, checked access and a conservative
//! generation boundary. It models no object teardown, aliasing, mutation during
//! native iteration, allocation failure, or runtime thread safety. See
//! re/cell-actor-pool-handle-static-v1.md.

/// Conservatively exclude index 0xffff: the source free-list sentinel is the
/// full word 0xffffffff, whose low 16 bits can make a freed last slot appear
/// occupied to the native iterator. The observed Cell capacities are below it.
pub const MAX_CAPACITY: usize = 0xffff;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolError {
    InvalidCapacity,
    Full,
    /// Next encoded generation would wrap through zero. No pool state changes.
    GenerationBoundary,
    InvalidHandle,
}

#[derive(Debug)]
struct Slot<T> {
    handle: u32,
    value: Option<T>,
    next_free: Option<usize>,
}

/// A raw handle has meaning only in the pool that issued it. Cross-pool
/// collision prevention and external handle ownership remain caller duties.
#[derive(Debug)]
pub struct CheckedActorPool<T> {
    capacity: usize,
    counter: u32,
    live: usize,
    free_head: Option<usize>,
    slots: Vec<Slot<T>>, // length equals the source high-water count
}

impl<T> CheckedActorPool<T> {
    /// `initial_counter` is explicit source state (+08), not a fabricated
    /// global initializer. A source initialization seed is global_word << 8.
    pub fn new(capacity: usize, initial_counter: u32) -> Result<Self, PoolError> {
        if capacity == 0 || capacity > MAX_CAPACITY { return Err(PoolError::InvalidCapacity); }
        Ok(Self { capacity, counter: initial_counter, live: 0, free_head: None, slots: Vec::new() })
    }

    pub fn live_count(&self) -> usize { self.live }
    pub fn high_water(&self) -> usize { self.slots.len() }
    pub fn counter(&self) -> u32 { self.counter }

    /// Adds an explicitly constructed value. Native allocation zeroes the full
    /// slot before higher-level initialization; this typed API does not model
    /// those bytes or authorize a source actor to become active by itself.
    pub fn allocate(&mut self, value: T) -> Result<u32, PoolError> {
        if self.live == self.capacity { return Err(PoolError::Full); }
        let next = self.counter.wrapping_add(1);
        // Conservative safety policy: stop before encoded generation crosses
        // zero. This prevents zero handles and reuse of prior generations in
        // this pool epoch, at the cost of rejecting some native operations.
        if (next & 0xffff) == 0 { return Err(PoolError::GenerationBoundary); }
        let index = if self.live == self.slots.len() { self.slots.len() }
                    else { self.free_head.expect("checked pool free-list invariant") };
        let handle = (next << 16) | (index as u32);
        if index == self.slots.len() {
            self.slots.push(Slot { handle, value: Some(value), next_free: None });
        } else {
            let slot = &mut self.slots[index];
            self.free_head = slot.next_free.take();
            slot.handle = handle;
            slot.value = Some(value);
        }
        self.counter = next;
        self.live += 1;
        Ok(handle)
    }

    fn valid_index(&self, handle: u32) -> Option<usize> {
        if handle == 0 { return None; }
        let index = (handle & 0xffff) as usize;
        let slot = self.slots.get(index)?; // strict index < high-water and capacity
        if slot.handle != handle || slot.value.is_none() { return None; }
        Some(index)
    }

    pub fn get(&self, handle: u32) -> Option<&T> {
        self.valid_index(handle).and_then(|index| self.slots[index].value.as_ref())
    }
    pub fn get_mut(&mut self, handle: u32) -> Option<&mut T> {
        let index = self.valid_index(handle)?;
        self.slots[index].value.as_mut()
    }

    /// Returns the removed value for caller-owned teardown. Native B71D50 does
    /// not validate the handle or invoke a destructor; these checks are safety
    /// additions and teardown order must be established by the actual caller.
    pub fn release(&mut self, handle: u32) -> Result<T, PoolError> {
        let index = self.valid_index(handle).ok_or(PoolError::InvalidHandle)?;
        let slot = &mut self.slots[index];
        let value = slot.value.take().expect("checked occupied slot");
        slot.handle = 0;
        slot.next_free = self.free_head;
        self.free_head = Some(index);
        self.live -= 1;
        Ok(value)
    }

    /// Iterates live slots in ascending slot order, consistent with the source
    /// scan under valid non-mutating pool invariants. Borrowing prevents mutation
    /// through this pool during the iterator's lifetime.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.slots.iter().filter_map(|slot| slot.value.as_ref().map(|v| (slot.handle, v)))
    }

    /// Clears checked occupancy and high-water, keeps the source counter, and
    /// returns values so the caller can handle teardown. Exact source reset only
    /// resets counters/head; object teardown and stale bytes are separate.
    pub fn reset(&mut self) -> Vec<T> {
        let values = self.slots.drain(..).filter_map(|slot| slot.value).collect();
        self.live = 0;
        self.free_head = None;
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocation_capacity_and_lifo_free_list() {
        let mut p = CheckedActorPool::new(3, 0).unwrap();
        let a = p.allocate('a').unwrap();
        let b = p.allocate('b').unwrap();
        let c = p.allocate('c').unwrap();
        assert_eq!((a,b,c), (0x0001_0000,0x0002_0001,0x0003_0002));
        assert_eq!(p.allocate('d'), Err(PoolError::Full));
        assert_eq!(p.release(a), Ok('a'));
        assert_eq!(p.release(c), Ok('c'));
        let d = p.allocate('d').unwrap();
        let e = p.allocate('e').unwrap();
        assert_eq!(d, 0x0004_0002);
        assert_eq!(e, 0x0005_0000);
        assert_eq!(p.iter().map(|(h,v)| (h,*v)).collect::<Vec<_>>(), vec![(e,'e'),(b,'b'),(d,'d')]);
        assert_eq!(p.high_water(),3);
    }
    #[test]
    fn rejects_zero_stale_double_free_and_high_water_index() {
        let mut p = CheckedActorPool::new(2, 0).unwrap();
        let h = p.allocate(11).unwrap();
        assert_eq!(p.get(0), None);
        assert_eq!(p.get(0x0001_0001), None); // index exactly high-water
        assert_eq!(p.release(0x0001_0001), Err(PoolError::InvalidHandle));
        assert_eq!(p.release(h), Ok(11));
        assert_eq!(p.release(h), Err(PoolError::InvalidHandle));
        let replacement = p.allocate(12).unwrap();
        assert_ne!(replacement,h);
        assert_eq!(p.get(h), None);
        assert_eq!(p.get(replacement), Some(&12));
    }
    #[test]
    fn rejects_generation_boundary_without_mutation() {
        let mut p = CheckedActorPool::new(1, 0xfffe).unwrap();
        let last = p.allocate(4).unwrap();
        assert_eq!(last,0xffff_0000);
        assert_eq!(p.release(last), Ok(4));
        assert_eq!(p.allocate(5), Err(PoolError::GenerationBoundary));
        assert_eq!((p.counter(),p.live_count(),p.high_water()),(0xffff,0,1));
        assert_eq!(p.get(last), None);
    }
    #[test]
    fn boundary_preserves_live_values_and_free_head() {
        let mut p = CheckedActorPool::new(3, 0xfffd).unwrap();
        let freed = p.allocate(10).unwrap();
        let live = p.allocate(20).unwrap();
        assert_eq!(p.release(freed), Ok(10));
        assert_eq!(p.free_head, Some(0));
        assert_eq!(p.allocate(30), Err(PoolError::GenerationBoundary));
        assert_eq!(p.free_head, Some(0));
        assert_eq!((p.slots[0].handle, p.slots[0].next_free, p.slots[0].value), (0, None, None));
        assert_eq!((p.counter(), p.live_count(), p.high_water()), (0xffff, 1, 2));
        assert_eq!(p.get(live), Some(&20));
        assert_eq!(p.iter().map(|(h, v)| (h, *v)).collect::<Vec<_>>(), vec![(live, 20)]);
    }
    #[test]
    fn reset_retains_counter_and_invalidates_handles() {
        let mut p = CheckedActorPool::new(2, 0x1234).unwrap();
        let h = p.allocate(7).unwrap();
        assert_eq!(p.reset(), vec![7]);
        assert_eq!((p.counter(),p.live_count(),p.high_water()),(0x1235,0,0));
        assert_eq!(p.get(h),None);
        let next = p.allocate(8).unwrap();
        assert_ne!(next,h);
        assert_eq!(p.get(h),None);
    }
    #[test]
    fn reset_with_holes_discards_free_list_and_preserves_slot_order_after_reuse() {
        let mut p = CheckedActorPool::new(3, 0).unwrap();
        let first = p.allocate('a').unwrap();
        let middle = p.allocate('b').unwrap();
        let last = p.allocate('c').unwrap();
        assert_eq!(p.release(middle), Ok('b'));
        let reused = p.allocate('d').unwrap();
        assert_eq!(p.iter().map(|(h, v)| (h, *v)).collect::<Vec<_>>(),
                   vec![(first, 'a'), (reused, 'd'), (last, 'c')]);
        assert_eq!(p.release(first), Ok('a'));
        assert_eq!(p.reset(), vec!['d', 'c']);
        assert_eq!((p.free_head, p.live_count(), p.high_water()), (None, 0, 0));
        assert_eq!(p.get(reused), None);
        assert_eq!(p.get(last), None);
        let new_handle = p.allocate('e').unwrap();
        assert_eq!(new_handle & 0xffff, 0);
        assert_eq!(p.iter().map(|(h, v)| (h, *v)).collect::<Vec<_>>(), vec![(new_handle, 'e')]);
    }
    #[test]
    fn capacity_limits_are_explicit() {
        assert!(matches!(CheckedActorPool::<()>::new(0,0), Err(PoolError::InvalidCapacity)));
        assert!(matches!(CheckedActorPool::<()>::new(MAX_CAPACITY+1,0), Err(PoolError::InvalidCapacity)));
    }
}
