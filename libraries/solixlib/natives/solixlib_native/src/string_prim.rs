//! String manipulation and numeric primitive native functions.

use crate::{
    alloc_solix_chars, alloc_solix_string, read_solix_chars, register_symbol_variants,
    write_out_box_f64, write_out_box_i64, NativeCFunctionPtr, RegisterNativeCallback, SolixCValue,
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
    let address = unsafe { alloc_solix_string(vm_context, &formatted_string) };
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
    let address = unsafe { alloc_solix_string(vm_context, &formatted_string) };
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
    let address = unsafe { alloc_solix_string(vm_context, formatted_string) };
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
    let allocated_address = unsafe { alloc_solix_string(vm_context, &lowercase_text) };
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
    let allocated_address = unsafe { alloc_solix_string(vm_context, &uppercase_text) };
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

    let allocated_address = unsafe { alloc_solix_string(vm_context, &replaced_text) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn string_native_chars_to_string(
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
    let allocated = unsafe { alloc_solix_string(vm_context, &text) };
    SolixCValue::ref_val(allocated)
}

pub unsafe extern "C" fn string_native_string_to_chars(
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
    let allocated = unsafe { alloc_solix_chars(vm_context, &text) };
    SolixCValue::ref_val(allocated)
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

pub unsafe extern "C" fn int_native_clz64(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(64);
    }
    let value = unsafe { (*args.add(0)).as_i64() } as u64;
    SolixCValue::i32_val(value.leading_zeros() as i32)
}

pub unsafe extern "C" fn int_native_ctz64(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(64);
    }
    let value = unsafe { (*args.add(0)).as_i64() } as u64;
    SolixCValue::i32_val(value.trailing_zeros() as i32)
}

pub unsafe extern "C" fn int_native_popcount64(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i64() } as u64;
    SolixCValue::i32_val(value.count_ones() as i32)
}

pub unsafe extern "C" fn int_native_bswap64(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i64() } as u64;
    SolixCValue::i64_val(value.swap_bytes() as i64)
}

pub unsafe extern "C" fn int_native_rotl32(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i32() } as u32;
    let distance = unsafe { (*args.add(1)).as_i32() } as u32;
    SolixCValue::i32_val(value.rotate_left(distance) as i32)
}

pub unsafe extern "C" fn int_native_rotr32(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i32() } as u32;
    let distance = unsafe { (*args.add(1)).as_i32() } as u32;
    SolixCValue::i32_val(value.rotate_right(distance) as i32)
}

pub unsafe extern "C" fn int_native_rotl64(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i64() } as u64;
    let distance = unsafe { (*args.add(1)).as_i32() } as u32;
    SolixCValue::i64_val(value.rotate_left(distance) as i64)
}

pub unsafe extern "C" fn int_native_rotr64(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i64() } as u64;
    let distance = unsafe { (*args.add(1)).as_i32() } as u32;
    SolixCValue::i64_val(value.rotate_right(distance) as i64)
}

pub unsafe extern "C" fn int_native_ushr32(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i32() } as u32;
    let shift = (unsafe { (*args.add(1)).as_i32() } as u32) & 31;
    SolixCValue::i32_val((value >> shift) as i32)
}

pub unsafe extern "C" fn int_native_ushr64(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let value = unsafe { (*args.add(0)).as_i64() } as u64;
    let shift = (unsafe { (*args.add(1)).as_i32() } as u32) & 63;
    SolixCValue::i64_val((value >> shift) as i64)
}

pub unsafe extern "C" fn prim_native_float_to_raw_bits(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let float_val = unsafe { (*args.add(0)).as_f64() } as f32;
    SolixCValue::i32_val(float_val.to_bits() as i32)
}

pub unsafe extern "C" fn prim_native_raw_bits_to_float(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let bits = unsafe { (*args.add(0)).as_i32() } as u32;
    SolixCValue::f64_val(f32::from_bits(bits) as f64)
}

pub unsafe extern "C" fn prim_native_double_to_raw_bits(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let double_val = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::i64_val(double_val.to_bits() as i64)
}

pub unsafe extern "C" fn prim_native_raw_bits_to_double(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let bits = unsafe { (*args.add(0)).as_i64() } as u64;
    SolixCValue::f64_val(f64::from_bits(bits))
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

// --- C++ std::format-Style Engine ---

fn parse_as_integer(raw_text: &str) -> i64 {
    if let Ok(parsed_val) = raw_text.trim().parse::<i64>() {
        parsed_val
    } else if let Ok(parsed_float) = raw_text.trim().parse::<f64>() {
        parsed_float as i64
    } else {
        0
    }
}

fn parse_as_float(raw_text: &str) -> f64 {
    raw_text.trim().parse::<f64>().unwrap_or(0.0)
}

fn pad_numeric_zero(numeric_text: &str, target_width: usize) -> String {
    if numeric_text.len() >= target_width {
        return numeric_text.to_string();
    }
    let has_sign = numeric_text.starts_with('+') || numeric_text.starts_with('-');
    let prefix = if has_sign { &numeric_text[..1] } else { "" };
    let body = if has_sign {
        &numeric_text[1..]
    } else {
        numeric_text
    };
    let zeros_needed = target_width.saturating_sub(prefix.len() + body.len());
    let mut padded_result = String::with_capacity(target_width);
    padded_result.push_str(prefix);
    padded_result.extend(std::iter::repeat_n('0', zeros_needed));
    padded_result.push_str(body);
    padded_result
}

fn format_radix_with_alternate_and_width(
    digits_text: &str,
    alternate_form: bool,
    prefix: &str,
    zero_pad: bool,
    width_value: Option<usize>,
) -> String {
    let prefix_length = if alternate_form { prefix.len() } else { 0 };
    if zero_pad && width_value.is_some() {
        let target_width = width_value.unwrap_or(0);
        let digits_target = target_width.saturating_sub(prefix_length);
        let mut result = String::new();
        if alternate_form {
            result.push_str(prefix);
        }
        if digits_text.len() < digits_target {
            let zeros_needed = digits_target - digits_text.len();
            result.extend(std::iter::repeat_n('0', zeros_needed));
        }
        result.push_str(digits_text);
        result
    } else {
        let mut result = String::new();
        if alternate_form {
            result.push_str(prefix);
        }
        result.push_str(digits_text);
        result
    }
}

pub fn format_with_specifier(value_text: &str, specifier: &str) -> Result<String, String> {
    if specifier.is_empty() {
        return Ok(value_text.to_string());
    }

    let characters: Vec<char> = specifier.chars().collect();
    let mut cursor = 0;
    let length = characters.len();

    // 1. [fill]align
    let mut fill_character = ' ';
    let mut alignment = None;

    if cursor + 1 < length && matches!(characters[cursor + 1], '<' | '>' | '^') {
        fill_character = characters[cursor];
        alignment = Some(characters[cursor + 1]);
        cursor += 2;
    } else if cursor < length && matches!(characters[cursor], '<' | '>' | '^') {
        alignment = Some(characters[cursor]);
        cursor += 1;
    }

    // 2. sign (+, -, ' ')
    let mut sign_option = None;
    if cursor < length && matches!(characters[cursor], '+' | '-' | ' ') {
        sign_option = Some(characters[cursor]);
        cursor += 1;
    }

    // 3. alternate '#'
    let mut alternate_form = false;
    if cursor < length && characters[cursor] == '#' {
        alternate_form = true;
        cursor += 1;
    }

    // 4. zero padding '0'
    let mut zero_pad = false;
    if cursor < length && characters[cursor] == '0' {
        zero_pad = true;
        cursor += 1;
        if alignment.is_none() {
            alignment = Some('>');
            fill_character = '0';
        }
    }

    // 5. width
    let mut width_value: Option<usize> = None;
    let width_start = cursor;
    while cursor < length && characters[cursor].is_ascii_digit() {
        cursor += 1;
    }
    if cursor > width_start {
        let width_text: String = characters[width_start..cursor].iter().collect();
        width_value = width_text.parse::<usize>().ok();
    }

    // 6. precision
    let mut precision_value: Option<usize> = None;
    if cursor < length && characters[cursor] == '.' {
        cursor += 1;
        let precision_start = cursor;
        while cursor < length && characters[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor > precision_start {
            let precision_text: String = characters[precision_start..cursor].iter().collect();
            precision_value = precision_text.parse::<usize>().ok();
        } else {
            precision_value = Some(0);
        }
    }

    // 7. type
    let mut type_character: Option<char> = None;
    if cursor < length {
        let candidate = characters[cursor];
        if matches!(
            candidate,
            'd' | 'i' | 'x' | 'X' | 'b' | 'o' | 'f' | 'F' | 'e' | 'E' | 's'
        ) {
            type_character = Some(candidate);
            cursor += 1;
        } else {
            return Err(format!(
                "[E0331] Invalid format specifier in template string: unexpected type character '{candidate}'"
            ));
        }
    }

    if cursor < length {
        let remaining: String = characters[cursor..].iter().collect();
        return Err(format!(
            "[E0331] Invalid format specifier in template string: trailing garbage '{remaining}'"
        ));
    }

    // Format based on type
    let formatted_content = match type_character {
        Some('x') => {
            let integer_value = parse_as_integer(value_text);
            let hex_text = format!("{integer_value:x}");
            format_radix_with_alternate_and_width(
                &hex_text,
                alternate_form,
                "0x",
                zero_pad,
                width_value,
            )
        }
        Some('X') => {
            let integer_value = parse_as_integer(value_text);
            let hex_text = format!("{integer_value:X}");
            format_radix_with_alternate_and_width(
                &hex_text,
                alternate_form,
                "0X",
                zero_pad,
                width_value,
            )
        }
        Some('b') => {
            let integer_value = parse_as_integer(value_text);
            let bin_text = format!("{integer_value:b}");
            format_radix_with_alternate_and_width(
                &bin_text,
                alternate_form,
                "0b",
                zero_pad,
                width_value,
            )
        }
        Some('o') => {
            let integer_value = parse_as_integer(value_text);
            let oct_text = format!("{integer_value:o}");
            format_radix_with_alternate_and_width(
                &oct_text,
                alternate_form,
                "0o",
                zero_pad,
                width_value,
            )
        }
        Some('d') | Some('i') => {
            let integer_value = parse_as_integer(value_text);
            let mut integer_text = integer_value.to_string();
            if integer_value >= 0 {
                if sign_option == Some('+') {
                    integer_text.insert(0, '+');
                } else if sign_option == Some(' ') {
                    integer_text.insert(0, ' ');
                }
            }
            if zero_pad && width_value.is_some() {
                pad_numeric_zero(&integer_text, width_value.unwrap_or(0))
            } else {
                integer_text
            }
        }
        Some('f') | Some('F') => {
            let float_value = parse_as_float(value_text);
            let precision = precision_value.unwrap_or(6);
            let mut float_text = format!("{float_value:.precision$}");
            if float_value >= 0.0 {
                if sign_option == Some('+') {
                    float_text.insert(0, '+');
                } else if sign_option == Some(' ') {
                    float_text.insert(0, ' ');
                }
            }
            if zero_pad && width_value.is_some() {
                pad_numeric_zero(&float_text, width_value.unwrap_or(0))
            } else {
                float_text
            }
        }
        Some('e') => {
            let float_value = parse_as_float(value_text);
            let precision = precision_value.unwrap_or(6);
            format!("{float_value:.precision$e}")
        }
        Some('E') => {
            let float_value = parse_as_float(value_text);
            let precision = precision_value.unwrap_or(6);
            format!("{float_value:.precision$E}")
        }
        Some('s') | None => {
            if let Some(precision) = precision_value {
                if value_text.len() > precision {
                    value_text[..precision].to_string()
                } else {
                    value_text.to_string()
                }
            } else if zero_pad && width_value.is_some() {
                pad_numeric_zero(value_text, width_value.unwrap_or(0))
            } else {
                value_text.to_string()
            }
        }
        _ => value_text.to_string(),
    };

    // Apply alignment & width padding if not already padded by radix/zero-pad
    if let Some(target_width) = width_value {
        if formatted_content.len() < target_width && !zero_pad {
            let padding_needed = target_width - formatted_content.len();
            let aligned_result = match alignment.unwrap_or('<') {
                '<' => {
                    let mut result_string = formatted_content;
                    result_string.extend(std::iter::repeat_n(fill_character, padding_needed));
                    result_string
                }
                '>' => {
                    let mut result_string: String =
                        std::iter::repeat_n(fill_character, padding_needed).collect();
                    result_string.push_str(&formatted_content);
                    result_string
                }
                '^' => {
                    let left_pad = padding_needed / 2;
                    let right_pad = padding_needed - left_pad;
                    let mut result_string: String =
                        std::iter::repeat_n(fill_character, left_pad).collect();
                    result_string.push_str(&formatted_content);
                    result_string.extend(std::iter::repeat_n(fill_character, right_pad));
                    result_string
                }
                _ => formatted_content,
            };
            return Ok(aligned_result);
        }
    }

    Ok(formatted_content)
}

pub fn format_template_with_arguments(
    template: &str,
    arguments: &[String],
) -> Result<String, String> {
    let mut result = String::with_capacity(template.len() + 32);
    let characters: Vec<char> = template.chars().collect();
    let mut cursor = 0;
    let length = characters.len();
    let mut sequential_index = 0;

    while cursor < length {
        let current_character = characters[cursor];
        if current_character == '{' {
            if cursor + 1 < length && characters[cursor + 1] == '{' {
                result.push('{');
                cursor += 2;
                continue;
            }
            let mut close_index = cursor + 1;
            while close_index < length && characters[close_index] != '}' {
                close_index += 1;
            }
            if close_index >= length {
                return Err(format!(
                    "[E0331] Invalid format specifier in template string: unclosed '{{' at index {cursor}"
                ));
            }
            let inside: String = characters[cursor + 1..close_index].iter().collect();
            cursor = close_index + 1;

            let (target_index, specifier) = if let Some(colon_pos) = inside.find(':') {
                let index_part = &inside[..colon_pos];
                let spec_part = &inside[colon_pos + 1..];
                let resolved_index = if index_part.is_empty() {
                    let current_seq = sequential_index;
                    sequential_index += 1;
                    current_seq
                } else if let Ok(parsed) = index_part.parse::<usize>() {
                    parsed
                } else {
                    return Err(format!(
                        "[E0331] Invalid format specifier in template string: invalid index '{index_part}'"
                    ));
                };
                (resolved_index, spec_part)
            } else {
                let resolved_index = if inside.is_empty() {
                    let current_seq = sequential_index;
                    sequential_index += 1;
                    current_seq
                } else if let Ok(parsed) = inside.parse::<usize>() {
                    parsed
                } else {
                    return Err(format!(
                        "[E0331] Invalid format specifier in template string: invalid index '{inside}'"
                    ));
                };
                (resolved_index, "")
            };

            let argument_value = arguments
                .get(target_index)
                .map(|arg_item| arg_item.as_str())
                .unwrap_or("");
            let formatted_piece = format_with_specifier(argument_value, specifier)?;
            result.push_str(&formatted_piece);
        } else if current_character == '}' {
            if cursor + 1 < length && characters[cursor + 1] == '}' {
                result.push('}');
                cursor += 2;
            } else {
                return Err(format!(
                    "[E0331] Invalid format specifier in template string: single '}}' at index {cursor}"
                ));
            }
        } else {
            result.push(current_character);
            cursor += 1;
        }
    }

    Ok(result)
}

pub unsafe extern "C" fn string_native_format(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 1 || args.is_null() {
        return SolixCValue::null();
    }
    let template_address = unsafe { (*args.add(0)).as_ref() };
    let template_string = unsafe { read_solix_chars(vm_context, template_address) };

    let mut arguments_list = Vec::new();
    if arg_count >= 2 {
        let array_address = unsafe { (*args.add(1)).as_ref() };
        if array_address != 0 {
            if let Some(context_ref) = unsafe { crate::validate_vm_context(vm_context) } {
                let array_length =
                    unsafe { (context_ref.array_len)(context_ref.vm_ptr, array_address) };
                for argument_index in 0..array_length {
                    let element_raw = unsafe {
                        (context_ref.array_get_u64)(
                            context_ref.vm_ptr,
                            array_address,
                            argument_index,
                        )
                    };
                    let element_address = element_raw as u32;
                    let element_string = unsafe { read_solix_chars(vm_context, element_address) };
                    arguments_list.push(element_string);
                }
            }
        }
    }

    match format_template_with_arguments(&template_string, &arguments_list) {
        Ok(formatted_text) => {
            let allocated_address = unsafe { alloc_solix_chars(vm_context, &formatted_text) };
            SolixCValue::ref_val(allocated_address)
        }
        Err(error_message) => {
            unsafe { crate::set_native_error(vm_context, out_error, &error_message) };
            SolixCValue::null()
        }
    }
}

pub unsafe extern "C" fn string_native_format_spec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::null();
    }
    let value_address = unsafe { (*args.add(0)).as_ref() };
    let spec_address = unsafe { (*args.add(1)).as_ref() };
    let value_string = unsafe { read_solix_chars(vm_context, value_address) };
    let spec_string = unsafe { read_solix_chars(vm_context, spec_address) };

    match format_with_specifier(&value_string, &spec_string) {
        Ok(formatted_text) => {
            let allocated_address = unsafe { alloc_solix_chars(vm_context, &formatted_text) };
            SolixCValue::ref_val(allocated_address)
        }
        Err(error_message) => {
            unsafe { crate::set_native_error(vm_context, out_error, &error_message) };
            SolixCValue::null()
        }
    }
}

pub unsafe fn register_string_prim_natives(
    collector: *mut c_void,
    callback: RegisterNativeCallback,
) {
    let string_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "String_native_format",
                "solix_core_String_native_format",
                "String::native_format",
            ],
            string_native_format,
        ),
        (
            &[
                "String_native_format_spec",
                "solix_core_String_native_format_spec",
                "String::native_format_spec",
            ],
            string_native_format_spec,
        ),
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
        (
            &[
                "String_native_chars_to_string",
                "solix_core_String_native_chars_to_string",
                "String::native_chars_to_string",
            ],
            string_native_chars_to_string,
        ),
        (
            &[
                "String_native_string_to_chars",
                "solix_core_String_native_string_to_chars",
                "String::native_string_to_chars",
            ],
            string_native_string_to_chars,
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
        (
            &[
                "Primitives_native_clz64",
                "solix_core_Primitives_native_clz64",
                "Primitives::native_clz64",
            ],
            int_native_clz64,
        ),
        (
            &[
                "Primitives_native_ctz64",
                "solix_core_Primitives_native_ctz64",
                "Primitives::native_ctz64",
            ],
            int_native_ctz64,
        ),
        (
            &[
                "Primitives_native_popcount64",
                "solix_core_Primitives_native_popcount64",
                "Primitives::native_popcount64",
            ],
            int_native_popcount64,
        ),
        (
            &[
                "Primitives_native_bswap64",
                "solix_core_Primitives_native_bswap64",
                "Primitives::native_bswap64",
            ],
            int_native_bswap64,
        ),
        (
            &[
                "Primitives_native_rotl32",
                "solix_core_Primitives_native_rotl32",
                "Primitives::native_rotl32",
            ],
            int_native_rotl32,
        ),
        (
            &[
                "Primitives_native_rotr32",
                "solix_core_Primitives_native_rotr32",
                "Primitives::native_rotr32",
            ],
            int_native_rotr32,
        ),
        (
            &[
                "Primitives_native_rotl64",
                "solix_core_Primitives_native_rotl64",
                "Primitives::native_rotl64",
            ],
            int_native_rotl64,
        ),
        (
            &[
                "Primitives_native_rotr64",
                "solix_core_Primitives_native_rotr64",
                "Primitives::native_rotr64",
            ],
            int_native_rotr64,
        ),
        (
            &[
                "Primitives_native_ushr32",
                "solix_core_Primitives_native_ushr32",
                "Primitives::native_ushr32",
            ],
            int_native_ushr32,
        ),
        (
            &[
                "Primitives_native_ushr64",
                "solix_core_Primitives_native_ushr64",
                "Primitives::native_ushr64",
            ],
            int_native_ushr64,
        ),
        (
            &[
                "Primitives_native_float_to_raw_bits",
                "solix_core_Primitives_native_float_to_raw_bits",
                "Primitives::native_float_to_raw_bits",
            ],
            prim_native_float_to_raw_bits,
        ),
        (
            &[
                "Primitives_native_raw_bits_to_float",
                "solix_core_Primitives_native_raw_bits_to_float",
                "Primitives::native_raw_bits_to_float",
            ],
            prim_native_raw_bits_to_float,
        ),
        (
            &[
                "Primitives_native_double_to_raw_bits",
                "solix_core_Primitives_native_double_to_raw_bits",
                "Primitives::native_double_to_raw_bits",
            ],
            prim_native_double_to_raw_bits,
        ),
        (
            &[
                "Primitives_native_raw_bits_to_double",
                "solix_core_Primitives_native_raw_bits_to_double",
                "Primitives::native_raw_bits_to_double",
            ],
            prim_native_raw_bits_to_double,
        ),
    ];

    for (variants, func) in int_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
