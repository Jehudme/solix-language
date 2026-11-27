//! JSON manipulation and serialization native bridge using serde_json.

use crate::{
    alloc_solix_chars, alloc_solix_string_array, read_solix_chars, register_symbol_variants,
    NativeCFunctionPtr, RegisterNativeCallback, SolixCValue,
};
use std::ffi::c_void;

fn parse_json_or_set_error(
    vm_context: *mut c_void,
    json_address: u32,
    out_error: *mut SolixCValue,
) -> Option<serde_json::Value> {
    let json_text = unsafe { read_solix_chars(vm_context, json_address) };
    match serde_json::from_str::<serde_json::Value>(&json_text) {
        Ok(json_value) => Some(json_value),
        Err(parse_error) => {
            let error_message = format!(
                "[E0332] Failed to parse JSON string at line {}, column {}: {}",
                parse_error.line(),
                parse_error.column(),
                parse_error
            );
            unsafe { crate::set_native_error(vm_context, out_error, &error_message) };
            None
        }
    }
}

pub unsafe extern "C" fn json_native_parse_and_validate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let text_address = unsafe { (*args.add(0)).as_ref() };
    let is_valid = parse_json_or_set_error(vm_context, text_address, out_error).is_some();
    SolixCValue::bool_val(is_valid)
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
    let pointer_text = unsafe { read_solix_chars(vm_context, pointer_address) };

    let Some(parsed_json) = parse_json_or_set_error(vm_context, json_address, out_error) else {
        return SolixCValue::null();
    };

    let pointer_path = if pointer_text.is_empty() || pointer_text.starts_with('/') {
        pointer_text
    } else {
        format!("/{pointer_text}")
    };

    if let Some(target_value) = parsed_json.pointer(&pointer_path) {
        let serialized_target = target_value.to_string();
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
    let Some(parsed_json) = parse_json_or_set_error(vm_context, json_address, out_error) else {
        return SolixCValue::null();
    };

    let formatted_result = if pretty {
        serde_json::to_string_pretty(&parsed_json).unwrap_or_default()
    } else {
        serde_json::to_string(&parsed_json).unwrap_or_default()
    };

    let allocated_address = unsafe { alloc_solix_chars(vm_context, &formatted_result) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn json_native_type_kind(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(-1);
    }
    let json_address = unsafe { (*args.add(0)).as_ref() };
    let json_text = unsafe { read_solix_chars(vm_context, json_address) };
    let kind_code = match serde_json::from_str::<serde_json::Value>(&json_text) {
        Ok(serde_json::Value::Null) => 0,
        Ok(serde_json::Value::Bool(_)) => 1,
        Ok(serde_json::Value::Number(_)) => 2,
        Ok(serde_json::Value::String(_)) => 3,
        Ok(serde_json::Value::Array(_)) => 4,
        Ok(serde_json::Value::Object(_)) => 5,
        Err(_) => -1,
    };
    SolixCValue::i32_val(kind_code)
}

pub unsafe extern "C" fn json_native_size(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let json_address = unsafe { (*args.add(0)).as_ref() };
    let json_text = unsafe { read_solix_chars(vm_context, json_address) };
    let container_len = match serde_json::from_str::<serde_json::Value>(&json_text) {
        Ok(serde_json::Value::Array(items)) => i32::try_from(items.len()).unwrap_or(i32::MAX),
        Ok(serde_json::Value::Object(entries)) => i32::try_from(entries.len()).unwrap_or(i32::MAX),
        _ => 0,
    };
    SolixCValue::i32_val(container_len)
}

pub unsafe extern "C" fn json_native_keys(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::ref_val(unsafe { alloc_solix_string_array(vm_context, &[]) });
    }
    let json_address = unsafe { (*args.add(0)).as_ref() };
    let json_text = unsafe { read_solix_chars(vm_context, json_address) };
    if let Ok(serde_json::Value::Object(entries)) =
        serde_json::from_str::<serde_json::Value>(&json_text)
    {
        let key_refs: Vec<&str> = entries.keys().map(String::as_str).collect();
        SolixCValue::ref_val(unsafe { alloc_solix_string_array(vm_context, &key_refs) })
    } else {
        SolixCValue::ref_val(unsafe { alloc_solix_string_array(vm_context, &[]) })
    }
}

pub unsafe extern "C" fn json_native_decode_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::ref_val(unsafe { alloc_solix_chars(vm_context, "") });
    }
    let json_address = unsafe { (*args.add(0)).as_ref() };
    let json_text = unsafe { read_solix_chars(vm_context, json_address) };
    let decoded = match serde_json::from_str::<serde_json::Value>(&json_text) {
        Ok(serde_json::Value::String(inner)) => inner,
        Ok(other_val) => other_val.to_string(),
        Err(_) => json_text,
    };
    SolixCValue::ref_val(unsafe { alloc_solix_chars(vm_context, &decoded) })
}

pub unsafe extern "C" fn json_native_encode_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::ref_val(unsafe { alloc_solix_chars(vm_context, "\"\"") });
    }
    let text_address = unsafe { (*args.add(0)).as_ref() };
    let raw_text = unsafe { read_solix_chars(vm_context, text_address) };
    let encoded = serde_json::to_string(&raw_text).unwrap_or_else(|_| "\"\"".to_string());
    SolixCValue::ref_val(unsafe { alloc_solix_chars(vm_context, &encoded) })
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
        (
            &[
                "JsonValue_native_type_kind",
                "solix_json_JsonValue_native_type_kind",
                "JsonValue::native_type_kind",
            ],
            json_native_type_kind,
        ),
        (
            &[
                "JsonValue_native_size",
                "solix_json_JsonValue_native_size",
                "JsonValue::native_size",
            ],
            json_native_size,
        ),
        (
            &[
                "JsonValue_native_keys",
                "solix_json_JsonValue_native_keys",
                "JsonValue::native_keys",
            ],
            json_native_keys,
        ),
        (
            &[
                "JsonValue_native_decode_string",
                "solix_json_JsonValue_native_decode_string",
                "JsonValue::native_decode_string",
            ],
            json_native_decode_string,
        ),
        (
            &[
                "JsonValue_native_encode_string",
                "solix_json_JsonValue_native_encode_string",
                "JsonValue::native_encode_string",
            ],
            json_native_encode_string,
        ),
    ];

    for (variants, func) in mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
