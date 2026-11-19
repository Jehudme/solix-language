//! Dual-mode Rust backend for the `ffi_dual_backend` matrix showcase.
//!
//! Compiles standalone (`cargo test` / `cargo build`) and as a Solix VM plugin
//! (`solix build --profile rust_backend`, enabling `--features solix-native`).

pub fn matrix_add_i32(left_val: i32, right_val: i32) -> i32 {
    left_val + right_val
}

pub fn matrix_mul_i64(left_val: i64, right_val: i64) -> i64 {
    left_val * right_val
}

pub fn matrix_fma_f64(factor_a: f64, factor_b: f64, offset_c: f64) -> f64 {
    (factor_a * factor_b) + offset_c
}

pub fn matrix_reverse_ascii(input_text: &str) -> String {
    input_text.chars().rev().collect()
}

pub fn matrix_upper_ascii(input_bytes: &[u8]) -> Vec<u8> {
    input_bytes.iter().map(u8::to_ascii_uppercase).collect()
}

pub fn matrix_xor_bytes(input_bytes: &[u8], mask: u8) -> Vec<u8> {
    input_bytes.iter().map(|byte_val| byte_val ^ mask).collect()
}

#[cfg(feature = "solix-native")]
mod solix_bridge {
    use super::*;
    use solix_ffi::{
        SOLIX_ARRAY_TAG_I64, SOLIX_ARRAY_TAG_REF, SolixCValue, SolixVmContext, solix_export,
    };
    use std::ffi::c_void;

    unsafe fn set_vm_error(vm_context: *mut c_void, out_error: *mut SolixCValue, msg: &str) {
        if out_error.is_null() {
            return;
        }
        if let Some(ctx) = unsafe { SolixVmContext::validate(vm_context) } {
            let err_addr = unsafe { ctx.alloc_string(msg) };
            if err_addr != 0 {
                unsafe {
                    *out_error = SolixCValue::from_ref(err_addr);
                }
                return;
            }
        }
        unsafe {
            *out_error = SolixCValue::from_bool(false);
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_echo_null(
        _vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        if argc < 1 || args.is_null() || unsafe { (*args).is_null() } {
            SolixCValue::null()
        } else {
            unsafe { *args }
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_invert_bool(
        _vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        if argc < 1 || args.is_null() {
            return SolixCValue::from_bool(false);
        }
        SolixCValue::from_bool(!unsafe { (*args).as_bool() })
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_add_i32(
        _vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        if argc < 2 || args.is_null() {
            return SolixCValue::from_i32(0);
        }
        SolixCValue::from_i32(matrix_add_i32(
            unsafe { (*args.add(0)).as_i32() },
            unsafe { (*args.add(1)).as_i32() },
        ))
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_mul_i64(
        _vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        if argc < 2 || args.is_null() {
            return SolixCValue::from_i64(0);
        }
        SolixCValue::from_i64(matrix_mul_i64(
            unsafe { (*args.add(0)).as_i64() },
            unsafe { (*args.add(1)).as_i64() },
        ))
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_fma_f64(
        _vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        if argc < 3 || args.is_null() {
            return SolixCValue::from_f64(0.0);
        }
        SolixCValue::from_f64(matrix_fma_f64(
            unsafe { (*args.add(0)).as_f64() },
            unsafe { (*args.add(1)).as_f64() },
            unsafe { (*args.add(2)).as_f64() },
        ))
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_reverse_string(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            unsafe { set_vm_error(vm, out_err, "invalid VM context") };
            return SolixCValue::null();
        };
        if argc < 1 || args.is_null() {
            return SolixCValue::null();
        }
        let text = unsafe { ctx.read_string((*args).as_ref()) }.unwrap_or_default();
        let reversed = matrix_reverse_ascii(&text);
        SolixCValue::from_ref(unsafe { ctx.alloc_string(&reversed) })
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_upper_chars(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            return SolixCValue::null();
        };
        if argc < 1 || args.is_null() {
            return SolixCValue::null();
        }
        let addr = unsafe { (*args).as_ref() };
        let len = unsafe { (ctx.read_bytes)(ctx.vm_ptr, addr, std::ptr::null_mut(), 0) };
        let mut buf = vec![0u8; len as usize];
        if len > 0 {
            unsafe { (ctx.read_bytes)(ctx.vm_ptr, addr, buf.as_mut_ptr(), len) };
        }
        let upper = matrix_upper_ascii(&buf);
        let text = String::from_utf8_lossy(&upper);
        SolixCValue::from_ref(unsafe { ctx.alloc_chars(&text) })
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_xor_bytes(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            return SolixCValue::null();
        };
        if argc < 2 || args.is_null() {
            return SolixCValue::null();
        }
        let addr = unsafe { (*args.add(0)).as_ref() };
        let mask = (unsafe { (*args.add(1)).as_i32() } & 0xFF) as u8;
        let len = unsafe { (ctx.read_bytes)(ctx.vm_ptr, addr, std::ptr::null_mut(), 0) };
        let mut buf = vec![0u8; len as usize];
        if len > 0 {
            unsafe { (ctx.read_bytes)(ctx.vm_ptr, addr, buf.as_mut_ptr(), len) };
        }
        let out_bytes = matrix_xor_bytes(&buf, mask);
        SolixCValue::from_ref(unsafe { ctx.alloc_byte_array(&out_bytes) })
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_scale_i64_array(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            return SolixCValue::null();
        };
        if argc < 2 || args.is_null() {
            return SolixCValue::null();
        }
        let src_addr = unsafe { (*args.add(0)).as_ref() };
        let factor = unsafe { (*args.add(1)).as_i64() };
        let count = unsafe { ctx.array_len(src_addr) };
        let dst_addr = unsafe { ctx.alloc_array(count, SOLIX_ARRAY_TAG_I64) };
        for idx in 0..count {
            let elem = unsafe { ctx.array_get_u64(src_addr, idx) } as i64;
            unsafe { ctx.array_set_u64(dst_addr, idx, (elem * factor) as u64) };
        }
        SolixCValue::from_ref(dst_addr)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_sum_f64_array(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            return SolixCValue::from_f64(0.0);
        };
        if argc < 1 || args.is_null() {
            return SolixCValue::from_f64(0.0);
        }
        let src_addr = unsafe { (*args).as_ref() };
        let count = unsafe { ctx.array_len(src_addr) };
        let mut total = 0.0_f64;
        for idx in 0..count {
            let raw_bits = unsafe { ctx.array_get_u64(src_addr, idx) };
            total += f64::from_bits(raw_bits);
        }
        SolixCValue::from_f64(total)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_make_pair_strings(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            return SolixCValue::null();
        };
        if argc < 2 || args.is_null() {
            return SolixCValue::null();
        }
        let left_text = unsafe { ctx.read_string((*args.add(0)).as_ref()) }.unwrap_or_default();
        let right_text = unsafe { ctx.read_string((*args.add(1)).as_ref()) }.unwrap_or_default();
        let str_a = unsafe { ctx.alloc_string(&left_text) };
        let str_b = unsafe { ctx.alloc_string(&right_text) };
        let pair_addr = unsafe { ctx.alloc_array(2, SOLIX_ARRAY_TAG_REF) };
        unsafe {
            ctx.array_set_u64(pair_addr, 0, u64::from(str_a));
            ctx.array_set_u64(pair_addr, 1, u64::from(str_b));
        }
        SolixCValue::from_ref(pair_addr)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_create_box(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            unsafe { set_vm_error(vm, out_err, "invalid VM context") };
            return SolixCValue::null();
        };
        if argc < 3 || args.is_null() {
            unsafe { set_vm_error(vm, out_err, "create_box expects 3 arguments") };
            return SolixCValue::null();
        }
        let vtable_id = unsafe { ctx.get_class_vtable_id("MatrixBox") };
        if vtable_id == 0 {
            unsafe { set_vm_error(vm, out_err, "MatrixBox class not found") };
            return SolixCValue::null();
        }
        let box_addr = unsafe { ctx.alloc_object(vtable_id, 3) };
        unsafe {
            ctx.set_field_i32(box_addr, 0, (*args.add(0)).as_i32());
            ctx.write_field_u64(box_addr, 1, (*args.add(1)).as_i64() as u64);
            ctx.write_field_u64(box_addr, 2, u64::from((*args.add(2)).as_ref()));
        }
        SolixCValue::from_ref(box_addr)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_mutate_box(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            return SolixCValue::from_i64(0);
        };
        if argc < 2 || args.is_null() {
            return SolixCValue::from_i64(0);
        }
        let box_addr = unsafe { (*args.add(0)).as_ref() };
        let delta = unsafe { (*args.add(1)).as_i32() };
        let next_tag = unsafe { ctx.get_field_i32(box_addr, 0) } + delta;
        unsafe { ctx.set_field_i32(box_addr, 0, next_tag) };
        let weight = unsafe { ctx.read_field_u64(box_addr, 1) } as i64;
        SolixCValue::from_i64(i64::from(next_tag) + weight)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_exercise_gc_and_metrics(
        vm: *mut c_void,
        _args: *const SolixCValue,
        _argc: u32,
        _out_err: *mut SolixCValue,
    ) -> SolixCValue {
        let Some(ctx) = (unsafe { SolixVmContext::validate(vm) }) else {
            return SolixCValue::from_i64(-1);
        };
        unsafe { ctx.set_cycle_threshold(128) };
        let metric_val = unsafe { ctx.get_vm_metric(0) };
        let collected = unsafe { ctx.collect_cycles() };
        SolixCValue::from_i64((metric_val + collected) as i64)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn native_checked_div(
        vm: *mut c_void,
        args: *const SolixCValue,
        argc: u32,
        out_err: *mut SolixCValue,
    ) -> SolixCValue {
        if argc < 2 || args.is_null() {
            unsafe { set_vm_error(vm, out_err, "checked_div requires 2 arguments") };
            return SolixCValue::from_i32(0);
        }
        let numerator = unsafe { (*args.add(0)).as_i32() };
        let denominator = unsafe { (*args.add(1)).as_i32() };
        if denominator == 0 {
            unsafe { set_vm_error(vm, out_err, "division by zero in native backend") };
            return SolixCValue::from_i32(0);
        }
        SolixCValue::from_i32(numerator / denominator)
    }

    solix_export! {
        "FfiMatrix::echo_null" => native_echo_null,
        "FfiMatrix::invert_bool" => native_invert_bool,
        "FfiMatrix::add_i32" => native_add_i32,
        "FfiMatrix::mul_i64" => native_mul_i64,
        "FfiMatrix::fma_f64" => native_fma_f64,
        "FfiMatrix::reverse_string" => native_reverse_string,
        "FfiMatrix::upper_chars" => native_upper_chars,
        "FfiMatrix::xor_bytes" => native_xor_bytes,
        "FfiMatrix::scale_i64_array" => native_scale_i64_array,
        "FfiMatrix::sum_f64_array" => native_sum_f64_array,
        "FfiMatrix::make_pair_strings" => native_make_pair_strings,
        "FfiMatrix::create_box" => native_create_box,
        "FfiMatrix::mutate_box" => native_mutate_box,
        "FfiMatrix::exercise_gc_and_metrics" => native_exercise_gc_and_metrics,
        "FfiMatrix::checked_div" => native_checked_div,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_backend_standalone_math_and_buffers() {
        assert_eq!(matrix_add_i32(15, 27), 42);
        assert_eq!(matrix_mul_i64(100_000, 500_000), 50_000_000_000);
        let fma_val = matrix_fma_f64(2.5, 4.0, 1.5);
        assert!((fma_val - 11.5).abs() < 1e-9);
        assert_eq!(matrix_reverse_ascii("solix"), "xilos");
        assert_eq!(matrix_upper_ascii(b"aBz1"), b"ABZ1");
        assert_eq!(matrix_xor_bytes(&[10, 20, 30], 7), vec![13, 19, 25]);
    }
}
