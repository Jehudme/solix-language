//! Automatic Reference Counting (ARC) primitive mechanics.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::memory::heap::VirtualHeap;

/// Increments the reference count of the heap block at `payload_addr`.
///
/// If `payload_addr` is 0 (null pointer), this operation is a safe no-op returning 0.
///
/// # Errors
/// Returns `RuntimeError` if the reference count overflows `MAX_REF_COUNT` or address is invalid.
#[inline(always)]
pub fn inc_ref(heap: &mut VirtualHeap, payload_addr: u32) -> Result<u32, RuntimeError> {
    if payload_addr == 0 {
        return Ok(0);
    }
    let mut header = heap.read_header(payload_addr)?;
    let new_rc = header.inc_ref()?;
    heap.write_header(payload_addr, header)?;
    Ok(new_rc)
}

/// Decrements the reference count of the heap block at `payload_addr`.
///
/// If `payload_addr` is 0 (null pointer), this operation is a safe no-op returning 0.
///
/// # Errors
/// Returns `RuntimeError` if the address is invalid.
#[inline(always)]
pub fn dec_ref(heap: &mut VirtualHeap, payload_addr: u32) -> Result<u32, RuntimeError> {
    if payload_addr == 0 {
        return Ok(0);
    }
    let mut header = heap.read_header(payload_addr)?;
    let new_rc = header.dec_ref();
    if new_rc > 0 {
        heap.write_header(payload_addr, header)?;
    }
    Ok(new_rc)
}
