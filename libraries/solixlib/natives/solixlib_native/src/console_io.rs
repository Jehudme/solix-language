//! Terminal console and stream I/O native functions.

use crate::{
    alloc_solix_chars, read_solix_chars, register_symbol_variants, write_out_box_f64,
    NativeCFunctionPtr, RegisterNativeCallback, SolixCValue, SolixVmContext,
    SOLIX_VM_CONTEXT_MAGIC,
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Read, Seek, SeekFrom, Write};
use std::sync::{LazyLock, Mutex};

static STREAM_TABLE: LazyLock<Mutex<HashMap<i64, File>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_STREAM_HANDLE: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(1));

// --- Console Natives ---

pub unsafe extern "C" fn console_native_print_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let address = unsafe { (*args.add(0)).as_ref() };
        let text = unsafe { read_solix_chars(vm_context, address) };
        print!("{text}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_print_int(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let value = unsafe { (*args.add(0)).as_i64() };
        print!("{value}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_print_double(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let value = unsafe { (*args.add(0)).as_f64() };
        print!("{value}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_print_bool(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let value = unsafe { (*args.add(0)).as_bool() };
        print!("{value}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_print_char(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let character_code = unsafe { (*args.add(0)).as_i32() } as u8;
        print!("{}", character_code as char);
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_println(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    println!();
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_error_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let address = unsafe { (*args.add(0)).as_ref() };
        let text = unsafe { read_solix_chars(vm_context, address) };
        eprintln!("\x1b[31m[ERROR]\x1b[0m {text}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_error_int(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let value = unsafe { (*args.add(0)).as_i64() };
        eprintln!("\x1b[31m[ERROR]\x1b[0m {value}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_warning_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let address = unsafe { (*args.add(0)).as_ref() };
        let text = unsafe { read_solix_chars(vm_context, address) };
        eprintln!("\x1b[33m[WARN]\x1b[0m {text}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_warning_int(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let value = unsafe { (*args.add(0)).as_i64() };
        eprintln!("\x1b[33m[WARN]\x1b[0m {value}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_info_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let address = unsafe { (*args.add(0)).as_ref() };
        let text = unsafe { read_solix_chars(vm_context, address) };
        println!("\x1b[34m[INFO]\x1b[0m {text}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_info_int(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let value = unsafe { (*args.add(0)).as_i64() };
        println!("\x1b[34m[INFO]\x1b[0m {value}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_success_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let address = unsafe { (*args.add(0)).as_ref() };
        let text = unsafe { read_solix_chars(vm_context, address) };
        println!("\x1b[32m[SUCCESS]\x1b[0m {text}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_success_int(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let value = unsafe { (*args.add(0)).as_i64() };
        println!("\x1b[32m[SUCCESS]\x1b[0m {value}");
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_read_line(
    vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let mut input_line = String::new();
    let stdin_handle = std::io::stdin();
    let mut locked_stdin = stdin_handle.lock();
    let _ = locked_stdin.read_line(&mut input_line);
    if input_line.ends_with('\n') {
        input_line.pop();
        if input_line.ends_with('\r') {
            input_line.pop();
        }
    }
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &input_line) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn console_native_read_char(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let mut single_byte_buffer = [0u8; 1];
    let stdin_handle = std::io::stdin();
    let mut locked_stdin = stdin_handle.lock();
    if locked_stdin.read_exact(&mut single_byte_buffer).is_ok() {
        SolixCValue::i32_val(single_byte_buffer[0] as i32)
    } else {
        SolixCValue::i32_val(-1)
    }
}

pub unsafe extern "C" fn console_native_clear(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    print!("\x1b[2J\x1b[H");
    let _ = std::io::stdout().flush();
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_flush(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let _ = std::io::stdout().flush();
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_set_color(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let color_index = unsafe { (*args.add(0)).as_i32() };
        let escape_code = match color_index {
            0 => "\x1b[30m", // Black
            1 => "\x1b[34m", // Blue
            2 => "\x1b[32m", // Green
            3 => "\x1b[36m", // Cyan
            4 => "\x1b[31m", // Red
            5 => "\x1b[35m", // Magenta
            6 => "\x1b[33m", // Yellow
            7 => "\x1b[37m", // White
            8 => "\x1b[90m", // Gray
            _ => "\x1b[0m",
        };
        print!("{escape_code}");
        let _ = std::io::stdout().flush();
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_reset_color(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    print!("\x1b[0m");
    let _ = std::io::stdout().flush();
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_set_cursor_position(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count >= 2 && !args.is_null() {
        let x_column = unsafe { (*args.add(0)).as_i32() } + 1;
        let y_row = unsafe { (*args.add(1)).as_i32() } + 1;
        print!("\x1b[{y_row};{x_column}H");
        let _ = std::io::stdout().flush();
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_set_title(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let address = unsafe { (*args.add(0)).as_ref() };
        let title_text = unsafe { read_solix_chars(vm_context, address) };
        print!("\x1b]0;{title_text}\x07");
        let _ = std::io::stdout().flush();
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn console_native_try_parse_double(
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

// --- FileStream Natives ---

pub unsafe extern "C" fn filestream_native_open(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i64_val(-1);
    }
    let path_address = unsafe { (*args.add(0)).as_ref() };
    let mode_value = unsafe { (*args.add(1)).as_i32() };
    let path_str = unsafe { read_solix_chars(vm_context, path_address) };

    let mut open_options = OpenOptions::new();
    match mode_value {
        0 => {
            // Read
            open_options.read(true);
        }
        1 => {
            // Write (create / truncate)
            open_options.write(true).create(true).truncate(true);
        }
        2 => {
            // Append
            open_options.write(true).create(true).append(true);
        }
        3 => {
            // ReadWrite
            open_options.read(true).write(true).create(true);
        }
        _ => {
            open_options.read(true);
        }
    }

    let file_result = open_options.open(&path_str);
    match file_result {
        Ok(file_handle) => {
            let mut next_handle_guard = NEXT_STREAM_HANDLE.lock().unwrap();
            let assigned_handle = *next_handle_guard;
            *next_handle_guard += 1;
            STREAM_TABLE
                .lock()
                .unwrap()
                .insert(assigned_handle, file_handle);
            SolixCValue::i64_val(assigned_handle)
        }
        Err(_) => SolixCValue::i64_val(-1),
    }
}

pub unsafe extern "C" fn filestream_native_close(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let stream_handle = unsafe { (*args.add(0)).as_i64() };
        let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
        if stream_table_guard.remove(&stream_handle).is_some() {
            return SolixCValue::i64_val(0);
        }
    }
    SolixCValue::i64_val(1)
}

pub unsafe extern "C" fn filestream_native_read_byte(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let stream_handle = unsafe { (*args.add(0)).as_i64() };
        let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
        if let Some(file_handle) = stream_table_guard.get_mut(&stream_handle) {
            let mut byte_buffer = [0u8; 1];
            if file_handle.read_exact(&mut byte_buffer).is_ok() {
                return SolixCValue::i32_val(byte_buffer[0] as i32);
            }
        }
    }
    SolixCValue::i32_val(-1)
}

pub unsafe extern "C" fn filestream_native_write_byte(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count >= 2 && !args.is_null() {
        let stream_handle = unsafe { (*args.add(0)).as_i64() };
        let byte_value = unsafe { (*args.add(1)).as_i32() } as u8;
        let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
        if let Some(file_handle) = stream_table_guard.get_mut(&stream_handle) {
            let _ = file_handle.write_all(&[byte_value]);
        }
    }
    SolixCValue::null()
}

pub unsafe extern "C" fn filestream_native_read(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 4 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let stream_handle = unsafe { (*args.add(0)).as_i64() };
    let buffer_address = unsafe { (*args.add(1)).as_ref() };
    let offset = unsafe { (*args.add(2)).as_i32() } as usize;
    let count = unsafe { (*args.add(3)).as_i32() } as usize;

    let mut temp_bytes = vec![0u8; count];
    let bytes_read_count = {
        let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
        if let Some(file_handle) = stream_table_guard.get_mut(&stream_handle) {
            file_handle.read(&mut temp_bytes).unwrap_or(0)
        } else {
            0
        }
    };

    if bytes_read_count > 0 && !vm_context.is_null() {
        let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
        if context_ref.magic == SOLIX_VM_CONTEXT_MAGIC {
            for (index, &temp_byte) in temp_bytes.iter().enumerate().take(bytes_read_count) {
                unsafe {
                    (context_ref.array_set_u64)(
                        context_ref.vm_ptr,
                        buffer_address,
                        (offset + index) as u32,
                        temp_byte as u64,
                    );
                }
            }
        }
    }

    SolixCValue::i32_val(bytes_read_count as i32)
}

pub unsafe extern "C" fn filestream_native_write(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 4 || args.is_null() {
        return SolixCValue::null();
    }
    let stream_handle = unsafe { (*args.add(0)).as_i64() };
    let buffer_address = unsafe { (*args.add(1)).as_ref() };
    let offset = unsafe { (*args.add(2)).as_i32() } as usize;
    let count = unsafe { (*args.add(3)).as_i32() } as usize;

    let mut temp_bytes = vec![0u8; count];
    if !vm_context.is_null() {
        let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
        if context_ref.magic == SOLIX_VM_CONTEXT_MAGIC {
            for (index, temp_byte) in temp_bytes.iter_mut().enumerate().take(count) {
                let element_word = unsafe {
                    (context_ref.array_get_u64)(
                        context_ref.vm_ptr,
                        buffer_address,
                        (offset + index) as u32,
                    )
                };
                *temp_byte = element_word as u8;
            }
        }
    }

    let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
    if let Some(file_handle) = stream_table_guard.get_mut(&stream_handle) {
        let _ = file_handle.write_all(&temp_bytes);
    }

    SolixCValue::null()
}

pub unsafe extern "C" fn filestream_native_seek(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 3 || args.is_null() {
        return SolixCValue::i64_val(-1);
    }
    let stream_handle = unsafe { (*args.add(0)).as_i64() };
    let offset = unsafe { (*args.add(1)).as_i64() };
    let origin = unsafe { (*args.add(2)).as_i32() };

    let seek_from = match origin {
        0 => SeekFrom::Start(offset.max(0) as u64),
        1 => SeekFrom::Current(offset),
        2 => SeekFrom::End(offset),
        _ => SeekFrom::Start(0),
    };

    let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
    if let Some(file_handle) = stream_table_guard.get_mut(&stream_handle) {
        if let Ok(new_position) = file_handle.seek(seek_from) {
            return SolixCValue::i64_val(new_position as i64);
        }
    }
    SolixCValue::i64_val(-1)
}

pub unsafe extern "C" fn filestream_native_tell(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let stream_handle = unsafe { (*args.add(0)).as_i64() };
        let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
        if let Some(file_handle) = stream_table_guard.get_mut(&stream_handle) {
            if let Ok(current_pos) = file_handle.stream_position() {
                return SolixCValue::i64_val(current_pos as i64);
            }
        }
    }
    SolixCValue::i64_val(-1)
}

pub unsafe extern "C" fn filestream_native_length(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let stream_handle = unsafe { (*args.add(0)).as_i64() };
        let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
        if let Some(file_handle) = stream_table_guard.get_mut(&stream_handle) {
            if let Ok(metadata) = file_handle.metadata() {
                return SolixCValue::i64_val(metadata.len() as i64);
            }
        }
    }
    SolixCValue::i64_val(-1)
}

pub unsafe extern "C" fn filestream_native_flush(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count > 0 && !args.is_null() {
        let stream_handle = unsafe { (*args.add(0)).as_i64() };
        let mut stream_table_guard = STREAM_TABLE.lock().unwrap();
        if let Some(file_handle) = stream_table_guard.get_mut(&stream_handle) {
            let _ = file_handle.flush();
        }
    }
    SolixCValue::null()
}

pub unsafe fn register_console_io_natives(
    collector: *mut c_void,
    callback: RegisterNativeCallback,
) {
    let console_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Console_native_print_string",
                "solix_io_Console_native_print_string",
                "solix_system_Console_native_print_string",
                "Console::native_print_string",
            ],
            console_native_print_string,
        ),
        (
            &[
                "Console_native_print_int",
                "solix_io_Console_native_print_int",
                "solix_system_Console_native_print_int",
                "Console::native_print_int",
            ],
            console_native_print_int,
        ),
        (
            &[
                "Console_native_print_double",
                "solix_io_Console_native_print_double",
                "solix_system_Console_native_print_double",
                "Console::native_print_double",
            ],
            console_native_print_double,
        ),
        (
            &[
                "Console_native_print_bool",
                "solix_io_Console_native_print_bool",
                "solix_system_Console_native_print_bool",
                "Console::native_print_bool",
            ],
            console_native_print_bool,
        ),
        (
            &[
                "Console_native_print_char",
                "solix_io_Console_native_print_char",
                "solix_system_Console_native_print_char",
                "Console::native_print_char",
            ],
            console_native_print_char,
        ),
        (
            &[
                "Console_native_println",
                "solix_io_Console_native_println",
                "solix_system_Console_native_println",
                "Console::native_println",
            ],
            console_native_println,
        ),
        (
            &[
                "Console_native_error_string",
                "solix_io_Console_native_error_string",
                "solix_system_Console_native_error_string",
                "Console::native_error_string",
            ],
            console_native_error_string,
        ),
        (
            &[
                "Console_native_error_int",
                "solix_io_Console_native_error_int",
                "solix_system_Console_native_error_int",
                "Console::native_error_int",
            ],
            console_native_error_int,
        ),
        (
            &[
                "Console_native_warning_string",
                "solix_io_Console_native_warning_string",
                "solix_system_Console_native_warning_string",
                "Console::native_warning_string",
            ],
            console_native_warning_string,
        ),
        (
            &[
                "Console_native_warning_int",
                "solix_io_Console_native_warning_int",
                "solix_system_Console_native_warning_int",
                "Console::native_warning_int",
            ],
            console_native_warning_int,
        ),
        (
            &[
                "Console_native_info_string",
                "solix_io_Console_native_info_string",
                "solix_system_Console_native_info_string",
                "Console::native_info_string",
            ],
            console_native_info_string,
        ),
        (
            &[
                "Console_native_info_int",
                "solix_io_Console_native_info_int",
                "solix_system_Console_native_info_int",
                "Console::native_info_int",
            ],
            console_native_info_int,
        ),
        (
            &[
                "Console_native_success_string",
                "solix_io_Console_native_success_string",
                "solix_system_Console_native_success_string",
                "Console::native_success_string",
            ],
            console_native_success_string,
        ),
        (
            &[
                "Console_native_success_int",
                "solix_io_Console_native_success_int",
                "solix_system_Console_native_success_int",
                "Console::native_success_int",
            ],
            console_native_success_int,
        ),
        (
            &[
                "Console_native_read_line",
                "solix_io_Console_native_read_line",
                "solix_system_Console_native_read_line",
                "Console::native_read_line",
            ],
            console_native_read_line,
        ),
        (
            &[
                "Console_native_read_char",
                "solix_io_Console_native_read_char",
                "solix_system_Console_native_read_char",
                "Console::native_read_char",
            ],
            console_native_read_char,
        ),
        (
            &[
                "Console_native_clear",
                "solix_io_Console_native_clear",
                "solix_system_Console_native_clear",
                "Console::native_clear",
            ],
            console_native_clear,
        ),
        (
            &[
                "Console_native_flush",
                "solix_io_Console_native_flush",
                "solix_system_Console_native_flush",
                "Console::native_flush",
            ],
            console_native_flush,
        ),
        (
            &[
                "Console_native_set_color",
                "solix_io_Console_native_set_color",
                "solix_system_Console_native_set_color",
                "Console::native_set_color",
            ],
            console_native_set_color,
        ),
        (
            &[
                "Console_native_reset_color",
                "solix_io_Console_native_reset_color",
                "solix_system_Console_native_reset_color",
                "Console::native_reset_color",
            ],
            console_native_reset_color,
        ),
        (
            &[
                "Console_native_set_cursor_position",
                "solix_io_Console_native_set_cursor_position",
                "solix_system_Console_native_set_cursor_position",
                "Console::native_set_cursor_position",
            ],
            console_native_set_cursor_position,
        ),
        (
            &[
                "Console_native_set_title",
                "solix_io_Console_native_set_title",
                "solix_system_Console_native_set_title",
                "Console::native_set_title",
            ],
            console_native_set_title,
        ),
        (
            &[
                "Console_native_try_parse_double",
                "solix_io_Console_native_try_parse_double",
                "solix_system_Console_native_try_parse_double",
                "Console::native_try_parse_double",
            ],
            console_native_try_parse_double,
        ),
    ];

    for (variants, func) in console_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let stream_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "FileStream_native_open",
                "solix_io_FileStream_native_open",
                "FileStream::native_open",
            ],
            filestream_native_open,
        ),
        (
            &[
                "FileStream_native_close",
                "solix_io_FileStream_native_close",
                "FileStream::native_close",
            ],
            filestream_native_close,
        ),
        (
            &[
                "FileStream_native_read_byte",
                "solix_io_FileStream_native_read_byte",
                "FileStream::native_read_byte",
            ],
            filestream_native_read_byte,
        ),
        (
            &[
                "FileStream_native_write_byte",
                "solix_io_FileStream_native_write_byte",
                "FileStream::native_write_byte",
            ],
            filestream_native_write_byte,
        ),
        (
            &[
                "FileStream_native_read",
                "solix_io_FileStream_native_read",
                "FileStream::native_read",
            ],
            filestream_native_read,
        ),
        (
            &[
                "FileStream_native_write",
                "solix_io_FileStream_native_write",
                "FileStream::native_write",
            ],
            filestream_native_write,
        ),
        (
            &[
                "FileStream_native_seek",
                "solix_io_FileStream_native_seek",
                "FileStream::native_seek",
            ],
            filestream_native_seek,
        ),
        (
            &[
                "FileStream_native_tell",
                "solix_io_FileStream_native_tell",
                "FileStream::native_tell",
            ],
            filestream_native_tell,
        ),
        (
            &[
                "FileStream_native_length",
                "solix_io_FileStream_native_length",
                "FileStream::native_length",
            ],
            filestream_native_length,
        ),
        (
            &[
                "FileStream_native_flush",
                "solix_io_FileStream_native_flush",
                "FileStream::native_flush",
            ],
            filestream_native_flush,
        ),
    ];

    for (variants, func) in stream_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
