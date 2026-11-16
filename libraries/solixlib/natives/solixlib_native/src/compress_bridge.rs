//! Compression and decompression native bridge using flate2.

use crate::{
    alloc_solix_bytes, alloc_solix_chars, read_solix_bytes, register_symbol_variants,
    NativeCFunctionPtr, RegisterNativeCallback, SolixCValue,
};
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use flate2::Compression;
use std::ffi::c_void;
use std::io::{Read, Write};

pub unsafe extern "C" fn gzip_native_compress(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let data_address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, data_address) };

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(&input_bytes).is_err() {
        return SolixCValue::null();
    }
    match encoder.finish() {
        Ok(compressed_bytes) => {
            let allocated_address = unsafe { alloc_solix_bytes(vm_context, &compressed_bytes) };
            SolixCValue::ref_val(allocated_address)
        }
        Err(_) => SolixCValue::null(),
    }
}

pub unsafe extern "C" fn gzip_native_decompress(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let data_address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, data_address) };

    let mut decoder = GzDecoder::new(&input_bytes[..]);
    let mut decompressed_bytes = Vec::new();
    match decoder.read_to_end(&mut decompressed_bytes) {
        Ok(_) => {
            let allocated_address = unsafe { alloc_solix_bytes(vm_context, &decompressed_bytes) };
            SolixCValue::ref_val(allocated_address)
        }
        Err(read_error) => {
            if !out_error.is_null() {
                let error_message = format!(
                    "[E0334] Failed to decompress buffer using algorithm 'gzip': {}",
                    read_error
                );
                let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                unsafe {
                    *out_error = SolixCValue::ref_val(allocated_error);
                }
            }
            SolixCValue::null()
        }
    }
}

pub unsafe extern "C" fn zlib_native_compress(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let data_address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, data_address) };

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(&input_bytes).is_err() {
        return SolixCValue::null();
    }
    match encoder.finish() {
        Ok(compressed_bytes) => {
            let allocated_address = unsafe { alloc_solix_bytes(vm_context, &compressed_bytes) };
            SolixCValue::ref_val(allocated_address)
        }
        Err(_) => SolixCValue::null(),
    }
}

pub unsafe extern "C" fn zlib_native_decompress(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let data_address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, data_address) };

    let mut decoder = ZlibDecoder::new(&input_bytes[..]);
    let mut decompressed_bytes = Vec::new();
    match decoder.read_to_end(&mut decompressed_bytes) {
        Ok(_) => {
            let allocated_address = unsafe { alloc_solix_bytes(vm_context, &decompressed_bytes) };
            SolixCValue::ref_val(allocated_address)
        }
        Err(read_error) => {
            if !out_error.is_null() {
                let error_message = format!(
                    "[E0334] Failed to decompress buffer using algorithm 'zlib': {}",
                    read_error
                );
                let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                unsafe {
                    *out_error = SolixCValue::ref_val(allocated_error);
                }
            }
            SolixCValue::null()
        }
    }
}

pub unsafe extern "C" fn deflate_native_compress(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let data_address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, data_address) };

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(&input_bytes).is_err() {
        return SolixCValue::null();
    }
    match encoder.finish() {
        Ok(compressed_bytes) => {
            let allocated_address = unsafe { alloc_solix_bytes(vm_context, &compressed_bytes) };
            SolixCValue::ref_val(allocated_address)
        }
        Err(_) => SolixCValue::null(),
    }
}

pub unsafe extern "C" fn deflate_native_decompress(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let data_address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, data_address) };

    let mut decoder = DeflateDecoder::new(&input_bytes[..]);
    let mut decompressed_bytes = Vec::new();
    match decoder.read_to_end(&mut decompressed_bytes) {
        Ok(_) => {
            let allocated_address = unsafe { alloc_solix_bytes(vm_context, &decompressed_bytes) };
            SolixCValue::ref_val(allocated_address)
        }
        Err(read_error) => {
            if !out_error.is_null() {
                let error_message = format!(
                    "[E0334] Failed to decompress buffer using algorithm 'deflate': {}",
                    read_error
                );
                let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                unsafe {
                    *out_error = SolixCValue::ref_val(allocated_error);
                }
            }
            SolixCValue::null()
        }
    }
}

pub unsafe fn register_compress_natives(
    collector: *mut c_void,
    callback: RegisterNativeCallback,
) {
    let mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Gzip_native_compress",
                "solix_compress_Gzip_native_compress",
                "Gzip::native_compress",
            ],
            gzip_native_compress,
        ),
        (
            &[
                "Gzip_native_decompress",
                "solix_compress_Gzip_native_decompress",
                "Gzip::native_decompress",
            ],
            gzip_native_decompress,
        ),
        (
            &[
                "Zlib_native_compress",
                "solix_compress_Zlib_native_compress",
                "Zlib::native_compress",
            ],
            zlib_native_compress,
        ),
        (
            &[
                "Zlib_native_decompress",
                "solix_compress_Zlib_native_decompress",
                "Zlib::native_decompress",
            ],
            zlib_native_decompress,
        ),
        (
            &[
                "Deflate_native_compress",
                "solix_compress_Deflate_native_compress",
                "Deflate::native_compress",
            ],
            deflate_native_compress,
        ),
        (
            &[
                "Deflate_native_decompress",
                "solix_compress_Deflate_native_decompress",
                "Deflate::native_decompress",
            ],
            deflate_native_decompress,
        ),
    ];

    for (variants, func) in mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
