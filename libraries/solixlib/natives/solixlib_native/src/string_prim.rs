//! String manipulation and numeric primitive native functions.

use crate::{
    alloc_solix_chars, read_solix_chars, register_symbol_variants, write_out_box_f64,
    write_out_box_i64, NativeCFunctionPtr, RegisterNativeCallback, SolixCValue,
};
use std::ffi::c_void;

// --- String Natives ---

pub unsafe extern "C" fn string_native_from_int(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let value = unsafe { (*args.add(0)).as_i64() };
    let formatted_string = value.to_string();
    let address = unsafe { alloc_solix_chars(vm_context, &formatted_string) };
    SolixCValue::ref_val(address)
}

pub unsafe extern "C" fn string_native_from_double(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    let precision = if arg_count > 1 {
        let raw_precision = unsafe { (*args.add(1)).as_i32() };
        if raw_precision >= 0 {
            raw_precision as usize
        } else {
            6
        }
    } else {
        6
    };
    let formatted_string = format!("{value:.precision$}");
    let address = unsafe { alloc_solix_chars(vm_context, &formatted_string) };
    SolixCValue::ref_val(address)
}

pub unsafe extern "C" fn string_native_from_bool(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let value = unsafe { (*args.add(0)).as_bool() };
    let formatted_string = if value { "true" } else { "false" };
    let address = unsafe { alloc_solix_chars(vm_context, formatted_string) };
    SolixCValue::ref_val(address)
}

pub unsafe extern "C" fn string_native_try_parse_int(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let text_address = unsafe { (*args.add(0)).as_ref() };
    let out_box_address = unsafe { (*args.add(1)).as_ref() };
    let text = unsafe { read_solix_chars(vm_context, text_address) };
    let trimmed = text.trim();
    if let Ok(parsed_integer) = trimmed.parse::<i64>() {
        unsafe { write_out_box_i64(vm_context, out_box_address, parsed_integer) };
        SolixCValue::i32_val(1)
    } else {
        SolixCValue::i32_val(0)
    }
}

pub unsafe extern "C" fn string_native_try_parse_double(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let text_address = unsafe { (*args.add(0)).as_ref() };
    let out_box_address = unsafe { (*args.add(1)).as_ref() };
    let text = unsafe { read_solix_chars(vm_context, text_address) };
    let trimmed = text.trim();
    if let Ok(parsed_double) = trimmed.parse::<f64>() {
        unsafe { write_out_box_f64(vm_context, out_box_address, parsed_double) };
        SolixCValue::i32_val(1)
    } else {
        SolixCValue::i32_val(0)
    }
}

pub unsafe extern "C" fn string_native_to_lower(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let text = unsafe { read_solix_chars(vm_context, address) };
    let lowercase_text = text.to_lowercase();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &lowercase_text) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn string_native_to_upper(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let text = unsafe { read_solix_chars(vm_context, address) };
    let uppercase_text = text.to_uppercase();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &uppercase_text) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn string_native_replace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 3 || args.is_null() {
        return SolixCValue::null();
    }
    let source_address = unsafe { (*args.add(0)).as_ref() };
    let old_token_address = unsafe { (*args.add(1)).as_ref() };
    let new_token_address = unsafe { (*args.add(2)).as_ref() };

    let source_text = unsafe { read_solix_chars(vm_context, source_address) };
    let old_token = unsafe { read_solix_chars(vm_context, old_token_address) };
    let new_token = unsafe { read_solix_chars(vm_context, new_token_address) };

    let replaced_text = if old_token.is_empty() {
        source_text
    } else {
        source_text.replace(&old_token, &new_token)
    };

    let allocated_address = unsafe { alloc_solix_chars(vm_context, &replaced_text) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn string_native_index_of(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i64_val(-1);
    }
    let haystack_address = unsafe { (*args.add(0)).as_ref() };
    let needle_address = unsafe { (*args.add(1)).as_ref() };
    let start_index = if arg_count > 2 {
        unsafe { (*args.add(2)).as_i32() }.max(0) as usize
    } else {
        0
    };

    let haystack = unsafe { read_solix_chars(vm_context, haystack_address) };
    let needle = unsafe { read_solix_chars(vm_context, needle_address) };

    if start_index >= haystack.len() {
        if needle.is_empty() && start_index == haystack.len() {
            return SolixCValue::i64_val(start_index as i64);
        }
        return SolixCValue::i64_val(-1);
    }

    match haystack[start_index..].find(&needle) {
        Some(relative_position) => SolixCValue::i64_val((start_index + relative_position) as i64),
        None => SolixCValue::i64_val(-1),
    }
}

pub unsafe extern "C" fn string_native_last_index_of(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i64_val(-1);
    }
    let haystack_address = unsafe { (*args.add(0)).as_ref() };
    let needle_address = unsafe { (*args.add(1)).as_ref() };

    let haystack = unsafe { read_solix_chars(vm_context, haystack_address) };
    let needle = unsafe { read_solix_chars(vm_context, needle_address) };

    match haystack.rfind(&needle) {
        Some(position) => SolixCValue::i64_val(position as i64),
        None => SolixCValue::i64_val(-1),
    }
}

// --- Primitive Natives ---

pub unsafe extern "C" fn double_native_min_value(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::f64_val(f64::MIN_POSITIVE)
}

pub unsafe extern "C" fn double_native_max_value(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::f64_val(f64::MAX)
}

pub unsafe extern "C" fn double_native_nan(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::f64_val(f64::NAN)
}

pub unsafe extern "C" fn double_native_pos_inf(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::f64_val(f64::INFINITY)
}

pub unsafe extern "C" fn double_native_neg_inf(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::f64_val(f64::NEG_INFINITY)
}

pub unsafe extern "C" fn double_native_is_nan(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::bool_val(value.is_nan())
}

pub unsafe extern "C" fn double_native_is_infinite(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::bool_val(value.is_infinite())
}

pub unsafe extern "C" fn int_native_clz(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(32);
    }
    let value = unsafe { (*args.add(0)).as_i32() } as u32;
    SolixCValue::i32_val(value.leading_zeros() as i32)
}

pub unsafe extern "C" fn int_native_ctz(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(32);
    }
    let value = unsafe { (*args.add(0)).as_i32() } as u32;
    SolixCValue::i32_val(value.trailing_zeros() as i32)
}

pub unsafe extern "C" fn int_native_popcount(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i32() } as u32;
    SolixCValue::i32_val(value.count_ones() as i32)
}

pub unsafe extern "C" fn int_native_bswap(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i32() } as u32;
    SolixCValue::i32_val(value.swap_bytes() as i32)
}

pub unsafe extern "C" fn primitives_native_hash_any(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let raw_payload = unsafe { (*args.add(0)).payload };
    // Murmur/SplitMix finalizer hash of the 64-bit word
    let mut hash_state = raw_payload;
    hash_state ^= hash_state >> 30;
    hash_state = hash_state.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    hash_state ^= hash_state >> 27;
    hash_state = hash_state.wrapping_mul(0x94d0_49bb_1331_11eb);
    hash_state ^= hash_state >> 31;
    let final_i32 = (hash_state as i32) & 0x7FFF_FFFF;
    SolixCValue::i32_val(if final_i32 == 0 { 1 } else { final_i32 })
}

pub unsafe fn register_string_prim_natives(
    collector: *mut c_void,
    callback: RegisterNativeCallback,
) {
    let string_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "String_native_from_int",
                "solix_core_String_native_from_int",
                "String::native_from_int",
            ],
            string_native_from_int,
        ),
        (
            &[
                "String_native_from_double",
                "solix_core_String_native_from_double",
                "String::native_from_double",
            ],
            string_native_from_double,
        ),
        (
            &[
                "String_native_from_bool",
                "solix_core_String_native_from_bool",
                "String::native_from_bool",
            ],
            string_native_from_bool,
        ),
        (
            &[
                "String_native_try_parse_int",
                "solix_core_String_native_try_parse_int",
                "String::native_try_parse_int",
            ],
            string_native_try_parse_int,
        ),
        (
            &[
                "String_native_try_parse_double",
                "solix_core_String_native_try_parse_double",
                "String::native_try_parse_double",
            ],
            string_native_try_parse_double,
        ),
        (
            &[
                "String_native_to_lower",
                "solix_core_String_native_to_lower",
                "String::native_to_lower",
            ],
            string_native_to_lower,
        ),
        (
            &[
                "String_native_to_upper",
                "solix_core_String_native_to_upper",
                "String::native_to_upper",
            ],
            string_native_to_upper,
        ),
        (
            &[
                "String_native_replace",
                "solix_core_String_native_replace",
                "String::native_replace",
            ],
            string_native_replace,
        ),
        (
            &[
                "String_native_index_of",
                "solix_core_String_native_index_of",
                "String::native_index_of",
            ],
            string_native_index_of,
        ),
        (
            &[
                "String_native_last_index_of",
                "solix_core_String_native_last_index_of",
                "String::native_last_index_of",
            ],
            string_native_last_index_of,
        ),
    ];

    for (variants, func) in string_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let double_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Double_native_min_value",
                "solix_core_Double_native_min_value",
                "Primitives_native_min_double",
                "Primitives::native_min_double",
            ],
            double_native_min_value,
        ),
        (
            &[
                "Double_native_max_value",
                "solix_core_Double_native_max_value",
                "Primitives_native_max_double",
                "Primitives::native_max_double",
            ],
            double_native_max_value,
        ),
        (
            &[
                "Double_native_nan",
                "solix_core_Double_native_nan",
                "Primitives_native_nan",
                "Primitives::native_nan",
            ],
            double_native_nan,
        ),
        (
            &[
                "Double_native_pos_inf",
                "solix_core_Double_native_pos_inf",
                "Primitives_native_pos_inf",
                "Primitives::native_pos_inf",
            ],
            double_native_pos_inf,
        ),
        (
            &[
                "Double_native_neg_inf",
                "solix_core_Double_native_neg_inf",
                "Primitives_native_neg_inf",
                "Primitives::native_neg_inf",
            ],
            double_native_neg_inf,
        ),
        (
            &[
                "Double_native_is_nan",
                "solix_core_Double_native_is_nan",
                "Primitives_native_is_nan",
                "Primitives::native_is_nan",
            ],
            double_native_is_nan,
        ),
        (
            &[
                "Double_native_is_infinite",
                "solix_core_Double_native_is_infinite",
                "Primitives_native_is_infinite",
                "Primitives::native_is_infinite",
            ],
            double_native_is_infinite,
        ),
    ];

    for (variants, func) in double_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let int_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Int_native_clz",
                "solix_core_Int_native_clz",
                "Primitives_native_clz",
                "Primitives::native_clz",
            ],
            int_native_clz,
        ),
        (
            &[
                "Int_native_ctz",
                "solix_core_Int_native_ctz",
                "Primitives_native_ctz",
                "Primitives::native_ctz",
            ],
            int_native_ctz,
        ),
        (
            &[
                "Int_native_popcount",
                "solix_core_Int_native_popcount",
                "Primitives_native_popcount",
                "Primitives::native_popcount",
            ],
            int_native_popcount,
        ),
        (
            &[
                "Int_native_bswap",
                "solix_core_Int_native_bswap",
                "Primitives_native_bswap",
                "Primitives::native_bswap",
            ],
            int_native_bswap,
        ),
        (
            &[
                "Primitives_native_hash_any",
                "solix_core_Primitives_native_hash_any",
                "Primitives::native_hash_any",
            ],
            primitives_native_hash_any,
        ),
    ];

    for (variants, func) in int_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
