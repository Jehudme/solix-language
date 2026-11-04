//! Segregated free-list allocator with bidirectional $O(\log K)$ block coalescing.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

/// Minimum remaining fragment size justifying a block split (header + 1 word).
pub const MIN_SPLIT_FRAGMENT: usize = 16;

/// Segregated free-list memory manager tracking unallocated dynamic heap chunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeListManager {
    /// Maps chunk offset -> chunk byte size for address-ordered adjacent coalescing.
    pub free_by_offset: BTreeMap<usize, usize>,
    /// Maps chunk byte size -> set of chunk offsets for best-fit allocation search.
    pub free_by_size: BTreeMap<usize, BTreeSet<usize>>,
    /// Total capacity of the dynamic heap area managed.
    pub total_capacity: usize,
}

impl FreeListManager {
    /// Creates a new free list manager covering the initial contiguous capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let mut free_by_offset = BTreeMap::new();
        let mut free_by_size = BTreeMap::new();

        if capacity > 0 {
            free_by_offset.insert(0, capacity);
            let mut set = BTreeSet::new();
            set.insert(0);
            free_by_size.insert(capacity, set);
        }

        Self {
            free_by_offset,
            free_by_size,
            total_capacity: capacity,
        }
    }

    /// Allocates a contiguous chunk of at least `required_size` bytes.
    ///
    /// Uses a best-fit search. Splits the chunk if leftover capacity exceeds `MIN_SPLIT_FRAGMENT`.
    /// Returns `Some((offset, allocated_size))` on success, or `None` if out of memory.
    pub fn allocate(&mut self, required_size: usize) -> Option<(usize, usize)> {
        // Find the smallest free size that satisfies required_size
        let (&best_size, offsets) = self.free_by_size.range(required_size..).next()?;
        let &offset = offsets.iter().next()?;

        // Remove the block from both indices
        self.remove_free_block(offset, best_size);

        // Determine if we should split the block
        if best_size >= required_size + MIN_SPLIT_FRAGMENT {
            let allocated_size = required_size;
            let remaining_offset = offset + allocated_size;
            let remaining_size = best_size - allocated_size;

            self.insert_free_block(remaining_offset, remaining_size);
            Some((offset, allocated_size))
        } else {
            // Allocate the whole block to avoid tiny unallocatable fragments
            Some((offset, best_size))
        }
    }

    /// Returns an allocated chunk back to the free list and coalesces with adjacent free neighbors.
    pub fn deallocate(&mut self, mut offset: usize, mut size: usize) {
        // 1. Check if the block immediately preceding this one is free
        if let Some((&prev_offset, &prev_size)) = self.free_by_offset.range(..offset).next_back()
            && prev_offset + prev_size == offset
        {
            // Coalesce with left neighbor
            self.remove_free_block(prev_offset, prev_size);
            offset = prev_offset;
            size += prev_size;
        }

        // 2. Check if the block immediately following this one is free
        let next_offset = offset + size;
        if let Some(&next_size) = self.free_by_offset.get(&next_offset) {
            // Coalesce with right neighbor
            self.remove_free_block(next_offset, next_size);
            size += next_size;
        }

        // 3. Re-insert the fully coalesced block
        self.insert_free_block(offset, size);
    }

    /// Checks if the managed heap is fully coalesced into a single initial free block.
    #[must_use]
    pub fn is_fully_coalesced(&self) -> bool {
        if self.free_by_offset.len() != 1 {
            return false;
        }
        self.free_by_offset.get(&0) == Some(&self.total_capacity)
    }

    /// Inserts a block into both the offset and size maps.
    fn insert_free_block(&mut self, offset: usize, size: usize) {
        self.free_by_offset.insert(offset, size);
        self.free_by_size.entry(size).or_default().insert(offset);
    }

    /// Removes a block from both the offset and size maps.
    fn remove_free_block(&mut self, offset: usize, size: usize) {
        self.free_by_offset.remove(&offset);
        if let Some(set) = self.free_by_size.get_mut(&size) {
            set.remove(&offset);
            if set.is_empty() {
                self.free_by_size.remove(&size);
            }
        }
    }
}
