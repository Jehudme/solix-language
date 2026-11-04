//! Virtual heap implementation with partitioned static and dynamic address spaces.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::memory::deallocator::FreeListManager;
use crate::memory::layout::{BlockHeader, BlockTag, HEADER_SIZE, align_up};
use std::collections::BTreeMap;
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

/// Byte-addressed virtual memory buffer partitioned into static and dynamic regions.
#[derive(Debug, Clone)]
pub struct VirtualHeap {
    /// Raw byte buffer holding static variables and dynamic heap blocks.
    buffer: Vec<u8>,
    /// Byte capacity reserved for static variables at the base of virtual memory.
    static_size: usize,
    /// Dynamic heap byte capacity following the static partition.
    dynamic_capacity: usize,
    /// Segregated free list tracking unallocated dynamic blocks.
    free_list: FreeListManager,
    /// Active allocations indexed by their 32-bit payload addresses.
    live_allocations: BTreeMap<u32, LiveAllocationInfo>,
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
            free_list: FreeListManager::new(dynamic_capacity),
            live_allocations: BTreeMap::new(),
        }
    }

    /// Resizes or reserves the static partition at the base of virtual memory.
    pub fn reserve_static_partition(&mut self, static_slots: u32) {
        let new_static_size = align_up(static_slots as usize * 8);
        if new_static_size != self.static_size {
            let total_size = new_static_size + self.dynamic_capacity;
            self.buffer = vec![0u8; total_size];
            self.static_size = new_static_size;
            self.free_list = FreeListManager::new(self.dynamic_capacity);
            self.live_allocations.clear();
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
        self.live_allocations.len()
    }

    // =========================================================================
    // Dynamic Allocation & Deallocation
    // =========================================================================

    /// Allocates a new dynamic block of `payload_size` bytes with the given `tag`.
    ///
    /// # Errors
    /// Returns `RuntimeError::OutOfMemory` if insufficient free heap space exists.
    pub fn allocate(&mut self, payload_size: usize, tag: BlockTag) -> Result<u32, RuntimeError> {
        let aligned_payload = align_up(payload_size);
        let required_block_size = HEADER_SIZE + aligned_payload;

        let (block_offset, allocated_block_size) = self
            .free_list
            .allocate(required_block_size)
            .ok_or(RuntimeError::OutOfMemory {
                requested: required_block_size,
                available: self.dynamic_capacity,
            })?;

        let block_abs_addr = self.static_size + block_offset;
        let payload_abs_addr = block_abs_addr + HEADER_SIZE;

        // Initialize header
        let header = BlockHeader::new(payload_size as u32, tag, 1);
        let header_bytes = header.to_bytes();
        self.buffer[block_abs_addr..block_abs_addr + HEADER_SIZE].copy_from_slice(&header_bytes);

        // Zero out payload bytes
        let payload_end = block_abs_addr + allocated_block_size;
        self.buffer[payload_abs_addr..payload_end].fill(0);

        // Record live allocation
        let payload_addr_u32 = payload_abs_addr as u32;
        self.live_allocations.insert(
            payload_addr_u32,
            LiveAllocationInfo {
                block_offset,
                total_block_size: allocated_block_size,
                payload_size,
                tag,
            },
        );

        Ok(payload_addr_u32)
    }

    /// Deallocates a heap block given its payload address.
    ///
    /// # Errors
    /// Returns `RuntimeError::NullReferenceAccess` if the address is 0 or unrecognized.
    pub fn deallocate(&mut self, payload_addr: u32) -> Result<(), RuntimeError> {
        if payload_addr == 0 {
            return Err(RuntimeError::NullReferenceAccess);
        }

        let info = self
            .live_allocations
            .remove(&payload_addr)
            .ok_or(RuntimeError::NullReferenceAccess)?;

        let block_abs_addr = self.static_size + info.block_offset;
        let block_end = block_abs_addr + info.total_block_size;

        // Clear payload and header memory to scrub remnants
        self.buffer[block_abs_addr..block_end].fill(0);

        // Return chunk to the free list and coalesce
        self.free_list
            .deallocate(info.block_offset, info.total_block_size);

        Ok(())
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

    /// Verifies that all allocated heap blocks have been cleanly freed and coalesced.
    ///
    /// # Errors
    /// Returns `RuntimeError::MemoryLeak` detailing leaked addresses and block tags
    /// if any allocations remain active or if the heap is fragmented.
    pub fn verify_clean_memory(&self) -> Result<(), RuntimeError> {
        if self.live_allocations.is_empty() && self.free_list.is_fully_coalesced() {
            return Ok(());
        }

        let mut report = String::new();
        let unallocated_count = self.live_allocations.len();

        let _ = writeln!(
            report,
            "Found {unallocated_count} live uncollected heap allocations:"
        );
        for (&addr, info) in &self.live_allocations {
            let _ = writeln!(
                report,
                "  - Address 0x{addr:08X}: Tag={:?}, PayloadSize={}B, TotalBlockSize={}B",
                info.tag, info.payload_size, info.total_block_size
            );
        }

        if !self.free_list.is_fully_coalesced() {
            let _ = writeln!(
                report,
                "Free-list is fragmented: {} free blocks remaining instead of 1 coalesced block.",
                self.free_list.free_by_offset.len()
            );
        }

        Err(RuntimeError::MemoryLeak {
            unallocated_count,
            report,
        })
    }
}
