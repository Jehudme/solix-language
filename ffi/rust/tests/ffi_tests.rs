//! Unit tests for `solix-ffi` tagged values, context bindings, and macros.

use solix_ffi::{
    NativeCFunctionPtr, NativeRegistry, SolixCValue, TAG_BOOL, TAG_F64, TAG_I32, TAG_I64, TAG_NULL,
    TAG_REF, solix_export,
};
use std::ffi::{CStr, c_char, c_void};

#[test]
fn test_solix_c_value_scalar_encodings_and_decodings() {
    // 1. Null value
    let null_value = SolixCValue::null();
    assert_eq!(null_value.tag, TAG_NULL);
    assert_eq!(null_value.payload, 0);
    assert!(null_value.is_null());

    // 2. Boolean values
    let true_value = SolixCValue::from_bool(true);
    assert_eq!(true_value.tag, TAG_BOOL);
    assert_eq!(true_value.payload, 1);
    assert!(true_value.as_bool());
    assert!(!true_value.is_null());

    let false_value = SolixCValue::from_bool(false);
    assert_eq!(false_value.tag, TAG_BOOL);
    assert_eq!(false_value.payload, 0);
    assert!(!false_value.as_bool());

    // 3. 32-bit signed integers
    let positive_i32 = SolixCValue::from_i32(42);
    assert_eq!(positive_i32.tag, TAG_I32);
    assert_eq!(positive_i32.as_i32(), 42);

    let negative_i32 = SolixCValue::from_i32(-999);
    assert_eq!(negative_i32.tag, TAG_I32);
    assert_eq!(negative_i32.as_i32(), -999);

    // 4. 64-bit signed integers
    let large_i64 = SolixCValue::from_i64(9_223_372_036_854_775_000);
    assert_eq!(large_i64.tag, TAG_I64);
    assert_eq!(large_i64.as_i64(), 9_223_372_036_854_775_000);

    // 5. 64-bit floating point numbers
    let pi_f64 = SolixCValue::from_f64(std::f64::consts::PI);
    assert_eq!(pi_f64.tag, TAG_F64);
    assert!((pi_f64.as_f64() - std::f64::consts::PI).abs() < 1e-12);

    // 6. Object heap reference addresses
    let reference_value = SolixCValue::from_ref(1048576);
    assert_eq!(reference_value.tag, TAG_REF);
    assert_eq!(reference_value.as_ref(), 1048576);
    assert!(!reference_value.is_null());
}

struct TestCollector {
    symbols: Vec<String>,
}

unsafe extern "C" fn mock_collector_callback(
    collector: *mut c_void,
    symbol_pointer: *const c_char,
    _function: NativeCFunctionPtr,
) {
    let collector_reference = unsafe { &mut *(collector as *mut TestCollector) };
    let symbol_string = unsafe { CStr::from_ptr(symbol_pointer) }
        .to_str()
        .expect("Valid UTF-8 symbol name")
        .to_string();
    collector_reference.symbols.push(symbol_string);
}

unsafe extern "C-unwind" fn test_native_dummy_function(
    _context: *mut c_void,
    _arguments: *const SolixCValue,
    _count: u32,
    _error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::from_i32(100)
}

#[test]
fn test_native_registry_symbol_registration() {
    let mut collector = TestCollector {
        symbols: Vec::new(),
    };
    let collector_pointer = (&mut collector as *mut TestCollector).cast::<c_void>();
    let mut registry = NativeRegistry::new(collector_pointer, mock_collector_callback);

    registry.register("Math::add", test_native_dummy_function);
    registry.register("Math::multiply", test_native_dummy_function);

    assert_eq!(collector.symbols.len(), 2);
    assert_eq!(collector.symbols[0], "Math::add");
    assert_eq!(collector.symbols[1], "Math::multiply");
}

mod export_test_module {
    use super::*;
    use std::collections::HashMap;

    struct ExportEntryCollector {
        entries: HashMap<String, NativeCFunctionPtr>,
    }

    unsafe extern "C" fn collect_export_entry(
        collector: *mut c_void,
        symbol_pointer: *const c_char,
        function_ptr: NativeCFunctionPtr,
    ) {
        let collector_ref = unsafe { &mut *(collector as *mut ExportEntryCollector) };
        let symbol_string = unsafe { CStr::from_ptr(symbol_pointer) }
            .to_str()
            .expect("Valid UTF-8 symbol name")
            .to_string();
        collector_ref.entries.insert(symbol_string, function_ptr);
    }

    unsafe extern "C-unwind" fn bridge_fast_sqrt(
        _context: *mut c_void,
        _arguments: *const SolixCValue,
        _count: u32,
        _error: *mut SolixCValue,
    ) -> SolixCValue {
        SolixCValue::from_f64(4.0)
    }

    unsafe extern "C-unwind" fn bridge_panicking_plugin(
        _context: *mut c_void,
        _arguments: *const SolixCValue,
        _count: u32,
        _error: *mut SolixCValue,
    ) -> SolixCValue {
        panic!("simulated native panic inside solix_export");
    }

    solix_export! {
        "Math::fast_sqrt" => bridge_fast_sqrt,
        "Math::panic_trap" => bridge_panicking_plugin,
    }

    #[test]
    fn test_solix_export_macro_invocation_null_guards_and_panic_catch() {
        // Null host pointers must safely no-op without UB or panics.
        unsafe {
            solix_register_natives(std::ptr::null_mut(), None);
            solix_register_natives(std::ptr::null_mut(), Some(mock_collector_callback));
        }
        let mut collector = TestCollector {
            symbols: Vec::new(),
        };
        let collector_pointer = (&mut collector as *mut TestCollector).cast::<c_void>();
        unsafe {
            solix_register_natives(collector_pointer, None);
            solix_register_natives(collector_pointer, Some(mock_collector_callback));
        }
        assert_eq!(collector.symbols.len(), 2);
        assert_eq!(collector.symbols[0], "Math::fast_sqrt");
        assert_eq!(collector.symbols[1], "Math::panic_trap");

        // Invoking the registered trampoline for a panicking callback catches the unwind cleanly.
        let mut entry_collector = ExportEntryCollector {
            entries: HashMap::new(),
        };
        let entry_ptr = (&mut entry_collector as *mut ExportEntryCollector).cast::<c_void>();
        unsafe { solix_register_natives(entry_ptr, Some(collect_export_entry)) };

        let panic_fn = *entry_collector
            .entries
            .get("Math::panic_trap")
            .expect("panic_trap must be registered");
        let mut err_slot = SolixCValue::null();
        let ret_val = unsafe { panic_fn(std::ptr::null_mut(), std::ptr::null(), 0, &mut err_slot) };
        assert!(ret_val.is_null());
        assert!(!err_slot.is_null());
    }
}

mod vm_context_wrapper_tests {
    use super::*;
    use solix_ffi::{
        SOLIX_ARRAY_TAG_BOOL, SOLIX_ARRAY_TAG_CHAR, SOLIX_ARRAY_TAG_F32, SOLIX_ARRAY_TAG_F64,
        SOLIX_ARRAY_TAG_I8, SOLIX_ARRAY_TAG_I16, SOLIX_ARRAY_TAG_I32, SOLIX_ARRAY_TAG_I64,
        SOLIX_ARRAY_TAG_REF, SOLIX_ARRAY_TAG_U8, SOLIX_ARRAY_TAG_U16, SOLIX_ARRAY_TAG_U32,
        SOLIX_ARRAY_TAG_U64, SOLIX_VM_CONTEXT_MAGIC, SolixVmContext, TAG_FNPTR,
    };

    struct MockHostVm {
        stored_text: String,
        field_i32: i32,
        field_u64: u64,
        array_items: Vec<u64>,
        cycle_threshold: u32,
        collected_count: u64,
    }

    unsafe extern "C" fn mock_get_i32(vm_ptr: *mut c_void, _addr: u32, _idx: u32) -> i32 {
        unsafe { (*(vm_ptr as *mut MockHostVm)).field_i32 }
    }
    unsafe extern "C" fn mock_set_i32(vm_ptr: *mut c_void, _addr: u32, _idx: u32, val: i32) {
        unsafe { (*(vm_ptr as *mut MockHostVm)).field_i32 = val };
    }
    unsafe extern "C" fn mock_read_bytes(
        vm_ptr: *mut c_void,
        _addr: u32,
        out_buf: *mut u8,
        cap: u32,
    ) -> u32 {
        let host = unsafe { &*(vm_ptr as *mut MockHostVm) };
        let bytes = host.stored_text.as_bytes();
        let copy_len = bytes.len().min(cap as usize);
        if !out_buf.is_null() && copy_len > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf, copy_len);
            }
        }
        bytes.len() as u32
    }
    unsafe extern "C" fn mock_alloc_string(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32 {
        let host = unsafe { &mut *(vm_ptr as *mut MockHostVm) };
        let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
        host.stored_text = String::from_utf8_lossy(slice).to_string();
        512
    }
    unsafe extern "C" fn mock_alloc_chars(_vm: *mut c_void, _ptr: *const u8, len: u32) -> u32 {
        1000 + len
    }
    unsafe extern "C" fn mock_alloc_byte_array(_vm: *mut c_void, _ptr: *const u8, len: u32) -> u32 {
        2000 + len
    }
    unsafe extern "C" fn mock_alloc_array(vm_ptr: *mut c_void, len: u32, _tag: u8) -> u32 {
        let host = unsafe { &mut *(vm_ptr as *mut MockHostVm) };
        host.array_items = vec![0; len as usize];
        3000
    }
    unsafe extern "C" fn mock_array_len(vm_ptr: *mut c_void, _addr: u32) -> u32 {
        unsafe { (*(vm_ptr as *mut MockHostVm)).array_items.len() as u32 }
    }
    unsafe extern "C" fn mock_array_get_u64(vm_ptr: *mut c_void, _addr: u32, index: u32) -> u64 {
        let host = unsafe { &*(vm_ptr as *const MockHostVm) };
        host.array_items[index as usize]
    }
    unsafe extern "C" fn mock_array_set_u64(
        vm_ptr: *mut c_void,
        _addr: u32,
        index: u32,
        raw_val: u64,
    ) {
        let host = unsafe { &mut *(vm_ptr as *mut MockHostVm) };
        host.array_items[index as usize] = raw_val;
    }
    unsafe extern "C" fn mock_alloc_object(_vm: *mut c_void, vtable_id: u32, _fields: u32) -> u32 {
        if vtable_id == 99 { 4096 } else { 0 }
    }
    unsafe extern "C" fn mock_read_field_u64(vm_ptr: *mut c_void, _addr: u32, _off: u32) -> u64 {
        unsafe { (*(vm_ptr as *mut MockHostVm)).field_u64 }
    }
    unsafe extern "C" fn mock_write_field_u64(
        vm_ptr: *mut c_void,
        _addr: u32,
        _off: u32,
        raw_val: u64,
    ) {
        unsafe { (*(vm_ptr as *mut MockHostVm)).field_u64 = raw_val };
    }
    unsafe extern "C" fn mock_get_vtable(_vm: *mut c_void, name_ptr: *const c_char) -> u32 {
        let name = unsafe { CStr::from_ptr(name_ptr) }.to_str().unwrap_or("");
        if name == "SampleObj" { 99 } else { 0 }
    }
    unsafe extern "C" fn mock_get_metric(_vm: *mut c_void, metric_id: u32) -> u64 {
        u64::from(metric_id) * 10
    }
    unsafe extern "C" fn mock_collect_cycles(vm_ptr: *mut c_void) -> u64 {
        let host = unsafe { &mut *(vm_ptr as *mut MockHostVm) };
        host.collected_count += 1;
        host.collected_count
    }
    unsafe extern "C" fn mock_set_threshold(vm_ptr: *mut c_void, threshold: u32) {
        unsafe { (*(vm_ptr as *mut MockHostVm)).cycle_threshold = threshold };
    }

    #[test]
    fn test_vm_context_parity_and_helpers() {
        assert_eq!(SOLIX_VM_CONTEXT_MAGIC, 0x534F_4C49_5856_4D43);
        let fn_val = SolixCValue::from_fn_ptr(21);
        assert_eq!(fn_val.tag, TAG_FNPTR);
        assert_eq!(fn_val.as_fn_ptr(), 21);

        assert_eq!(SOLIX_ARRAY_TAG_I8, 1);
        assert_eq!(SOLIX_ARRAY_TAG_I16, 2);
        assert_eq!(SOLIX_ARRAY_TAG_I32, 3);
        assert_eq!(SOLIX_ARRAY_TAG_I64, 4);
        assert_eq!(SOLIX_ARRAY_TAG_U8, 5);
        assert_eq!(SOLIX_ARRAY_TAG_U16, 6);
        assert_eq!(SOLIX_ARRAY_TAG_U32, 7);
        assert_eq!(SOLIX_ARRAY_TAG_U64, 8);
        assert_eq!(SOLIX_ARRAY_TAG_F32, 9);
        assert_eq!(SOLIX_ARRAY_TAG_F64, 10);
        assert_eq!(SOLIX_ARRAY_TAG_BOOL, 11);
        assert_eq!(SOLIX_ARRAY_TAG_CHAR, 12);
        assert_eq!(SOLIX_ARRAY_TAG_REF, 13);

        let mut host = MockHostVm {
            stored_text: String::new(),
            field_i32: 0,
            field_u64: 0,
            array_items: Vec::new(),
            cycle_threshold: 0,
            collected_count: 0,
        };
        let mut ctx = SolixVmContext {
            magic: SOLIX_VM_CONTEXT_MAGIC,
            vm_ptr: (&mut host as *mut MockHostVm).cast::<c_void>(),
            get_field_i32: mock_get_i32,
            set_field_i32: mock_set_i32,
            read_bytes: mock_read_bytes,
            alloc_string: mock_alloc_string,
            alloc_chars: mock_alloc_chars,
            alloc_byte_array: mock_alloc_byte_array,
            alloc_array: mock_alloc_array,
            array_len: mock_array_len,
            array_get_u64: mock_array_get_u64,
            array_set_u64: mock_array_set_u64,
            alloc_object: mock_alloc_object,
            read_field_u64: mock_read_field_u64,
            write_field_u64: mock_write_field_u64,
            get_class_vtable_id: mock_get_vtable,
            get_vm_metric: mock_get_metric,
            collect_cycles: mock_collect_cycles,
            set_cycle_threshold: mock_set_threshold,
            get_field_i32_by_index: mock_get_i32,
            set_field_i32_by_index: mock_set_i32,
            get_field_i32_by_offset: mock_get_i32,
            set_field_i32_by_offset: mock_set_i32,
            read_field_u64_by_index: mock_read_field_u64,
            write_field_u64_by_index: mock_write_field_u64,
            read_field_u64_by_offset: mock_read_field_u64,
            write_field_u64_by_offset: mock_write_field_u64,
        };

        let validated =
            unsafe { SolixVmContext::validate((&mut ctx as *mut SolixVmContext).cast()) }
                .expect("valid SolixVmContext");
        unsafe {
            let str_addr = validated.alloc_string("rust-ffi-parity");
            assert_eq!(
                validated.read_string(str_addr).as_deref(),
                Some("rust-ffi-parity")
            );
            assert_eq!(validated.alloc_chars("abc"), 1003);
            assert_eq!(validated.alloc_byte_array(b"xy"), 2002);
            let arr_addr = validated.alloc_array(3, SOLIX_ARRAY_TAG_I64);
            assert_eq!(validated.array_len(arr_addr), 3);
            validated.array_set_u64(arr_addr, 1, 777);
            assert_eq!(validated.array_get_u64(arr_addr, 1), 777);
            let vtable = validated.get_class_vtable_id("SampleObj");
            let obj_addr = validated.alloc_object(vtable, 2);
            assert_eq!(obj_addr, 4096);
            validated.set_field_i32(obj_addr, 0, -55);
            assert_eq!(validated.get_field_i32(obj_addr, 0), -55);
            validated.set_field_i32_by_index(obj_addr, 9, -77);
            assert_eq!(validated.get_field_i32_by_index(obj_addr, 9), -77);
            validated.set_field_i32_by_offset(obj_addr, 12, 88);
            assert_eq!(validated.get_field_i32_by_offset(obj_addr, 12), 88);
            validated.write_field_u64(obj_addr, 16, 123456);
            assert_eq!(validated.read_field_u64(obj_addr, 16), 123456);
            validated.write_field_u64_by_index(obj_addr, 10, 223344);
            assert_eq!(validated.read_field_u64_by_index(obj_addr, 10), 223344);
            validated.write_field_u64_by_offset(obj_addr, 24, 556677);
            assert_eq!(validated.read_field_u64_by_offset(obj_addr, 24), 556677);
            let mut err_slot = SolixCValue::null();
            validated.set_error(&mut err_slot, "explicit error string");
            assert_eq!(err_slot.tag, TAG_REF);
            assert_eq!(validated.get_vm_metric(4), 40);
            validated.set_cycle_threshold(256);
            assert_eq!(validated.collect_cycles(), 1);
        }
    }
}
