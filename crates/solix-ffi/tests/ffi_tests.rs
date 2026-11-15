//! Unit tests for `solix-ffi` tagged values, context bindings, and macros.

use solix_ffi::{
    solix_export, NativeCFunctionPtr, NativeRegistry, SolixCValue, TAG_BOOL, TAG_F64, TAG_I32,
    TAG_I64, TAG_NULL, TAG_REF,
};
use std::ffi::{c_char, c_void, CStr};

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
    let pi_f64 = SolixCValue::from_f64(3.141592653589793);
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

unsafe extern "C" fn test_native_dummy_function(
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

    unsafe extern "C" fn bridge_fast_sqrt(
        _context: *mut c_void,
        _arguments: *const SolixCValue,
        _count: u32,
        _error: *mut SolixCValue,
    ) -> SolixCValue {
        SolixCValue::from_f64(4.0)
    }

    solix_export! {
        "Math::fast_sqrt" => bridge_fast_sqrt,
    }

    #[test]
    fn test_solix_export_macro_invocation() {
        let mut collector = TestCollector {
            symbols: Vec::new(),
        };
        let collector_pointer = (&mut collector as *mut TestCollector).cast::<c_void>();
        unsafe {
            solix_register_natives(collector_pointer, mock_collector_callback);
        }
        assert_eq!(collector.symbols.len(), 1);
        assert_eq!(collector.symbols[0], "Math::fast_sqrt");
    }
}
