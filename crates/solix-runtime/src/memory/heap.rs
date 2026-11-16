//! Virtual heap implementation with partitioned static and dynamic address spaces.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::memory::deallocator::FreeListManager;
use crate::memory::layout::{ALIGNMENT, BlockHeader, BlockTag, HEADER_SIZE, align_up};
use std::fmt::Write;

/// Allocation metadata record maintained for heap integrity and leak auditing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveAllocationInfo {
    /// Byte offset within the dynamic heap space.
    pub block_offset: usize,
    /// Total block capacity including the 8-byte header and padding.
    pub total_block_size: usize,
    /// Usable payload size requested by the allocator.
    pub payload_size: usize,
    /// Memory block classification tag.
    pub tag: BlockTag,
}

/// Telemetry metrics recorded across virtual heap memory operations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HeapTelemetry {
    /// Currently allocated dynamic block bytes.
    pub current_allocated_bytes: usize,
    /// Peak allocated dynamic block bytes recorded.
    pub peak_allocated_bytes: usize,
    /// Total number of dynamic allocation operations processed.
    pub total_allocations_count: u64,
    /// Total number of dynamic deallocation operations processed.
    pub total_deallocations_count: u64,
    /// Number of active live objects.
    pub active_live_objects: usize,
}

/// Byte-addressed virtual memory buffer partitioned into static and dynamic regions.
#[derive(Debug, Clone)]
pub struct VirtualHeap {
    /// Raw byte buffer holding static variables and dynamic heap blocks.
    pub buffer: Vec<u8>,
    /// Byte capacity reserved for static variables at the base of virtual memory.
    static_size: usize,
    /// Dynamic heap byte capacity following the static partition.
    dynamic_capacity: usize,
    /// Segregated free list tracking unallocated dynamic blocks.
    pub free_list: FreeListManager,
    /// Currently allocated dynamic bytes.
    current_allocated_bytes: usize,
    /// Peak dynamic bytes allocated over the lifetime of the heap.
    peak_allocated_bytes: usize,
    /// Total count of allocations executed.
    total_allocations_count: u64,
    /// Total count of deallocations executed.
    total_deallocations_count: u64,
    /// Legacy safe mode flag (kept for compatibility).
    pub safe_mode: bool,
    /// Live allocation counter for O(1) allocation tracking.
    pub active_live_objects: usize,
    /// Monotonically advancing bump allocation cursor.
    pub bump_cursor: usize,
    /// O(1) intrusive quick-lists for size classes 8B..256B in 8-byte increments.
    pub quick_lists: [u32; 32],
}

impl VirtualHeap {
    /// Creates a new virtual heap with the specified static slot count and dynamic capacity.
    #[must_use]
    pub fn new(static_slots: u32, dynamic_capacity: usize) -> Self {
        let static_size = align_up(static_slots as usize * 8);
        let dynamic_capacity = align_up(dynamic_capacity);
        let total_size = static_size + dynamic_capacity;

        Self {
            buffer: vec![0u8; total_size],
            static_size,
            dynamic_capacity,
            free_list: FreeListManager::new_empty(dynamic_capacity),
            current_allocated_bytes: 0,
            peak_allocated_bytes: 0,
            total_allocations_count: 0,
            total_deallocations_count: 0,
            safe_mode: false,
            active_live_objects: 0,
            bump_cursor: 0,
            quick_lists: [0u32; 32],
        }
    }

    /// Sets whether safe mode tracking is enabled.
    pub fn set_safe_mode(&mut self, safe_mode: bool) {
        self.safe_mode = safe_mode;
    }

    /// Builder method to set safe mode.
    #[must_use]
    pub fn with_safe_mode(mut self, safe_mode: bool) -> Self {
        self.safe_mode = safe_mode;
        self
    }

    /// Resizes or reserves the static partition at the base of virtual memory.
    pub fn reserve_static_partition(&mut self, static_slots: u32) {
        let new_static_size = align_up(static_slots as usize * 8);
        if new_static_size != self.static_size {
            let total_size = new_static_size + self.dynamic_capacity;
            self.buffer = vec![0u8; total_size];
            self.static_size = new_static_size;
            self.free_list = FreeListManager::new_empty(self.dynamic_capacity);
            self.bump_cursor = 0;
            self.quick_lists = [0u32; 32];
            self.active_live_objects = 0;
        }
    }

    /// Returns the static region size in bytes.
    #[must_use]
    pub const fn static_size(&self) -> usize {
        self.static_size
    }

    /// Returns the dynamic heap capacity in bytes.
    #[must_use]
    pub const fn dynamic_capacity(&self) -> usize {
        self.dynamic_capacity
    }

    /// Number of currently active, unfreed heap allocations.
    #[must_use]
    pub fn live_objects_count(&self) -> usize {
        self.active_live_objects
    }

    // =========================================================================
    // Dynamic Allocation & Deallocation
    // =========================================================================

    /// Allocates a new dynamic block of `payload_size` bytes with the given `tag`.
    ///
    /// # Errors
    /// Returns `RuntimeError::OutOfMemory` if insufficient free heap space exists.
    pub fn allocate(&mut self, payload_size: usize, tag: BlockTag) -> Result<u32, RuntimeError> {
        let aligned_payload = align_up(payload_size).max(ALIGNMENT);
        let required_block_size = HEADER_SIZE + aligned_payload;

        if required_block_size > self.dynamic_capacity {
            return Err(RuntimeError::OutOfMemory {
                requested: required_block_size,
                available: self.dynamic_capacity,
            });
        }

        let (payload_abs_addr, allocated_bytes): (u32, usize) =
            if required_block_size <= 256 && self.quick_lists[(required_block_size / 8) - 1] != 0 {
                let class_index = (required_block_size / 8) - 1;
                let cached_payload = self.quick_lists[class_index];
                // O(1) Quick-list pop
                let next_cached = self.read_u32(cached_payload)?;
                self.quick_lists[class_index] = next_cached;

                let block_abs_addr = cached_payload as usize - HEADER_SIZE;
                let header = BlockHeader::new(payload_size as u32, tag, 1);
                let header_bytes = header.to_bytes();
                self.buffer[block_abs_addr..block_abs_addr + HEADER_SIZE]
                    .copy_from_slice(&header_bytes);

                let payload_end = block_abs_addr + required_block_size;
                self.buffer[cached_payload as usize..payload_end].fill(0);

                (cached_payload, required_block_size)
            } else if self.bump_cursor + required_block_size <= self.dynamic_capacity {
                // O(1) Bump allocation for any block size fitting the remaining bump capacity
                let block_offset = self.bump_cursor;
                let block_abs_addr = self.static_size + block_offset;
                let payload_addr = (block_abs_addr + HEADER_SIZE) as u32;
                self.bump_cursor += required_block_size;

                let header = BlockHeader::new(payload_size as u32, tag, 1);
                let header_bytes = header.to_bytes();
                self.buffer[block_abs_addr..block_abs_addr + HEADER_SIZE]
                    .copy_from_slice(&header_bytes);

                let payload_end = block_abs_addr + required_block_size;
                self.buffer[payload_addr as usize..payload_end].fill(0);

                (payload_addr, required_block_size)
            } else {
                // Bump pointer exhausted. Flush un-bumped remainder to free_list and flush quick lists.
                if self.bump_cursor < self.dynamic_capacity {
                    let remaining_size = self.dynamic_capacity - self.bump_cursor;
                    self.free_list.deallocate(self.bump_cursor, remaining_size);
                    self.bump_cursor = self.dynamic_capacity;
                }
                self.flush_quick_lists_to_free_list();
                self.allocate_from_free_list(required_block_size, tag)?
            };

        self.total_allocations_count += 1;
        self.active_live_objects += 1;
        self.current_allocated_bytes += allocated_bytes;
        if self.current_allocated_bytes > self.peak_allocated_bytes {
            self.peak_allocated_bytes = self.current_allocated_bytes;
        }

        Ok(payload_abs_addr)
    }

    /// Allocates from `FreeListManager` when quick lists and bump allocation are exhausted.
    fn allocate_from_free_list(
        &mut self,
        required_block_size: usize,
        tag: BlockTag,
    ) -> Result<(u32, usize), RuntimeError> {
        let (block_offset, allocated_block_size) = self
            .free_list
            .allocate(required_block_size)
            .ok_or(RuntimeError::OutOfMemory {
                requested: required_block_size,
                available: self
                    .dynamic_capacity
                    .saturating_sub(self.current_allocated_bytes),
            })?;

        let block_abs_addr = self.static_size + block_offset;
        let payload_abs_addr = (block_abs_addr + HEADER_SIZE) as u32;

        let header = BlockHeader::new((allocated_block_size - HEADER_SIZE) as u32, tag, 1);
        let header_bytes = header.to_bytes();
        self.buffer[block_abs_addr..block_abs_addr + HEADER_SIZE].copy_from_slice(&header_bytes);

        let payload_end = block_abs_addr + allocated_block_size;
        self.buffer[payload_abs_addr as usize..payload_end].fill(0);

        Ok((payload_abs_addr, allocated_block_size))
    }

    /// Deallocates a heap block given its payload address.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the address is 0 or unrecognized.
    pub fn deallocate(&mut self, payload_addr: u32) -> Result<(), RuntimeError> {
        if payload_addr == 0 {
            return Err(RuntimeError::NullReferenceAccess);
        }
        let addr = payload_addr as usize;
        if addr < self.static_size + HEADER_SIZE || addr > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }

        let header = self.read_header(payload_addr)?;
        let total_block_size = HEADER_SIZE + align_up(header.size as usize).max(ALIGNMENT);
        let block_abs_addr = addr - HEADER_SIZE;
        let block_offset = block_abs_addr - self.static_size;

        if header.ref_count == 0 {
            return Err(RuntimeError::NullReferenceAccess);
        }

        // Clear payload and mark header dead
        let dead_header = BlockHeader::new(header.size, BlockTag::Raw, 0);
        let dead_bytes = dead_header.to_bytes();
        self.buffer[block_abs_addr..block_abs_addr + HEADER_SIZE].copy_from_slice(&dead_bytes);

        if total_block_size <= 256 {
            // O(1) Quick-list push
            let class_index = (total_block_size / 8) - 1;
            let current_head = self.quick_lists[class_index];
            self.write_u32(payload_addr, current_head)?;
            self.quick_lists[class_index] = payload_addr;
        } else {
            self.free_list.deallocate(block_offset, total_block_size);
        }

        self.total_deallocations_count += 1;
        self.active_live_objects = self.active_live_objects.saturating_sub(1);
        self.current_allocated_bytes = self
            .current_allocated_bytes
            .saturating_sub(total_block_size);

        Ok(())
    }

    /// Fast inline deallocation for POD blocks where ref count has reached 0.
    ///
    /// Skips cascading destructor checks and pushes directly to quick lists or free list.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the address is invalid.
    #[inline(always)]
    pub fn deallocate_pod_fast(
        &mut self,
        payload_addr: u32,
        size: u32,
    ) -> Result<(), RuntimeError> {
        if payload_addr == 0 {
            return Err(RuntimeError::NullReferenceAccess);
        }
        let addr_usize = payload_addr as usize;
        if addr_usize < self.static_size + HEADER_SIZE || addr_usize > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }

        let total_block_size = HEADER_SIZE + align_up(size as usize).max(ALIGNMENT);
        let block_abs_addr = addr_usize - HEADER_SIZE;
        let block_offset = block_abs_addr - self.static_size;

        // Clear payload and mark header dead
        let dead_header = BlockHeader::new(size, BlockTag::Raw, 0);
        let dead_bytes = dead_header.to_bytes();
        self.buffer[block_abs_addr..block_abs_addr + HEADER_SIZE].copy_from_slice(&dead_bytes);

        if total_block_size <= 256 {
            // O(1) Quick-list push
            let class_index = (total_block_size / 8) - 1;
            let current_head = self.quick_lists[class_index];
            self.write_u32(payload_addr, current_head)?;
            self.quick_lists[class_index] = payload_addr;
        } else {
            self.free_list.deallocate(block_offset, total_block_size);
        }

        self.total_deallocations_count += 1;
        self.active_live_objects = self.active_live_objects.saturating_sub(1);
        self.current_allocated_bytes = self
            .current_allocated_bytes
            .saturating_sub(total_block_size);

        Ok(())
    }

    /// Flushes all entries cached in `quick_lists` into `FreeListManager`.
    pub fn flush_quick_lists_to_free_list(&mut self) {
        for class_index in 0..32 {
            let mut current_payload = self.quick_lists[class_index];
            self.quick_lists[class_index] = 0;
            let block_size = (class_index + 1) * 8;

            while current_payload != 0 {
                let next_payload = self.read_u32(current_payload).unwrap_or(0);
                let block_abs_addr = current_payload as usize - HEADER_SIZE;
                if block_abs_addr >= self.static_size {
                    let block_offset = block_abs_addr - self.static_size;
                    self.free_list.deallocate(block_offset, block_size);
                }
                current_payload = next_payload;
            }
        }
    }

    /// Returns the live telemetry metrics of the virtual heap.
    #[must_use]
    pub fn telemetry(&self) -> HeapTelemetry {
        HeapTelemetry {
            current_allocated_bytes: self.current_allocated_bytes,
            peak_allocated_bytes: self.peak_allocated_bytes,
            total_allocations_count: self.total_allocations_count,
            total_deallocations_count: self.total_deallocations_count,
            active_live_objects: self.live_objects_count(),
        }
    }

    // =========================================================================
    // Header Inspection & Mutation
    // =========================================================================

    /// Reads the `BlockHeader` preceding the payload at `payload_addr`.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the address is invalid.
    pub fn read_header(&self, payload_addr: u32) -> Result<BlockHeader, RuntimeError> {
        let addr = payload_addr as usize;
        if addr < self.static_size + HEADER_SIZE || addr > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        let header_start = addr - HEADER_SIZE;
        let mut bytes = [0u8; HEADER_SIZE];
        bytes.copy_from_slice(&self.buffer[header_start..header_start + HEADER_SIZE]);
        Ok(BlockHeader::from_bytes(bytes))
    }

    /// Writes updated `BlockHeader` metadata preceding the payload at `payload_addr`.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the address is invalid.
    pub fn write_header(
        &mut self,
        payload_addr: u32,
        header: BlockHeader,
    ) -> Result<(), RuntimeError> {
        let addr = payload_addr as usize;
        if addr < self.static_size + HEADER_SIZE || addr > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        let header_start = addr - HEADER_SIZE;
        let bytes = header.to_bytes();
        self.buffer[header_start..header_start + HEADER_SIZE].copy_from_slice(&bytes);
        Ok(())
    }

    // =========================================================================
    // Static Memory Accessors
    // =========================================================================

    /// Reads a 64-bit integer from a static slot.
    ///
    /// # Errors
    /// Returns `RuntimeError::InvalidStaticOffset` if the slot exceeds reserved static memory.
    pub fn read_static_u64(&self, slot: u32) -> Result<u64, RuntimeError> {
        let offset = slot as usize * 8;
        if offset + 8 > self.static_size {
            return Err(RuntimeError::InvalidStaticOffset {
                offset: slot,
                total_static: (self.static_size / 8) as u32,
            });
        }
        let bytes = [
            self.buffer[offset],
            self.buffer[offset + 1],
            self.buffer[offset + 2],
            self.buffer[offset + 3],
            self.buffer[offset + 4],
            self.buffer[offset + 5],
            self.buffer[offset + 6],
            self.buffer[offset + 7],
        ];
        Ok(u64::from_le_bytes(bytes))
    }

    /// Writes a 64-bit integer to a static slot.
    ///
    /// # Errors
    /// Returns `RuntimeError::InvalidStaticOffset` if the slot exceeds reserved static memory.
    pub fn write_static_u64(&mut self, slot: u32, val: u64) -> Result<(), RuntimeError> {
        let offset = slot as usize * 8;
        if offset + 8 > self.static_size {
            return Err(RuntimeError::InvalidStaticOffset {
                offset: slot,
                total_static: (self.static_size / 8) as u32,
            });
        }
        let bytes = val.to_le_bytes();
        self.buffer[offset..offset + 8].copy_from_slice(&bytes);
        Ok(())
    }

    // =========================================================================
    // Dynamic Memory Accessors
    // =========================================================================

    /// Reads a single byte at virtual byte address `addr`.
    pub fn read_u8(&self, addr: u32) -> Result<u8, RuntimeError> {
        let pos = addr as usize;
        self.buffer
            .get(pos)
            .copied()
            .ok_or(RuntimeError::NullReferenceAccess)
    }

    /// Writes a single byte at virtual byte address `addr`.
    pub fn write_u8(&mut self, addr: u32, val: u8) -> Result<(), RuntimeError> {
        let pos = addr as usize;
        let cell = self
            .buffer
            .get_mut(pos)
            .ok_or(RuntimeError::NullReferenceAccess)?;
        *cell = val;
        Ok(())
    }

    /// Reads a little-endian 16-bit unsigned integer at virtual byte address `addr`.
    pub fn read_u16(&self, addr: u32) -> Result<u16, RuntimeError> {
        let pos = addr as usize;
        if pos + 2 > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        let bytes = [self.buffer[pos], self.buffer[pos + 1]];
        Ok(u16::from_le_bytes(bytes))
    }

    /// Writes a little-endian 16-bit unsigned integer at virtual byte address `addr`.
    pub fn write_u16(&mut self, addr: u32, word_val: u16) -> Result<(), RuntimeError> {
        let pos = addr as usize;
        if pos + 2 > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        self.buffer[pos..pos + 2].copy_from_slice(&word_val.to_le_bytes());
        Ok(())
    }

    /// Reads a little-endian 32-bit unsigned integer at virtual byte address `addr`.
    pub fn read_u32(&self, addr: u32) -> Result<u32, RuntimeError> {
        let pos = addr as usize;
        if pos + 4 > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        let bytes = [
            self.buffer[pos],
            self.buffer[pos + 1],
            self.buffer[pos + 2],
            self.buffer[pos + 3],
        ];
        Ok(u32::from_le_bytes(bytes))
    }

    /// Writes a little-endian 32-bit unsigned integer at virtual byte address `addr`.
    pub fn write_u32(&mut self, addr: u32, val: u32) -> Result<(), RuntimeError> {
        let pos = addr as usize;
        if pos + 4 > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        self.buffer[pos..pos + 4].copy_from_slice(&val.to_le_bytes());
        Ok(())
    }

    /// Reads a little-endian 64-bit unsigned integer at virtual byte address `addr`.
    pub fn read_u64(&self, addr: u32) -> Result<u64, RuntimeError> {
        let pos = addr as usize;
        if pos + 8 > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        let bytes = [
            self.buffer[pos],
            self.buffer[pos + 1],
            self.buffer[pos + 2],
            self.buffer[pos + 3],
            self.buffer[pos + 4],
            self.buffer[pos + 5],
            self.buffer[pos + 6],
            self.buffer[pos + 7],
        ];
        Ok(u64::from_le_bytes(bytes))
    }

    /// Writes a little-endian 64-bit unsigned integer at virtual byte address `addr`.
    pub fn write_u64(&mut self, addr: u32, val: u64) -> Result<(), RuntimeError> {
        let pos = addr as usize;
        if pos + 8 > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        self.buffer[pos..pos + 8].copy_from_slice(&val.to_le_bytes());
        Ok(())
    }

    /// Reads a contiguous byte slice from virtual byte address `addr`.
    pub fn read_bytes(&self, addr: u32, len: usize) -> Result<&[u8], RuntimeError> {
        let pos = addr as usize;
        if pos + len > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        Ok(&self.buffer[pos..pos + len])
    }

    /// Writes a contiguous byte slice to virtual byte address `addr`.
    pub fn write_bytes(&mut self, addr: u32, bytes: &[u8]) -> Result<(), RuntimeError> {
        let pos = addr as usize;
        if pos + bytes.len() > self.buffer.len() {
            return Err(RuntimeError::NullReferenceAccess);
        }
        self.buffer[pos..pos + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    /// Reads a UTF-8 string from virtual byte address `addr` using the block header size.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the address or length is invalid.
    pub fn read_string(&self, addr: u32) -> Result<String, RuntimeError> {
        let header = self.read_header(addr)?;
        let bytes = self.read_bytes(addr, header.size as usize)?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    // =========================================================================
    // Memory Integrity & Clean Verification
    // =========================================================================

    /// Returns the live block counts partitioned by tag: (strings, arrays, objects, closures).
    #[must_use]
    pub fn count_blocks_by_tag(&self) -> (usize, usize, usize, usize) {
        let mut string_count = 0;
        let mut array_count = 0;
        let mut object_count = 0;
        let mut closure_count = 0;
        let mut current_offset = 0;
        while current_offset < self.bump_cursor {
            let block_abs_addr = self.static_size + current_offset;
            let payload_addr = (block_abs_addr + HEADER_SIZE) as u32;
            if let Ok(header) = self.read_header(payload_addr) {
                let total_block_size = HEADER_SIZE + align_up(header.size as usize).max(ALIGNMENT);
                if total_block_size == 0 {
                    break;
                }
                if header.ref_count > 0 {
                    match header.tag {
                        BlockTag::String => string_count += 1,
                        BlockTag::ArrayRef => array_count += 1,
                        BlockTag::Object => object_count += 1,
                        BlockTag::Closure => closure_count += 1,
                        _ => {}
                    }
                }
                current_offset += total_block_size;
            } else {
                break;
            }
        }
        (string_count, array_count, object_count, closure_count)
    }

    /// Verifies that all allocated heap blocks have been cleanly freed and coalesced.
    ///
    /// # Errors
    /// Returns `RuntimeError::MemoryLeak` detailing leaked addresses and block tags
    /// if any allocations remain active or if the heap is fragmented.
    pub fn verify_clean_memory(&mut self) -> Result<(), RuntimeError> {
        let unallocated_count = self.active_live_objects;

        if unallocated_count > 0 {
            let mut report = String::new();
            let _ = writeln!(
                report,
                "Found {unallocated_count} live uncollected heap allocations:"
            );

            let mut current_offset = 0;
            while current_offset < self.bump_cursor {
                let block_abs_addr = self.static_size + current_offset;
                let payload_addr = (block_abs_addr + HEADER_SIZE) as u32;
                if let Ok(header) = self.read_header(payload_addr) {
                    let total_block_size =
                        HEADER_SIZE + align_up(header.size as usize).max(ALIGNMENT);
                    if total_block_size == 0 {
                        break;
                    }
                    if header.ref_count > 0 {
                        let _ = writeln!(
                            report,
                            "  - Address 0x{payload_addr:08X}: Tag={:?}, PayloadSize={}B, TotalBlockSize={}B",
                            header.tag, header.size, total_block_size
                        );
                    }
                    current_offset += total_block_size;
                } else {
                    break;
                }
            }

            return Err(RuntimeError::MemoryLeak {
                unallocated_count,
                report,
            });
        }

        // Flush all quick-lists into FreeListManager
        self.flush_quick_lists_to_free_list();

        // Flush remaining un-bumped space into FreeListManager
        if self.bump_cursor < self.dynamic_capacity {
            let remaining_size = self.dynamic_capacity - self.bump_cursor;
            self.free_list.deallocate(self.bump_cursor, remaining_size);
            self.bump_cursor = self.dynamic_capacity;
        }

        if !self.free_list.is_fully_coalesced() {
            let mut report = String::new();
            let _ = writeln!(
                report,
                "Free-list is fragmented: {} free blocks remaining instead of 1 coalesced block.",
                self.free_list.free_by_offset.len()
            );
            return Err(RuntimeError::MemoryLeak {
                unallocated_count: 0,
                report,
            });
        }

        Ok(())
    }
}
