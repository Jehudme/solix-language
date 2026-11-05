//! Unit and integration tests for native FFI shared library loading (Suite 49 conformance).

use solix_common::bytecode::{BinaryProgram, CompiledFunction, Instruction, OpCode};
use solix_compilation::CompilationContext::{BuildProfile, CompilationContext, CompilationOptions};
use solix_compilation::pipeline::CompilationPipeline;
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};
use std::path::PathBuf;
use std::process::Command;

fn custom_host_multiply(
    _vm: &mut VirtualMachine,
    arguments: &[Value],
) -> Result<Value, RuntimeError> {
    let first = arguments[0].as_i32()?;
    let second = arguments[1].as_i32()?;
    Ok(Value::I32(first * second))
}

fn compile_solix_source(source_text: &str) -> BinaryProgram {
    let mut context =
        CompilationContext::new(CompilationOptions::new().with_source("main.slx", source_text));
    CompilationPipeline::for_profile(BuildProfile::Release)
        .execute(&mut context)
        .expect("compilation failed");
    context
        .emission_artifacts
        .expect("emission artifacts missing")
        .program
}

fn host_shared_library_filename(base_name: &str) -> String {
    #[cfg(target_os = "windows")]
    return format!("{base_name}.dll");
    #[cfg(target_os = "macos")]
    return format!("lib{base_name}.dylib");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return format!("lib{base_name}.so");
}

fn compile_rust_cdylib(rust_source_code: &str, base_name: &str) -> PathBuf {
    let library_filename = host_shared_library_filename(base_name);
    let output_directory =
        std::env::temp_dir().join(format!("solix_ffi_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&output_directory);

    let source_file_path = output_directory.join(format!("{base_name}_src.rs"));
    let library_file_path = output_directory.join(&library_filename);

    std::fs::write(&source_file_path, rust_source_code).expect("failed to write rust source");

    let compiler_status = Command::new("rustc")
        .args([
            "--edition",
            "2024",
            "--crate-type",
            "cdylib",
            "-o",
            library_file_path.to_str().expect("valid utf-8 path"),
            source_file_path.to_str().expect("valid utf-8 path"),
        ])
        .status()
        .expect("failed to invoke rustc");

    assert!(
        compiler_status.success(),
        "rustc cdylib compilation failed for {base_name}"
    );
    library_file_path
}

#[test]
fn test_register_and_invoke_custom_native_host_callback() {
    let mut binary = BinaryProgram::new();
    let symbol_index = binary.constant_pool.add_string("host_mult").unwrap();

    let mut main_func = CompiledFunction::new("main", 0, 0, 8);
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 6));
    main_func
        .instructions
        .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 1, 7));
    main_func.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        2,
        symbol_index,
        0,
        2,
    ));
    main_func
        .instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 2, 0, 0));

    binary.functions.push(main_func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    vm.native_registry
        .register("host_mult", custom_host_multiply);

    let result = vm.run().expect("Execution failed");
    assert_eq!(result, Value::I32(42));
}

#[test]
fn test_builtin_assert_success_and_failure() {
    let mut binary = BinaryProgram::new();
    let symbol_assert = binary.constant_pool.add_string("assert").unwrap();

    let mut func = CompiledFunction::new("main", 0, 0, 4);
    func.instructions
        .push(Instruction::format_b(OpCode::LoadBool, 0, 0, 0));
    func.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        1,
        symbol_assert,
        0,
        1,
    ));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run();
    assert!(matches!(result, Err(RuntimeError::AssertionFailed { .. })));
}

#[test]
fn test_builtin_clock() {
    let mut binary = BinaryProgram::new();
    let symbol_clock = binary.constant_pool.add_string("clock").unwrap();

    let mut func = CompiledFunction::new("main", 0, 0, 4);
    func.instructions.push(Instruction::format_call(
        OpCode::CallSymbol,
        0,
        symbol_clock,
        0,
        0,
    ));
    func.instructions
        .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));

    binary.functions.push(func);
    binary.entry_point_id = Some(0);

    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).unwrap();
    let mut vm = VirtualMachine::new(context);

    let result = vm.run().expect("Execution failed");
    let timestamp = result.as_i64().expect("Expected i64 from clock");
    assert!(timestamp > 1_000_000_000_000);
}

// ============================================================================
// Suite 49: Native Interoperability & Dynamic Shared Library Loader
// ============================================================================

#[test]
fn test_case_49_1_static_native_function_call() {
    let solix_code = r#"
public class NativeMath {
    public static native int32 add(int32 a, int32 b);
}

public class Main {
    public static int32 main() {
        return NativeMath.add(15, 27) == 42 ? 0 : 1;
    }
}
"#;
    let rust_bridge_code = r#"
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn NativeMath_add(
    _vm: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_err: *mut SolixCValue,
) -> SolixCValue {
    assert_eq!(arg_count, 2);
    let first = unsafe { (*args.add(0)).payload as i32 };
    let second = unsafe { (*args.add(1)).payload as i32 };
    SolixCValue {
        tag: 2,
        flags: 0,
        payload: ((first + second) as i64) as u64,
    }
}
"#;

    let binary = compile_solix_source(solix_code);
    let library_path = compile_rust_cdylib(rust_bridge_code, "test_native_math_49_1");

    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_native_library(&library_path);
    let context = RuntimeContext::new(options).expect("failed to create context with library");
    let mut vm = VirtualMachine::new(context);

    let exit_value = vm.run().expect("VM execution failed");
    assert_eq!(exit_value, Value::I32(0));
}

#[test]
fn test_case_49_2_instance_native_function_call_with_object_context() {
    let solix_code = r#"
public class Counter {
    public int32 value;
    public native int32 increment();
}

public class Main {
    public static int32 main() {
        Counter c = new Counter();
        c.value = 10;
        int32 res = c.increment();
        return (res == 11 && c.value == 11) ? 0 : 1;
    }
}
"#;
    let rust_bridge_code = r#"
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

#[repr(C)]
pub struct SolixVmContext {
    pub magic: u64,
    pub vm_ptr: *mut std::ffi::c_void,
    pub get_field_i32: unsafe extern "C" fn(*mut std::ffi::c_void, u32, u32) -> i32,
    pub set_field_i32: unsafe extern "C" fn(*mut std::ffi::c_void, u32, u32, i32),
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Counter_increment(
    vm_context: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_err: *mut SolixCValue,
) -> SolixCValue {
    assert_eq!(arg_count, 1);
    let self_address = unsafe { (*args.add(0)).payload as u32 };
    let context_wrapper = unsafe { &*(vm_context as *const SolixVmContext) };
    let current_value =
        unsafe { (context_wrapper.get_field_i32)(context_wrapper.vm_ptr, self_address, 0) };
    let incremented_value = current_value + 1;
    unsafe {
        (context_wrapper.set_field_i32)(context_wrapper.vm_ptr, self_address, 0, incremented_value)
    };
    SolixCValue {
        tag: 2,
        flags: 0,
        payload: (incremented_value as i64) as u64,
    }
}
"#;

    let binary = compile_solix_source(solix_code);
    let library_path = compile_rust_cdylib(rust_bridge_code, "test_counter_49_2");

    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_native_library(&library_path);
    let context = RuntimeContext::new(options).expect("failed to create context with library");
    let mut vm = VirtualMachine::new(context);

    let exit_value = vm.run().expect("VM execution failed");
    assert_eq!(exit_value, Value::I32(0));
}

#[test]
fn test_case_49_3_batch_registration_hook_via_solix_register_natives() {
    let solix_code = r#"
public class BatchPlugin {
    public static native int32 multiply(int32 a, int32 b);
    public static native int32 subtract(int32 a, int32 b);
}

public class Main {
    public static int32 main() {
        int32 m = BatchPlugin.multiply(6, 7);
        int32 s = BatchPlugin.subtract(50, 8);
        return (m == 42 && s == 42) ? 0 : 1;
    }
}
"#;
    let rust_bridge_code = r#"
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

pub type NativeCFunctionPtr = unsafe extern "C" fn(
    *mut std::ffi::c_void,
    *const SolixCValue,
    u32,
    *mut SolixCValue,
) -> SolixCValue;

pub type RegisterNativeCallback = unsafe extern "C" fn(
    *mut std::ffi::c_void,
    *const std::ffi::c_char,
    NativeCFunctionPtr,
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn batch_multiply_hook(
    _vm: *mut std::ffi::c_void,
    args: *const SolixCValue,
    _count: u32,
    _out_err: *mut SolixCValue,
) -> SolixCValue {
    let first = unsafe { (*args.add(0)).payload as i32 };
    let second = unsafe { (*args.add(1)).payload as i32 };
    SolixCValue {
        tag: 2,
        flags: 0,
        payload: ((first * second) as i64) as u64,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn batch_subtract_hook(
    _vm: *mut std::ffi::c_void,
    args: *const SolixCValue,
    _count: u32,
    _out_err: *mut SolixCValue,
) -> SolixCValue {
    let first = unsafe { (*args.add(0)).payload as i32 };
    let second = unsafe { (*args.add(1)).payload as i32 };
    SolixCValue {
        tag: 2,
        flags: 0,
        payload: ((first - second) as i64) as u64,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_register_natives(
    collector_context: *mut std::ffi::c_void,
    register_callback: RegisterNativeCallback,
) {
    let multiply_symbol = c"BatchPlugin::multiply";
    let subtract_symbol = c"BatchPlugin::subtract";
    unsafe {
        register_callback(
            collector_context,
            multiply_symbol.as_ptr(),
            batch_multiply_hook,
        );
        register_callback(
            collector_context,
            subtract_symbol.as_ptr(),
            batch_subtract_hook,
        );
    }
}
"#;

    let binary = compile_solix_source(solix_code);
    let library_path = compile_rust_cdylib(rust_bridge_code, "test_batch_plugin_49_3");

    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_native_library(&library_path);
    let context = RuntimeContext::new(options).expect("failed to create context with library");
    let mut vm = VirtualMachine::new(context);

    let exit_value = vm.run().expect("VM execution failed");
    assert_eq!(exit_value, Value::I32(0));
}

#[test]
fn test_case_49_4_direct_dynamic_symbol_resolution_fallback() {
    let solix_code = r#"
public class DynamicLib {
    public static native int32 direct_export(int32 x);
}

public class Main {
    public static int32 main() {
        return DynamicLib.direct_export(99) == 100 ? 0 : 1;
    }
}
"#;
    let rust_bridge_code = r#"
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn DynamicLib_direct_export(
    _vm: *mut std::ffi::c_void,
    args: *const SolixCValue,
    _count: u32,
    _out_err: *mut SolixCValue,
) -> SolixCValue {
    let input_value = unsafe { (*args.add(0)).payload as i32 };
    SolixCValue {
        tag: 2,
        flags: 0,
        payload: ((input_value + 1) as i64) as u64,
    }
}
"#;

    let binary = compile_solix_source(solix_code);
    let library_path = compile_rust_cdylib(rust_bridge_code, "test_dynamic_lib_49_4");

    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_native_library(&library_path);
    let context = RuntimeContext::new(options).expect("failed to create context with library");
    let mut vm = VirtualMachine::new(context);

    let exit_value = vm.run().expect("VM execution failed");
    assert_eq!(exit_value, Value::I32(0));
}

#[test]
fn test_case_49_5_multiple_shared_libraries_loaded_concurrently() {
    let solix_code = r#"
public class LibA {
    public static native int32 funcA();
}

public class LibB {
    public static native int32 funcB();
}

public class Main {
    public static int32 main() {
        return (LibA.funcA() + LibB.funcB()) == 100 ? 0 : 1;
    }
}
"#;
    let rust_bridge_a = r#"
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn LibA_funcA(
    _vm: *mut std::ffi::c_void,
    _args: *const SolixCValue,
    _count: u32,
    _out_err: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue {
        tag: 2,
        flags: 0,
        payload: 60,
    }
}
"#;
    let rust_bridge_b = r#"
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn LibB_funcB(
    _vm: *mut std::ffi::c_void,
    _args: *const SolixCValue,
    _count: u32,
    _out_err: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue {
        tag: 2,
        flags: 0,
        payload: 40,
    }
}
"#;

    let binary = compile_solix_source(solix_code);
    let library_a_path = compile_rust_cdylib(rust_bridge_a, "test_lib_a_49_5");
    let library_b_path = compile_rust_cdylib(rust_bridge_b, "test_lib_b_49_5");

    let options = RuntimeOptions::new()
        .with_binary(binary)
        .with_native_library(&library_a_path)
        .with_native_library(&library_b_path);
    let context = RuntimeContext::new(options).expect("failed to create context with libraries");
    let mut vm = VirtualMachine::new(context);

    let exit_value = vm.run().expect("VM execution failed");
    assert_eq!(exit_value, Value::I32(0));
}

#[test]
fn test_case_49_6_missing_shared_library_path_negative() {
    let dummy_missing_name = host_shared_library_filename("non_existent_plugin");
    let options = RuntimeOptions::new().with_native_library(&dummy_missing_name);
    let result = RuntimeContext::new(options);

    match result {
        Err(RuntimeError::SharedLibraryError { path, reason }) => {
            assert!(path.contains("non_existent_plugin"));
            assert!(reason.contains("file not found"));
        }
        other => panic!("Expected SharedLibraryError for missing file, got {other:?}"),
    }
}

#[test]
fn test_case_49_7_corrupted_or_non_binary_file_negative() {
    let corrupt_filename = host_shared_library_filename("corrupt_dummy");
    let corrupt_path = std::env::temp_dir()
        .join(format!("solix_corrupt_{}", std::process::id()))
        .join(&corrupt_filename);
    let _ = std::fs::create_dir_all(corrupt_path.parent().unwrap());
    std::fs::write(&corrupt_path, "NOT_AN_ELF_OR_PE_BINARY_TEXT_DATA").unwrap();

    let options = RuntimeOptions::new().with_native_library(&corrupt_path);
    let result = RuntimeContext::new(options);

    assert!(matches!(
        result,
        Err(RuntimeError::SharedLibraryError { .. })
    ));
}

#[test]
fn test_case_49_8_unresolved_native_method_symbol_negative() {
    let solix_code = r#"
public class MissingNative {
    public static native int32 non_existent_function();
}

public class Main {
    public static int32 main() {
        return MissingNative.non_existent_function();
    }
}
"#;
    let binary = compile_solix_source(solix_code);
    let options = RuntimeOptions::new().with_binary(binary);
    let context = RuntimeContext::new(options).expect("context created");
    let mut vm = VirtualMachine::new(context);

    let execution_result = vm.run();
    match execution_result {
        Err(RuntimeError::FunctionNotFound { symbol_or_id }) => {
            assert!(
                symbol_or_id.contains(
                    "Call to unknown native function: MissingNative_non_existent_function"
                )
            );
        }
        other => panic!("Expected FunctionNotFound with native diagnostic, got {other:?}"),
    }
}

#[test]
fn test_case_49_9_reject_foreign_platform_native_library_format_negative() {
    #[cfg(target_os = "windows")]
    let foreign_library_name = "plugin.so";
    #[cfg(not(target_os = "windows"))]
    let foreign_library_name = "plugin.dll";

    let options = RuntimeOptions::new().with_native_library(foreign_library_name);
    let result = RuntimeContext::new(options);

    match result {
        Err(RuntimeError::SharedLibraryError { reason, .. }) => {
            assert!(
                reason.contains("unsupported native library format for host platform"),
                "unexpected reason message: {reason}"
            );
        }
        other => panic!("Expected SharedLibraryError rejecting foreign format, got {other:?}"),
    }
}
