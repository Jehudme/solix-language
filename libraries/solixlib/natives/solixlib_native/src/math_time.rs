//! Mathematical transcendental functions, clocks, and time formatting native functions.

use crate::{
    alloc_solix_chars, register_symbol_variants, NativeCFunctionPtr, RegisterNativeCallback,
    SolixCValue, SolixVmContext, SOLIX_VM_CONTEXT_MAGIC,
};
use std::ffi::c_void;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static NEXT_RANDOM_HANDLE: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(1));
static RANDOM_STATES: LazyLock<Mutex<std::collections::HashMap<u64, u64>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

// --- Math Natives ---

pub unsafe extern "C" fn math_native_sqrt(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.sqrt())
}

pub unsafe extern "C" fn math_native_cbrt(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.cbrt())
}

pub unsafe extern "C" fn math_native_hypot(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let x_value = unsafe { (*args.add(0)).as_f64() };
    let y_value = unsafe { (*args.add(1)).as_f64() };
    SolixCValue::f64_val(x_value.hypot(y_value))
}

pub unsafe extern "C" fn math_native_pow(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let base_val = unsafe { (*args.add(0)).as_f64() };
    let exponent_val = unsafe { (*args.add(1)).as_f64() };
    SolixCValue::f64_val(base_val.powf(exponent_val))
}

pub unsafe extern "C" fn math_native_exp(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.exp())
}

pub unsafe extern "C" fn math_native_log(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.ln())
}

pub unsafe extern "C" fn math_native_log10(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.log10())
}

pub unsafe extern "C" fn math_native_log2(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.log2())
}

pub unsafe extern "C" fn math_native_sin(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.sin())
}

pub unsafe extern "C" fn math_native_cos(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.cos())
}

pub unsafe extern "C" fn math_native_tan(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.tan())
}

pub unsafe extern "C" fn math_native_asin(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.asin())
}

pub unsafe extern "C" fn math_native_acos(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.acos())
}

pub unsafe extern "C" fn math_native_atan(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.atan())
}

pub unsafe extern "C" fn math_native_atan2(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let y_value = unsafe { (*args.add(0)).as_f64() };
    let x_value = unsafe { (*args.add(1)).as_f64() };
    SolixCValue::f64_val(y_value.atan2(x_value))
}

pub unsafe extern "C" fn math_native_sinh(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.sinh())
}

pub unsafe extern "C" fn math_native_cosh(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.cosh())
}

pub unsafe extern "C" fn math_native_tanh(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.tanh())
}

pub unsafe extern "C" fn math_native_floor(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.floor())
}

pub unsafe extern "C" fn math_native_ceil(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.ceil())
}

pub unsafe extern "C" fn math_native_round(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.round())
}

pub unsafe extern "C" fn math_native_trunc(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let value = unsafe { (*args.add(0)).as_f64() };
    SolixCValue::f64_val(value.trunc())
}

// --- PRNG Natives ---

pub unsafe extern "C" fn random_native_seed(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let seed_value = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0x1234_5678_9ABC_DEF0);
    SolixCValue::i64_val(seed_value as i64)
}

pub unsafe extern "C" fn random_native_init(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let initial_seed = if arg_count > 0 && !args.is_null() {
        let provided = unsafe { (*args.add(0)).payload };
        if provided != 0 {
            provided
        } else {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos() as u64)
                .unwrap_or(0x1234_5678)
        }
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x1234_5678)
    };

    let mut next_handle_guard = NEXT_RANDOM_HANDLE.lock().unwrap();
    let assigned_handle = *next_handle_guard;
    *next_handle_guard += 1;
    RANDOM_STATES
        .lock()
        .unwrap()
        .insert(assigned_handle, initial_seed);
    SolixCValue::i64_val(assigned_handle as i64)
}

pub unsafe extern "C" fn random_native_next_u64(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let handle = unsafe { (*args.add(0)).payload };
    let mut states_guard = RANDOM_STATES.lock().unwrap();
    if let Some(state) = states_guard.get_mut(&handle) {
        let generated = splitmix64(state);
        SolixCValue::i64_val(generated as i64)
    } else {
        SolixCValue::i64_val(0)
    }
}

pub unsafe extern "C" fn random_native_next_double(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::f64_val(0.0);
    }
    let handle = unsafe { (*args.add(0)).payload };
    let mut states_guard = RANDOM_STATES.lock().unwrap();
    if let Some(state) = states_guard.get_mut(&handle) {
        let generated = splitmix64(state) >> 11;
        let unit_float = (generated as f64) * (1.0 / (1u64 << 53) as f64);
        SolixCValue::f64_val(unit_float)
    } else {
        SolixCValue::f64_val(0.0)
    }
}

// --- Time Natives ---

pub unsafe extern "C" fn time_native_steady_nanos(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    static START_INSTANT: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);
    let elapsed = START_INSTANT.elapsed();
    SolixCValue::i64_val(elapsed.as_nanos() as i64)
}

pub unsafe extern "C" fn time_native_system_millis(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0);
    SolixCValue::i64_val(millis)
}

pub unsafe extern "C" fn time_native_utc_parts(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() || vm_context.is_null() {
        return SolixCValue::null();
    }
    let epoch_millis = unsafe { (*args.add(0)).as_i64() };
    let out_array_address = unsafe { (*args.add(1)).as_ref() };

    let total_seconds = epoch_millis.div_euclid(1000);
    let remainder_millis = epoch_millis.rem_euclid(1000);

    // Days since UNIX epoch
    let days = total_seconds.div_euclid(86400);
    let second_of_day = total_seconds.rem_euclid(86400);
    let hour = second_of_day / 3600;
    let minute = (second_of_day % 3600) / 60;
    let second = second_of_day % 60;

    // Civil date from days calculation (Hinnant algorithm)
    let shifted_days = days + 719_468;
    let era = if shifted_days >= 0 {
        shifted_days / 146_097
    } else {
        (shifted_days - 146_096) / 146_097
    };
    let day_of_era = (shifted_days - era * 146_097) as u32;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146_096) / 365;
    let year = (year_of_era as i64) + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = (month_prime as i64) + if month_prime < 10 { 3 } else { -9 };
    let final_year = year + if month <= 2 { 1 } else { 0 };

    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic == SOLIX_VM_CONTEXT_MAGIC {
        unsafe {
            (context_ref.array_set_u64)(
                context_ref.vm_ptr,
                out_array_address,
                0,
                final_year as u64,
            );
            (context_ref.array_set_u64)(context_ref.vm_ptr, out_array_address, 1, month as u64);
            (context_ref.array_set_u64)(context_ref.vm_ptr, out_array_address, 2, day as u64);
            (context_ref.array_set_u64)(context_ref.vm_ptr, out_array_address, 3, hour as u64);
            (context_ref.array_set_u64)(context_ref.vm_ptr, out_array_address, 4, minute as u64);
            (context_ref.array_set_u64)(context_ref.vm_ptr, out_array_address, 5, second as u64);
            (context_ref.array_set_u64)(
                context_ref.vm_ptr,
                out_array_address,
                6,
                remainder_millis as u64,
            );
        }
    }

    SolixCValue::null()
}

pub unsafe extern "C" fn time_native_local_parts(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    // For deterministic portability, local parts delegates to utc parts
    unsafe { time_native_utc_parts(vm_context, args, arg_count, out_error) }
}

pub unsafe extern "C" fn time_native_iso8601(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let epoch_millis = unsafe { (*args.add(0)).as_i64() };
    let total_seconds = epoch_millis.div_euclid(1000);
    let remainder_millis = epoch_millis.rem_euclid(1000);

    let days = total_seconds.div_euclid(86400);
    let second_of_day = total_seconds.rem_euclid(86400);
    let hour = second_of_day / 3600;
    let minute = (second_of_day % 3600) / 60;
    let second = second_of_day % 60;

    let shifted_days = days + 719_468;
    let era = if shifted_days >= 0 {
        shifted_days / 146_097
    } else {
        (shifted_days - 146_096) / 146_097
    };
    let day_of_era = (shifted_days - era * 146_097) as u32;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146_096) / 365;
    let year = (year_of_era as i64) + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = (month_prime as i64) + if month_prime < 10 { 3 } else { -9 };
    let final_year = year + if month <= 2 { 1 } else { 0 };

    let formatted_iso = format!(
        "{final_year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{remainder_millis:03}Z"
    );
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &formatted_iso) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn time_native_sleep_millis(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let milliseconds = unsafe { (*args.add(0)).as_i64() };
        if milliseconds > 0 {
            std::thread::sleep(Duration::from_millis(milliseconds as u64));
        }
    }
    SolixCValue::null()
}

pub unsafe fn register_math_time_natives(collector: *mut c_void, callback: RegisterNativeCallback) {
    let math_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Math_native_sqrt",
                "solix_math_Math_native_sqrt",
                "Math::native_sqrt",
            ],
            math_native_sqrt,
        ),
        (
            &[
                "Math_native_cbrt",
                "solix_math_Math_native_cbrt",
                "Math::native_cbrt",
            ],
            math_native_cbrt,
        ),
        (
            &[
                "Math_native_hypot",
                "solix_math_Math_native_hypot",
                "Math::native_hypot",
            ],
            math_native_hypot,
        ),
        (
            &[
                "Math_native_pow",
                "solix_math_Math_native_pow",
                "Math::native_pow",
            ],
            math_native_pow,
        ),
        (
            &[
                "Math_native_exp",
                "solix_math_Math_native_exp",
                "Math::native_exp",
            ],
            math_native_exp,
        ),
        (
            &[
                "Math_native_log",
                "solix_math_Math_native_log",
                "Math::native_log",
            ],
            math_native_log,
        ),
        (
            &[
                "Math_native_log10",
                "solix_math_Math_native_log10",
                "Math::native_log10",
            ],
            math_native_log10,
        ),
        (
            &[
                "Math_native_log2",
                "solix_math_Math_native_log2",
                "Math::native_log2",
            ],
            math_native_log2,
        ),
        (
            &[
                "Math_native_sin",
                "solix_math_Math_native_sin",
                "Math::native_sin",
            ],
            math_native_sin,
        ),
        (
            &[
                "Math_native_cos",
                "solix_math_Math_native_cos",
                "Math::native_cos",
            ],
            math_native_cos,
        ),
        (
            &[
                "Math_native_tan",
                "solix_math_Math_native_tan",
                "Math::native_tan",
            ],
            math_native_tan,
        ),
        (
            &[
                "Math_native_asin",
                "solix_math_Math_native_asin",
                "Math::native_asin",
            ],
            math_native_asin,
        ),
        (
            &[
                "Math_native_acos",
                "solix_math_Math_native_acos",
                "Math::native_acos",
            ],
            math_native_acos,
        ),
        (
            &[
                "Math_native_atan",
                "solix_math_Math_native_atan",
                "Math::native_atan",
            ],
            math_native_atan,
        ),
        (
            &[
                "Math_native_atan2",
                "solix_math_Math_native_atan2",
                "Math::native_atan2",
            ],
            math_native_atan2,
        ),
        (
            &[
                "Math_native_sinh",
                "solix_math_Math_native_sinh",
                "Math::native_sinh",
            ],
            math_native_sinh,
        ),
        (
            &[
                "Math_native_cosh",
                "solix_math_Math_native_cosh",
                "Math::native_cosh",
            ],
            math_native_cosh,
        ),
        (
            &[
                "Math_native_tanh",
                "solix_math_Math_native_tanh",
                "Math::native_tanh",
            ],
            math_native_tanh,
        ),
        (
            &[
                "Math_native_floor",
                "solix_math_Math_native_floor",
                "Math::native_floor",
            ],
            math_native_floor,
        ),
        (
            &[
                "Math_native_ceil",
                "solix_math_Math_native_ceil",
                "Math::native_ceil",
            ],
            math_native_ceil,
        ),
        (
            &[
                "Math_native_round",
                "solix_math_Math_native_round",
                "Math::native_round",
            ],
            math_native_round,
        ),
        (
            &[
                "Math_native_trunc",
                "solix_math_Math_native_trunc",
                "Math::native_trunc",
            ],
            math_native_trunc,
        ),
        (
            &[
                "Random_native_seed",
                "solix_math_Random_native_seed",
                "Random::native_seed",
            ],
            random_native_seed,
        ),
        (
            &[
                "Random_native_init",
                "solix_math_Random_native_init",
                "Random::native_init",
            ],
            random_native_init,
        ),
        (
            &[
                "Random_native_next_u64",
                "solix_math_Random_native_next_u64",
                "Random::native_next_u64",
            ],
            random_native_next_u64,
        ),
        (
            &[
                "Random_native_next_double",
                "solix_math_Random_native_next_double",
                "Random::native_next_double",
            ],
            random_native_next_double,
        ),
    ];

    for (variants, func) in math_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let time_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Instant_native_steady_nanos",
                "solix_time_Instant_native_steady_nanos",
                "Instant::native_steady_nanos",
            ],
            time_native_steady_nanos,
        ),
        (
            &[
                "DateTime_native_system_millis",
                "solix_time_DateTime_native_system_millis",
                "DateTime::native_system_millis",
            ],
            time_native_system_millis,
        ),
        (
            &[
                "DateTime_native_utc_parts",
                "solix_time_DateTime_native_utc_parts",
                "DateTime::native_utc_parts",
            ],
            time_native_utc_parts,
        ),
        (
            &[
                "DateTime_native_local_parts",
                "solix_time_DateTime_native_local_parts",
                "DateTime::native_local_parts",
            ],
            time_native_local_parts,
        ),
        (
            &[
                "DateTime_native_iso8601",
                "solix_time_DateTime_native_iso8601",
                "DateTime::native_iso8601",
            ],
            time_native_iso8601,
        ),
        (
            &[
                "Stopwatch_native_steady_nanos",
                "solix_time_Stopwatch_native_steady_nanos",
                "Stopwatch::native_steady_nanos",
            ],
            time_native_steady_nanos,
        ),
        (
            &[
                "Stopwatch_native_sleep_millis",
                "solix_time_Stopwatch_native_sleep_millis",
                "Stopwatch::native_sleep_millis",
            ],
            time_native_sleep_millis,
        ),
    ];

    for (variants, func) in time_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
