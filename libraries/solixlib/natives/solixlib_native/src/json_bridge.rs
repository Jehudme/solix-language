//! JSON manipulation and serialization native bridge using serde_json.

use crate::{
    alloc_solix_chars, read_solix_chars, register_symbol_variants, NativeCFunctionPtr,
    RegisterNativeCallback, SolixCValue,
};
use std::ffi::c_void;

pub unsafe extern "C" fn json_native_parse_and_validate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let text_address = unsafe { (*args.add(0)).as_ref() };
    let json_text = unsafe { read_solix_chars(vm_context, text_address) };

    match serde_json::from_str::<serde_json::Value>(&json_text) {
        Ok(_) => SolixCValue::bool_val(true),
        Err(parse_error) => {
            let error_message = format!(
                "[E0332] Failed to parse JSON string at line {}, column {}: {}",
                parse_error.line(),
                parse_error.column(),
                parse_error
            );
            unsafe { crate::set_native_error(vm_context, out_error, &error_message) };
            SolixCValue::bool_val(false)
        }
    }
}

pub unsafe extern "C" fn json_native_pointer_query(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::null();
    }
    let json_address = unsafe { (*args.add(0)).as_ref() };
    let pointer_address = unsafe { (*args.add(1)).as_ref() };
    let json_text = unsafe { read_solix_chars(vm_context, json_address) };
    let pointer_text = unsafe { read_solix_chars(vm_context, pointer_address) };

    let parsed_json = match serde_json::from_str::<serde_json::Value>(&json_text) {
        Ok(json_value) => json_value,
        Err(parse_error) => {
            let error_message = format!(
                "[E0332] Failed to parse JSON string at line {}, column {}: {}",
                parse_error.line(),
                parse_error.column(),
                parse_error
            );
            unsafe { crate::set_native_error(vm_context, out_error, &error_message) };
            return SolixCValue::null();
        }
    };

    let pointer_path = if pointer_text.starts_with('/') {
        pointer_text
    } else {
        format!("/{pointer_text}")
    };

    if let Some(target_value) = parsed_json.pointer(&pointer_path) {
        let serialized_target = match target_value {
            serde_json::Value::String(string_content) => string_content.clone(),
            other_value => other_value.to_string(),
        };
        let allocated_address = unsafe { alloc_solix_chars(vm_context, &serialized_target) };
        SolixCValue::ref_val(allocated_address)
    } else {
        SolixCValue::null()
    }
}

pub unsafe extern "C" fn json_native_stringify(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let json_address = unsafe { (*args.add(0)).as_ref() };
    let pretty = if arg_count > 1 {
        unsafe { (*args.add(1)).as_bool() }
    } else {
        false
    };
    let json_text = unsafe { read_solix_chars(vm_context, json_address) };

    let parsed_json = match serde_json::from_str::<serde_json::Value>(&json_text) {
        Ok(json_value) => json_value,
        Err(parse_error) => {
            let error_message = format!(
                "[E0332] Failed to parse JSON string at line {}, column {}: {}",
                parse_error.line(),
                parse_error.column(),
                parse_error
            );
            unsafe { crate::set_native_error(vm_context, out_error, &error_message) };
            return SolixCValue::null();
        }
    };

    let formatted_result = if pretty {
        serde_json::to_string_pretty(&parsed_json).unwrap_or_default()
    } else {
        serde_json::to_string(&parsed_json).unwrap_or_default()
    };

    let allocated_address = unsafe { alloc_solix_chars(vm_context, &formatted_result) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe fn register_json_natives(collector: *mut c_void, callback: RegisterNativeCallback) {
    let mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "JsonValue_native_parse_and_validate",
                "solix_json_JsonValue_native_parse_and_validate",
                "JsonValue::native_parse_and_validate",
            ],
            json_native_parse_and_validate,
        ),
        (
            &[
                "JsonValue_native_pointer_query",
                "solix_json_JsonValue_native_pointer_query",
                "JsonValue::native_pointer_query",
            ],
            json_native_pointer_query,
        ),
        (
            &[
                "JsonValue_native_stringify",
                "solix_json_JsonValue_native_stringify",
                "JsonValue::native_stringify",
            ],
            json_native_stringify,
        ),
    ];

    for (variants, func) in mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
