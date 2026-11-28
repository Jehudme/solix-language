//! Segregated free-list allocator with O(1) bitmask size-class lookup and bidirectional coalescing.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// Minimum remaining fragment size justifying a block split (header + 1 word).
pub const MIN_SPLIT_FRAGMENT: usize = 16;

/// Number of power-of-two size-class bins (`16B` .. `>= 32 GiB`).
const NUM_SIZE_BINS: usize = 32;

/// Maps a block byte size (`>= 16`) to its power-of-two size-class bin (`0..32`).
#[inline]
const fn bin_index_for_size(size: usize) -> usize {
    let clamped = if size < MIN_SPLIT_FRAGMENT {
        MIN_SPLIT_FRAGMENT
    } else {
        size
    };
    let floor_log2 = (usize::BITS - 1 - clamped.leading_zeros()) as usize;
    let bin = floor_log2.saturating_sub(4);
    if bin >= NUM_SIZE_BINS {
        NUM_SIZE_BINS - 1
    } else {
        bin
    }
}

/// Segregated free-list memory manager tracking unallocated dynamic heap chunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeListManager {
    /// Maps chunk offset -> chunk byte size for address-ordered adjacent coalescing.
    pub free_by_offset: BTreeMap<usize, usize>,
    /// Segregated `(offset, size)` buckets indexed by power-of-two size class.
    size_bins: [Vec<(usize, usize)>; NUM_SIZE_BINS],
    /// Bitmask indicating which `size_bins` entries are non-empty for O(1) lookup.
    non_empty_bins: u32,
    /// Total capacity of the dynamic heap area managed.
    pub total_capacity: usize,
}

impl FreeListManager {
    /// Creates a new free list manager covering the initial contiguous capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let mut manager = Self::new_empty(capacity);
        if capacity > 0 {
            manager.insert_free_block(0, capacity);
        }
        manager
    }

    /// Creates a new free list manager with no free blocks initially (used when paired with a bump allocator).
    #[must_use]
    pub fn new_empty(capacity: usize) -> Self {
        Self {
            free_by_offset: BTreeMap::new(),
            size_bins: std::array::from_fn(|_| Vec::new()),
            non_empty_bins: 0,
            total_capacity: capacity,
        }
    }

    /// Returns the number of tracked free blocks.
    #[must_use]
    pub fn free_block_count(&self) -> usize {
        self.free_by_offset.len()
    }

    /// Allocates a contiguous chunk of at least `required_size` bytes.
    ///
    /// Uses O(1) bitmask size-class lookup. Splits the chunk if leftover capacity exceeds `MIN_SPLIT_FRAGMENT`.
    /// Returns `Some((offset, allocated_size))` on success, or `None` if out of memory.
    pub fn allocate(&mut self, required_size: usize) -> Option<(usize, usize)> {
        let start_bin = bin_index_for_size(required_size);

        // 1. Check the exact size-class bin first for a fitting block (best-fit within bin).
        let mut chosen: Option<(usize, usize)> = None;
        if (self.non_empty_bins & (1u32 << start_bin)) != 0 {
            let mut best_fit: Option<(usize, usize)> = None;
            for &(cand_offset, cand_size) in &self.size_bins[start_bin] {
                if cand_size >= required_size
                    && best_fit.is_none_or(|(_, best_sz)| cand_size < best_sz)
                {
                    best_fit = Some((cand_offset, cand_size));
                    if cand_size == required_size {
                        break;
                    }
                }
            }
            chosen = best_fit;
        }

        // 2. Otherwise jump directly to the smallest non-empty larger size class in O(1).
        if chosen.is_none() && start_bin + 1 < NUM_SIZE_BINS {
            let higher_mask = self.non_empty_bins & (!0u32 << (start_bin + 1));
            if higher_mask != 0 {
                let target_bin = higher_mask.trailing_zeros() as usize;
                chosen = self.size_bins[target_bin].last().copied();
            }
        }

        let (offset, best_size) = chosen?;
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

    /// Inserts a block into both the offset index and segregated size-class bin.
    fn insert_free_block(&mut self, offset: usize, size: usize) {
        self.free_by_offset.insert(offset, size);
        let bin_idx = bin_index_for_size(size);
        self.size_bins[bin_idx].push((offset, size));
        self.non_empty_bins |= 1u32 << bin_idx;
    }

    /// Removes a block from both the offset index and segregated size-class bin.
    fn remove_free_block(&mut self, offset: usize, size: usize) {
        self.free_by_offset.remove(&offset);
        let bin_idx = bin_index_for_size(size);
        let bin = &mut self.size_bins[bin_idx];
        if let Some(pos) = bin
            .iter()
            .position(|&(entry_offset, _)| entry_offset == offset)
        {
            bin.swap_remove(pos);
        }
        if bin.is_empty() {
            self.non_empty_bins &= !(1u32 << bin_idx);
        }
    }
}
