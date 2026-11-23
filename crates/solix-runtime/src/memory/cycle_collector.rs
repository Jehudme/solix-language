//! Bacon & Rajan (2001) Synchronous Cycle Collector for ARC memory management.

#![forbid(unsafe_code)]

use crate::context::RuntimeContext;
use crate::exceptions::RuntimeError;
use crate::memory::heap::VirtualHeap;
use crate::memory::layout::{BlockTag, GcColor, MAX_REF_COUNT, can_contain_references};
use crate::value::model::Value;
use std::collections::HashSet;
use std::time::Instant;

/// Extracts all valid, outgoing child reference heap pointers from the given block.
pub fn get_outgoing_references(
    heap: &VirtualHeap,
    context: &RuntimeContext,
    address: u32,
) -> Vec<u32> {
    if address == 0 {
        return Vec::new();
    }
    let Ok(header) = heap.read_header(address) else {
        return Vec::new();
    };
    if header.ref_count == MAX_REF_COUNT || !can_contain_references(header.tag) {
        return Vec::new();
    }

    let mut children = Vec::new();
    match header.tag {
        BlockTag::Object => {
            let vtable_identifier = heap.read_u32(address).unwrap_or(0);
            if let Some(class_item) = context.get_class_by_vtable_id(vtable_identifier) {
                for field_offset in &class_item.reference_field_offsets {
                    let field_address = address + *field_offset as u32;
                    let raw_field_value = heap.read_u64(field_address).unwrap_or(0);
                    if let Value::Ref(child_address) = Value::from_raw_u64(raw_field_value)
                        && child_address > 0
                    {
                        children.push(child_address);
                    }
                }
            }
        }
        BlockTag::ArrayRef => {
            let element_count = heap.read_u32(address).unwrap_or(0);
            let element_type_tag = heap.read_u8(address + 7).unwrap_or(0);
            if element_type_tag == 13 || element_type_tag >= 0x80 {
                let element_byte_size = heap.read_u8(address + 4).unwrap_or(8) as usize;
                for element_index in 0..element_count {
                    let element_slot_address =
                        address + 8 + (element_index as usize * element_byte_size) as u32;
                    let child_address = heap.read_u64(element_slot_address).unwrap_or(0) as u32;
                    if child_address > 0 {
                        children.push(child_address);
                    }
                }
            }
        }
        BlockTag::Tuple => {
            let tuple_item_count = heap.read_u32(address).unwrap_or(0);
            for tuple_index in 0..tuple_item_count {
                let tuple_slot_address = address + 8 + (tuple_index as usize * 16) as u32;
                let raw_type_tag = heap.read_u64(tuple_slot_address).unwrap_or(0);
                let raw_payload = heap.read_u64(tuple_slot_address + 8).unwrap_or(0);
                let element_value = Value::from_tagged_parts(raw_type_tag, raw_payload);
                if let Value::Ref(child_address) = element_value
                    && child_address > 0
                {
                    children.push(child_address);
                }
            }
        }
        BlockTag::Closure => {
            let captured_variable_count = heap.read_u32(address + 4).unwrap_or(0);
            for capture_index in 0..captured_variable_count {
                let capture_slot_address = address + 8 + (capture_index as usize * 16) as u32;
                let raw_type_tag = heap.read_u64(capture_slot_address).unwrap_or(0);
                let raw_payload = heap.read_u64(capture_slot_address + 8).unwrap_or(0);
                let captured_value = Value::from_tagged_parts(raw_type_tag, raw_payload);
                if let Value::Ref(child_address) = captured_value
                    && child_address > 0
                {
                    children.push(child_address);
                }
            }
        }
        _ => {}
    }

    children
}

/// Phase 1: MarkGray (Trial Deletion)
///
/// Iteratively simulates removing candidate roots by decrementing reference counts
/// on internal outgoing pointers, painting traversed nodes Gray.
pub fn mark_gray(
    heap: &mut VirtualHeap,
    context: &RuntimeContext,
    start_address: u32,
) -> Result<(), RuntimeError> {
    let mut worklist = vec![start_address];
    while let Some(current_address) = worklist.pop() {
        let mut header = heap.read_header(current_address)?;
        if header.color != GcColor::Gray {
            header.color = GcColor::Gray;
            heap.write_header(current_address, header)?;

            let child_pointers = get_outgoing_references(heap, context, current_address);
            for child_address in child_pointers {
                let mut child_header = heap.read_header(child_address)?;
                if child_header.ref_count != MAX_REF_COUNT && child_header.ref_count > 0 {
                    child_header.ref_count -= 1;
                    heap.write_header(child_address, child_header)?;
                }
                if child_header.color != GcColor::Gray {
                    worklist.push(child_address);
                }
            }
        }
    }
    Ok(())
}

/// Phase 2: Scan (Detect True Cycles)
///
/// Scans the candidate subgraph iteratively. If reference count > 0, an external reference exists,
/// so node and reachable children are repainted Black and counts are restored.
/// If reference count == 0, paints node White (confirmed garbage cycle).
pub fn scan(
    heap: &mut VirtualHeap,
    context: &RuntimeContext,
    start_address: u32,
) -> Result<(), RuntimeError> {
    let mut worklist = vec![start_address];
    while let Some(current_address) = worklist.pop() {
        let mut header = heap.read_header(current_address)?;
        if header.color == GcColor::Gray {
            if header.ref_count > 0 {
                scan_black(heap, context, current_address)?;
            } else {
                header.color = GcColor::White;
                heap.write_header(current_address, header)?;

                let child_pointers = get_outgoing_references(heap, context, current_address);
                for child_address in child_pointers {
                    worklist.push(child_address);
                }
            }
        }
    }
    Ok(())
}

/// Restores reference counts decremented during trial deletion and repaints nodes Black iteratively.
pub fn scan_black(
    heap: &mut VirtualHeap,
    context: &RuntimeContext,
    start_address: u32,
) -> Result<(), RuntimeError> {
    let mut worklist = vec![start_address];
    let mut header = heap.read_header(start_address)?;
    header.color = GcColor::Black;
    heap.write_header(start_address, header)?;

    while let Some(current_address) = worklist.pop() {
        let child_pointers = get_outgoing_references(heap, context, current_address);
        for child_address in child_pointers {
            let mut child_header = heap.read_header(child_address)?;
            if child_header.ref_count != MAX_REF_COUNT {
                child_header.ref_count += 1;
                heap.write_header(child_address, child_header)?;
            }
            if child_header.color != GcColor::Black {
                child_header.color = GcColor::Black;
                heap.write_header(child_address, child_header)?;
                worklist.push(child_address);
            }
        }
    }
    Ok(())
}

/// Phase 3: CollectWhite (Reclamation)
///
/// Sweeps confirmed White cyclic garbage iteratively, repainting nodes Black and returning heap memory.
pub fn collect_white(
    heap: &mut VirtualHeap,
    context: &RuntimeContext,
    start_address: u32,
    freed_addresses: &mut HashSet<u32>,
) -> Result<u64, RuntimeError> {
    let mut worklist = vec![start_address];
    let mut to_deallocate = Vec::new();

    while let Some(current_address) = worklist.pop() {
        if freed_addresses.contains(&current_address) {
            continue;
        }
        let Ok(mut header) = heap.read_header(current_address) else {
            continue;
        };
        if header.color == GcColor::White {
            header.color = GcColor::Black;
            heap.write_header(current_address, header)?;
            freed_addresses.insert(current_address);
            to_deallocate.push(current_address);

            let child_pointers = get_outgoing_references(heap, context, current_address);
            for child_address in child_pointers {
                if !freed_addresses.contains(&child_address) {
                    worklist.push(child_address);
                }
            }
        }
    }

    let mut collected_total = 0u64;
    for block_address in to_deallocate {
        heap.force_deallocate(block_address)?;
        collected_total += 1;
    }
    Ok(collected_total)
}

/// Orchestrates Bacon & Rajan (2001) Synchronous Cycle Collection across suspect purple roots.
///
/// Returns `(collected_count, deferred_roots)`.
pub fn run_cycle_collection(
    heap: &mut VirtualHeap,
    context: &RuntimeContext,
    roots: &[u32],
    budget_microseconds: Option<u64>,
) -> Result<(u64, Vec<u32>), RuntimeError> {
    let collection_start_time = Instant::now();
    let mut valid_roots = Vec::new();

    for candidate_address in roots {
        let address = *candidate_address;
        if address == 0 {
            continue;
        }
        let Ok(header) = heap.read_header(address) else {
            continue;
        };
        if header.color == GcColor::Purple
            && header.ref_count > 0
            && can_contain_references(header.tag)
        {
            valid_roots.push(address);
        } else if header.color == GcColor::Black && header.ref_count == 0 {
            let _ = heap.force_deallocate(address);
        }
    }

    let mut deferred_roots = Vec::new();
    let mut roots_to_process = Vec::new();

    for (root_index, root_address) in valid_roots.iter().enumerate() {
        if let Some(budget_limit) = budget_microseconds
            && collection_start_time.elapsed().as_micros() as u64 >= budget_limit
        {
            deferred_roots.extend_from_slice(&valid_roots[root_index..]);
            eprintln!(
                "[W0340] Cycle collector exceeded time slice budget of {} us; deferred {} roots to next cycle.",
                budget_limit,
                deferred_roots.len()
            );
            break;
        }
        roots_to_process.push(*root_address);
    }

    // Phase 1: MarkGray (Trial Deletion)
    for root_address in &roots_to_process {
        let current_header = heap.read_header(*root_address)?;
        if current_header.color == GcColor::Purple {
            mark_gray(heap, context, *root_address)?;
        }
    }

    // Phase 2: Scan
    for root_address in &roots_to_process {
        scan(heap, context, *root_address)?;
    }

    // Phase 3: CollectWhite
    let mut total_reclaimed_blocks = 0u64;
    let mut freed_addresses = HashSet::new();
    for root_address in &roots_to_process {
        total_reclaimed_blocks +=
            collect_white(heap, context, *root_address, &mut freed_addresses)?;
    }

    Ok((total_reclaimed_blocks, deferred_roots))
}

/// Stateful cycle collector controller managing the purple root candidate buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleCollector {
    /// Preallocated candidate purple root buffer.
    pub purple_buffer: Vec<u32>,
    /// Configured candidate threshold triggering synchronous collection.
    pub threshold: usize,
    /// Bounded time slice budget in microseconds.
    pub budget_microseconds: Option<u64>,
    /// Total collections executed.
    pub total_runs: u64,
    /// Total candidate roots processed.
    pub total_candidates: u64,
    /// Total cyclic garbage blocks collected and freed.
    pub total_cycles_collected: u64,
    /// Cumulative microseconds spent in cycle collection.
    pub total_time_microseconds: u64,
}

impl CycleCollector {
    /// Creates a new cycle collector with specified candidate threshold and budget.
    #[must_use]
    pub fn new(threshold: usize, budget_microseconds: Option<u64>) -> Self {
        Self {
            purple_buffer: Vec::with_capacity(threshold.max(16)),
            threshold: threshold.max(1),
            budget_microseconds,
            total_runs: 0,
            total_candidates: 0,
            total_cycles_collected: 0,
            total_time_microseconds: 0,
        }
    }

    /// Suspects an object whose reference count decremented to a non-zero count.
    #[inline(always)]
    pub fn suspect_candidate(
        &mut self,
        heap: &mut VirtualHeap,
        address: u32,
    ) -> Result<bool, RuntimeError> {
        let mut header = heap.read_header(address)?;
        if header.color != GcColor::Purple && can_contain_references(header.tag) {
            header.color = GcColor::Purple;
            heap.write_header(address, header)?;
            self.purple_buffer.push(address);
            self.total_candidates += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Collects cycles across all buffered purple roots.
    pub fn collect(
        &mut self,
        heap: &mut VirtualHeap,
        context: &RuntimeContext,
    ) -> Result<u64, RuntimeError> {
        if self.purple_buffer.is_empty() {
            return Ok(0);
        }

        let run_start_time = Instant::now();
        self.total_runs += 1;

        let candidate_roots = std::mem::take(&mut self.purple_buffer);
        let (reclaimed_count, deferred) =
            run_cycle_collection(heap, context, &candidate_roots, self.budget_microseconds)?;

        self.purple_buffer = deferred;
        self.total_cycles_collected += reclaimed_count;
        self.total_time_microseconds += run_start_time.elapsed().as_micros() as u64;

        Ok(reclaimed_count)
    }
}
