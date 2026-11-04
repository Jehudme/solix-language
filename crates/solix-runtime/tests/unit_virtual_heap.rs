//! Unit tests for VirtualHeap, segregated free-list allocator, and memory integrity.

#![forbid(unsafe_code)]

use solix_runtime::{
    BlockHeader, BlockTag, HEADER_SIZE, RuntimeError, VirtualHeap, align_up, dec_ref, inc_ref,
};

#[test]
fn test_align_up_helper() {
    assert_eq!(align_up(0), 0);
    assert_eq!(align_up(1), 8);
    assert_eq!(align_up(7), 8);
    assert_eq!(align_up(8), 8);
    assert_eq!(align_up(9), 16);
}

#[test]
fn test_block_header_packing_and_unpacking() {
    let header = BlockHeader::new(1024, BlockTag::Object, 42);
    let bytes = header.to_bytes();
    let unpacked = BlockHeader::from_bytes(bytes);

    assert_eq!(unpacked.size, 1024);
    assert_eq!(unpacked.tag, BlockTag::Object);
    assert_eq!(unpacked.ref_count, 42);
}

#[test]
fn test_block_header_all_tags() {
    let tags = [
        BlockTag::Raw,
        BlockTag::Object,
        BlockTag::ArrayU8,
        BlockTag::ArrayI32,
        BlockTag::ArrayI64,
        BlockTag::ArrayF64,
        BlockTag::ArrayRef,
        BlockTag::Tuple,
        BlockTag::Closure,
        BlockTag::String,
    ];

    for tag in tags {
        let h = BlockHeader::new(64, tag, 1);
        let unpacked = BlockHeader::from_bytes(h.to_bytes());
        assert_eq!(unpacked.tag, tag);
    }
}

#[test]
fn test_static_partition_read_write_and_bounds() {
    let mut heap = VirtualHeap::new(4, 1024);
    assert_eq!(heap.static_size(), 32); // 4 slots * 8 bytes = 32

    // Write and read all 4 static slots
    heap.write_static_u64(0, 0x1122_3344_5566_7788).unwrap();
    heap.write_static_u64(1, 42).unwrap();
    heap.write_static_u64(2, 999_999).unwrap();
    heap.write_static_u64(3, 0xFFFF_0000_FFFF_0000).unwrap();

    assert_eq!(heap.read_static_u64(0).unwrap(), 0x1122_3344_5566_7788);
    assert_eq!(heap.read_static_u64(1).unwrap(), 42);
    assert_eq!(heap.read_static_u64(2).unwrap(), 999_999);
    assert_eq!(heap.read_static_u64(3).unwrap(), 0xFFFF_0000_FFFF_0000);

    // Slot 4 is out of bounds
    let err = heap.write_static_u64(4, 123);
    assert!(matches!(err, Err(RuntimeError::InvalidStaticOffset { .. })));
}

#[test]
fn test_dynamic_allocation_isolation_and_zero_init() {
    let mut heap = VirtualHeap::new(2, 1024);
    // Write static variable
    heap.write_static_u64(0, 0xCAFEBABE).unwrap();

    // Allocate dynamic block
    let addr = heap.allocate(16, BlockTag::ArrayI32).unwrap();
    assert!(addr as usize >= heap.static_size() + HEADER_SIZE);

    // Header inspection
    let header = heap.read_header(addr).unwrap();
    assert_eq!(header.tag, BlockTag::ArrayI32);
    assert_eq!(header.size, 16);
    assert_eq!(header.ref_count, 1);

    // Verify payload is zero-initialized
    let payload = heap.read_bytes(addr, 16).unwrap();
    assert_eq!(payload, &[0u8; 16]);

    // Verify static variable was not overwritten
    assert_eq!(heap.read_static_u64(0).unwrap(), 0xCAFEBABE);

    // Write and read dynamic data
    heap.write_u32(addr, 100).unwrap();
    heap.write_u32(addr + 4, 200).unwrap();
    assert_eq!(heap.read_u32(addr).unwrap(), 100);
    assert_eq!(heap.read_u32(addr + 4).unwrap(), 200);

    // Cleanup
    heap.deallocate(addr).unwrap();
    assert!(heap.verify_clean_memory().is_ok());
}

#[test]
fn test_free_list_three_block_coalescing() {
    let mut heap = VirtualHeap::new(0, 1024);

    // Allocate 3 consecutive blocks
    let addr_a = heap.allocate(32, BlockTag::Object).unwrap();
    let addr_b = heap.allocate(32, BlockTag::Object).unwrap();
    let addr_c = heap.allocate(32, BlockTag::Object).unwrap();
    assert_eq!(heap.live_objects_count(), 3);

    // Free middle block B (creates free gap)
    heap.deallocate(addr_b).unwrap();
    assert_eq!(heap.live_objects_count(), 2);
    assert!(heap.verify_clean_memory().is_err());

    // Free block A (should coalesce with B)
    heap.deallocate(addr_a).unwrap();
    assert_eq!(heap.live_objects_count(), 1);

    // Free block C (should coalesce with A+B and the rest of heap)
    heap.deallocate(addr_c).unwrap();
    assert_eq!(heap.live_objects_count(), 0);

    // Must be completely clean and fully coalesced
    assert!(heap.verify_clean_memory().is_ok());
}

#[test]
fn test_verify_clean_memory_detects_leaks() {
    let mut heap = VirtualHeap::new(0, 512);

    let _leaked_addr = heap.allocate(64, BlockTag::Closure).unwrap();
    assert_eq!(heap.live_objects_count(), 1);

    let result = heap.verify_clean_memory();
    assert!(result.is_err());
    match result {
        Err(RuntimeError::MemoryLeak {
            unallocated_count,
            report,
        }) => {
            assert_eq!(unallocated_count, 1);
            assert!(
                report.contains("Tag=Closure"),
                "Expected Tag=Closure in leak report: {report}"
            );
        }
        _ => panic!("Expected MemoryLeak error"),
    }
}

#[test]
fn test_out_of_memory_error() {
    let mut heap = VirtualHeap::new(0, 128);

    // Requesting more than total capacity triggers OOM
    let result = heap.allocate(256, BlockTag::Raw);
    assert!(matches!(result, Err(RuntimeError::OutOfMemory { .. })));
}

#[test]
fn test_arc_retain_and_release() {
    let mut heap = VirtualHeap::new(0, 256);

    let addr = heap.allocate(16, BlockTag::String).unwrap();
    assert_eq!(heap.read_header(addr).unwrap().ref_count, 1);

    // Retain
    let rc2 = inc_ref(&mut heap, addr).unwrap();
    assert_eq!(rc2, 2);
    assert_eq!(heap.read_header(addr).unwrap().ref_count, 2);

    // Retain again
    let rc3 = inc_ref(&mut heap, addr).unwrap();
    assert_eq!(rc3, 3);

    // Release
    let rc_after = dec_ref(&mut heap, addr).unwrap();
    assert_eq!(rc_after, 2);

    let rc_after2 = dec_ref(&mut heap, addr).unwrap();
    assert_eq!(rc_after2, 1);

    let rc_final = dec_ref(&mut heap, addr).unwrap();
    assert_eq!(rc_final, 0);

    // Null pointer is safe no-op
    assert_eq!(inc_ref(&mut heap, 0).unwrap(), 0);
    assert_eq!(dec_ref(&mut heap, 0).unwrap(), 0);

    heap.deallocate(addr).unwrap();
    assert!(heap.verify_clean_memory().is_ok());
}
