//! Byte-addressed virtual memory subsystem and segregated free-list allocator.

#![forbid(unsafe_code)]

pub mod arc;
pub mod deallocator;
pub mod heap;
pub mod layout;

pub use arc::{dec_ref, inc_ref};
pub use deallocator::FreeListManager;
pub use heap::{HeapTelemetry, LiveAllocationInfo, VirtualHeap};
pub use layout::{ALIGNMENT, BlockHeader, BlockTag, HEADER_SIZE, MAX_REF_COUNT, align_up};
