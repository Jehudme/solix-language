//! Operating system environment and child process native functions.

use crate::{
    alloc_solix_chars, read_solix_chars, register_symbol_variants, NativeCFunctionPtr,
    RegisterNativeCallback, SolixCValue, SolixVmContext, SOLIX_VM_CONTEXT_MAGIC,
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::process::{Child, Command, Stdio};
use std::sync::{LazyLock, Mutex};

struct ProcessHandleEntry {
    child_process: Option<Child>,
    exit_status: Option<i32>,
    standard_output: String,
    standard_error: String,
}

static NEXT_PROCESS_HANDLE: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(1));
static PROCESS_TABLE: LazyLock<Mutex<HashMap<i64, ProcessHandleEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// --- Environment Natives ---

pub unsafe extern "C" fn env_native_get_env(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let key_address = unsafe { (*args.add(0)).as_ref() };
    let key_string = unsafe { read_solix_chars(vm_context, key_address) };
    if let Ok(value_string) = std::env::var(key_string) {
        let allocated_address = unsafe { alloc_solix_chars(vm_context, &value_string) };
        SolixCValue::ref_val(allocated_address)
    } else {
        SolixCValue::null()
    }
}

pub unsafe extern "C" fn env_native_set_env(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count >= 2 && !args.is_null() {
        let key_address = unsafe { (*args.add(0)).as_ref() };
        let value_address = unsafe { (*args.add(1)).as_ref() };
        let key_string = unsafe { read_solix_chars(vm_context, key_address) };
        let value_string = unsafe { read_solix_chars(vm_context, value_address) };
        // SAFETY: set_var is called in single-threaded context for test/script runner
        unsafe {
            std::env::set_var(key_string, value_string);
        }
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn env_native_get_all_env(
    vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let mut entries = Vec::new();
    for (env_key, env_value) in std::env::vars() {
        entries.push(format!("{env_key}={env_value}"));
    }
    let combined_env = entries.join("\n");
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &combined_env) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn env_native_get_args(
    vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if vm_context.is_null() {
        return SolixCValue::null();
    }
    let cli_arguments: Vec<String> = std::env::args().collect();
    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic != SOLIX_VM_CONTEXT_MAGIC {
        return SolixCValue::null();
    }

    let array_length = cli_arguments.len() as u32;
    let array_address = unsafe { (context_ref.alloc_array)(context_ref.vm_ptr, array_length, 13) };
    for (arg_index, arg_text) in cli_arguments.iter().enumerate() {
        let string_address = unsafe { alloc_solix_chars(vm_context, arg_text) };
        unsafe {
            (context_ref.array_set_u64)(
                context_ref.vm_ptr,
                array_address,
                arg_index as u32,
                string_address as u64,
            );
        }
    }

    SolixCValue::ref_val(array_address)
}

pub unsafe extern "C" fn env_native_os_name(
    vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let os_string = std::env::consts::OS;
    let allocated_address = unsafe { alloc_solix_chars(vm_context, os_string) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn env_native_os_version(
    vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let version_info = std::env::consts::ARCH;
    let allocated_address = unsafe { alloc_solix_chars(vm_context, version_info) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn env_native_is_windows(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::bool_val(cfg!(target_os = "windows"))
}

pub unsafe extern "C" fn env_native_is_linux(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::bool_val(cfg!(target_os = "linux"))
}

pub unsafe extern "C" fn env_native_is_macos(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::bool_val(cfg!(target_os = "macos"))
}

pub unsafe extern "C" fn env_native_processor_count(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let core_count = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get() as i32)
        .unwrap_or(1);
    SolixCValue::i32_val(core_count)
}

// --- Process Natives ---

pub unsafe extern "C" fn process_native_start(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i64_val(-1);
    }
    let command_address = unsafe { (*args.add(0)).as_ref() };
    let arguments_address = unsafe { (*args.add(1)).as_ref() };

    let command_string = unsafe { read_solix_chars(vm_context, command_address) };
    let mut command_runner = Command::new(&command_string);

    if arguments_address != 0 && !vm_context.is_null() {
        let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
        if context_ref.magic == SOLIX_VM_CONTEXT_MAGIC {
            let argument_count =
                unsafe { (context_ref.array_len)(context_ref.vm_ptr, arguments_address) };
            for argument_index in 0..argument_count {
                let argument_slot = unsafe {
                    (context_ref.array_get_u64)(
                        context_ref.vm_ptr,
                        arguments_address,
                        argument_index,
                    )
                } as u32;
                let single_argument = unsafe { read_solix_chars(vm_context, argument_slot) };
                command_runner.arg(single_argument);
            }
        }
    }

    command_runner.stdout(Stdio::piped());
    command_runner.stderr(Stdio::piped());

    match command_runner.spawn() {
        Ok(spawned_child) => {
            let mut next_handle_guard = NEXT_PROCESS_HANDLE.lock().unwrap();
            let assigned_handle = *next_handle_guard;
            *next_handle_guard += 1;

            let process_entry = ProcessHandleEntry {
                child_process: Some(spawned_child),
                exit_status: None,
                standard_output: String::new(),
                standard_error: String::new(),
            };

            PROCESS_TABLE
                .lock()
                .unwrap()
                .insert(assigned_handle, process_entry);
            SolixCValue::i64_val(assigned_handle)
        }
        Err(_) => SolixCValue::i64_val(-1),
    }
}

pub unsafe extern "C" fn process_native_wait_for_exit(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i32_val(-1);
    }
    let process_handle = unsafe { (*args.add(0)).as_i64() };
    let mut process_table_guard = PROCESS_TABLE.lock().unwrap();
    if let Some(entry) = process_table_guard.get_mut(&process_handle) {
        if let Some(child) = entry.child_process.take() {
            let output_result = child.wait_with_output();
            if let Ok(process_output) = output_result {
                let exit_code =
                    process_output
                        .status
                        .code()
                        .unwrap_or(if process_output.status.success() {
                            0
                        } else {
                            1
                        });
                entry.exit_status = Some(exit_code);
                entry.standard_output = String::from_utf8_lossy(&process_output.stdout).to_string();
                entry.standard_error = String::from_utf8_lossy(&process_output.stderr).to_string();
                return SolixCValue::i32_val(exit_code);
            }
        } else if let Some(cached_status) = entry.exit_status {
            return SolixCValue::i32_val(cached_status);
        }
    }
    SolixCValue::i32_val(-1)
}

pub unsafe extern "C" fn process_native_get_stdout(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let process_handle = unsafe { (*args.add(0)).as_i64() };
    let process_table_guard = PROCESS_TABLE.lock().unwrap();
    if let Some(entry) = process_table_guard.get(&process_handle) {
        let allocated_address = unsafe { alloc_solix_chars(vm_context, &entry.standard_output) };
        SolixCValue::ref_val(allocated_address)
    } else {
        SolixCValue::null()
    }
}

pub unsafe extern "C" fn process_native_get_stderr(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let process_handle = unsafe { (*args.add(0)).as_i64() };
    let process_table_guard = PROCESS_TABLE.lock().unwrap();
    if let Some(entry) = process_table_guard.get(&process_handle) {
        let allocated_address = unsafe { alloc_solix_chars(vm_context, &entry.standard_error) };
        SolixCValue::ref_val(allocated_address)
    } else {
        SolixCValue::null()
    }
}

pub unsafe extern "C" fn process_native_kill(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let process_handle = unsafe { (*args.add(0)).as_i64() };
    let mut process_table_guard = PROCESS_TABLE.lock().unwrap();
    if let Some(entry) = process_table_guard.get_mut(&process_handle) {
        if let Some(mut child) = entry.child_process.take() {
            let kill_success = child.kill().is_ok();
            let _ = child.wait();
            return SolixCValue::bool_val(kill_success);
        }
    }
    SolixCValue::bool_val(false)
}

pub unsafe fn register_sys_process_natives(
    collector: *mut c_void,
    callback: RegisterNativeCallback,
) {
    let env_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Environment_native_get_env",
                "solix_system_Environment_native_get_env",
                "Environment::native_get_env",
            ],
            env_native_get_env,
        ),
        (
            &[
                "Environment_native_set_env",
                "solix_system_Environment_native_set_env",
                "Environment::native_set_env",
            ],
            env_native_set_env,
        ),
        (
            &[
                "Environment_native_get_all_env",
                "solix_system_Environment_native_get_all_env",
                "Environment::native_get_all_env",
            ],
            env_native_get_all_env,
        ),
        (
            &[
                "Environment_native_get_args",
                "solix_system_Environment_native_get_args",
                "Environment::native_get_args",
            ],
            env_native_get_args,
        ),
        (
            &[
                "Environment_native_os_name",
                "solix_system_Environment_native_os_name",
                "Environment::native_os_name",
            ],
            env_native_os_name,
        ),
        (
            &[
                "Environment_native_os_version",
                "solix_system_Environment_native_os_version",
                "Environment::native_os_version",
            ],
            env_native_os_version,
        ),
        (
            &[
                "Environment_native_is_windows",
                "solix_system_Environment_native_is_windows",
                "Environment::native_is_windows",
            ],
            env_native_is_windows,
        ),
        (
            &[
                "Environment_native_is_linux",
                "solix_system_Environment_native_is_linux",
                "Environment::native_is_linux",
            ],
            env_native_is_linux,
        ),
        (
            &[
                "Environment_native_is_macos",
                "solix_system_Environment_native_is_macos",
                "Environment::native_is_macos",
            ],
            env_native_is_macos,
        ),
        (
            &[
                "Environment_native_processor_count",
                "solix_system_Environment_native_processor_count",
                "Environment::native_processor_count",
            ],
            env_native_processor_count,
        ),
    ];

    for (variants, func) in env_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let process_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Process_native_start",
                "solix_system_Process_native_start",
                "Process::native_start",
            ],
            process_native_start,
        ),
        (
            &[
                "Process_native_wait_for_exit",
                "solix_system_Process_native_wait_for_exit",
                "Process::native_wait_for_exit",
            ],
            process_native_wait_for_exit,
        ),
        (
            &[
                "Process_native_get_stdout",
                "solix_system_Process_native_get_stdout",
                "Process::native_get_stdout",
            ],
            process_native_get_stdout,
        ),
        (
            &[
                "Process_native_get_stderr",
                "solix_system_Process_native_get_stderr",
                "Process::native_get_stderr",
            ],
            process_native_get_stderr,
        ),
        (
            &[
                "Process_native_kill",
                "solix_system_Process_native_kill",
                "Process::native_kill",
            ],
            process_native_kill,
        ),
    ];

    for (variants, func) in process_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
