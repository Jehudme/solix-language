//! File system and path manipulation native functions.

use crate::{
    alloc_solix_chars, read_solix_chars, register_symbol_variants, NativeCFunctionPtr,
    RegisterNativeCallback, SolixCValue, SolixVmContext, SOLIX_VM_CONTEXT_MAGIC,
};
use std::ffi::c_void;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

fn get_resource_root() -> PathBuf {
    if let Ok(custom_root) = std::env::var("SOLIX_RESOURCE_ROOT") {
        if !custom_root.is_empty() {
            return PathBuf::from(custom_root);
        }
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("resources")
}

fn resolve_target_path(path_str: &str) -> PathBuf {
    if let Some(stripped_prefix) = path_str.strip_prefix("res://") {
        let clean_relative = stripped_prefix.trim_start_matches(['/', '\\']);
        get_resource_root().join(clean_relative)
    } else {
        PathBuf::from(path_str)
    }
}

// --- Path Natives ---

pub unsafe extern "C" fn path_native_combine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::null();
    }
    let first_address = unsafe { (*args.add(0)).as_ref() };
    let second_address = unsafe { (*args.add(1)).as_ref() };
    let first_path = unsafe { read_solix_chars(vm_context, first_address) };
    let second_path = unsafe { read_solix_chars(vm_context, second_address) };

    let combined = Path::new(&first_path).join(second_path);
    let normalized_string = combined.to_string_lossy().replace('\\', "/");
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &normalized_string) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn path_native_get_directory_name(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let path_str = unsafe { read_solix_chars(vm_context, address) };
    let parent_directory = Path::new(&path_str)
        .parent()
        .map(|parent_path| parent_path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &parent_directory) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn path_native_get_file_name(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let path_str = unsafe { read_solix_chars(vm_context, address) };
    let file_name = Path::new(&path_str)
        .file_name()
        .map(|name_str| name_str.to_string_lossy().to_string())
        .unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &file_name) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn path_native_get_extension(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let path_str = unsafe { read_solix_chars(vm_context, address) };
    let extension = Path::new(&path_str)
        .extension()
        .map(|ext_str| format!(".{}", ext_str.to_string_lossy()))
        .unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &extension) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn path_native_get_file_name_without_extension(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let path_str = unsafe { read_solix_chars(vm_context, address) };
    let stem = Path::new(&path_str)
        .file_stem()
        .map(|stem_str| stem_str.to_string_lossy().to_string())
        .unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &stem) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn path_native_is_absolute(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let path_str = unsafe { read_solix_chars(vm_context, address) };
    SolixCValue::bool_val(Path::new(&path_str).is_absolute())
}

pub unsafe extern "C" fn path_native_get_temp_path(
    vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let temporary_directory = std::env::temp_dir().to_string_lossy().replace('\\', "/");
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &temporary_directory) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn path_native_normalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let path_str = unsafe { read_solix_chars(vm_context, address) };
    let input_path = Path::new(&path_str);
    let mut normalized_components = Vec::new();
    for component in input_path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized_components.pop();
            }
            normal_component => {
                normalized_components.push(normal_component.as_os_str().to_string_lossy());
            }
        }
    }
    let joined = normalized_components.join("/").replace('\\', "/");
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &joined) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn path_native_get_separator(
    _vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    SolixCValue::i32_val(std::path::MAIN_SEPARATOR as i32)
}

// --- File Natives ---

pub unsafe extern "C" fn file_native_exists(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);
    SolixCValue::bool_val(target.is_file())
}

pub unsafe extern "C" fn file_native_read_all_text(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);

    let content = fs::read_to_string(&target).unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &content) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn file_native_write_all_text(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let path_address = unsafe { (*args.add(0)).as_ref() };
    let content_address = unsafe { (*args.add(1)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, path_address) };
    let content = unsafe { read_solix_chars(vm_context, content_address) };
    let target = resolve_target_path(&raw_path);

    let write_success = fs::write(&target, content).is_ok();
    SolixCValue::bool_val(write_success)
}

pub unsafe extern "C" fn file_native_append_all_text(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let path_address = unsafe { (*args.add(0)).as_ref() };
    let content_address = unsafe { (*args.add(1)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, path_address) };
    let content = unsafe { read_solix_chars(vm_context, content_address) };
    let target = resolve_target_path(&raw_path);

    use std::io::Write;
    let append_success = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&target)
        .and_then(|mut file| file.write_all(content.as_bytes()))
        .is_ok();
    SolixCValue::bool_val(append_success)
}

pub unsafe extern "C" fn file_native_delete(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);
    SolixCValue::bool_val(fs::remove_file(target).is_ok())
}

pub unsafe extern "C" fn file_native_copy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let source_address = unsafe { (*args.add(0)).as_ref() };
    let destination_address = unsafe { (*args.add(1)).as_ref() };
    let source_path = unsafe { read_solix_chars(vm_context, source_address) };
    let destination_path = unsafe { read_solix_chars(vm_context, destination_address) };
    let copy_success = fs::copy(
        resolve_target_path(&source_path),
        resolve_target_path(&destination_path),
    )
    .is_ok();
    SolixCValue::bool_val(copy_success)
}

pub unsafe extern "C" fn file_native_move(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let source_address = unsafe { (*args.add(0)).as_ref() };
    let destination_address = unsafe { (*args.add(1)).as_ref() };
    let source_path = unsafe { read_solix_chars(vm_context, source_address) };
    let destination_path = unsafe { read_solix_chars(vm_context, destination_address) };
    let rename_success = fs::rename(
        resolve_target_path(&source_path),
        resolve_target_path(&destination_path),
    )
    .is_ok();
    SolixCValue::bool_val(rename_success)
}

pub unsafe extern "C" fn file_native_get_size(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(-1);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);
    let file_size = fs::metadata(target)
        .map(|meta| meta.len() as i64)
        .unwrap_or(-1);
    SolixCValue::i64_val(file_size)
}

pub unsafe extern "C" fn file_native_get_last_modified(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);
    let timestamp_millis = fs::metadata(target)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|system_time| system_time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0);
    SolixCValue::i64_val(timestamp_millis)
}

// --- Directory Natives ---

pub unsafe extern "C" fn directory_native_exists(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);
    SolixCValue::bool_val(target.is_dir())
}

pub unsafe extern "C" fn directory_native_create(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);
    SolixCValue::bool_val(fs::create_dir_all(target).is_ok())
}

pub unsafe extern "C" fn directory_native_delete(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);
    SolixCValue::bool_val(fs::remove_dir_all(target).is_ok())
}

pub unsafe extern "C" fn directory_native_list_entries(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() || vm_context.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let raw_path = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&raw_path);

    let entries: Vec<String> = if let Ok(read_dir) = fs::read_dir(target) {
        read_dir
            .filter_map(|entry_result| entry_result.ok())
            .map(|entry| entry.path().to_string_lossy().replace('\\', "/"))
            .collect()
    } else {
        Vec::new()
    };

    let context_ref = unsafe { &*(vm_context as *const SolixVmContext) };
    if context_ref.magic != SOLIX_VM_CONTEXT_MAGIC {
        return SolixCValue::null();
    }

    let array_length = entries.len() as u32;
    // Allocate ArrayRef with type tag 13 (String)
    let array_address = unsafe { (context_ref.alloc_array)(context_ref.vm_ptr, array_length, 13) };
    for (entry_index, entry_path) in entries.iter().enumerate() {
        let string_address = unsafe { alloc_solix_chars(vm_context, entry_path) };
        unsafe {
            (context_ref.array_set_u64)(
                context_ref.vm_ptr,
                array_address,
                entry_index as u32,
                string_address as u64,
            );
        }
    }

    SolixCValue::ref_val(array_address)
}

pub unsafe extern "C" fn directory_native_get_current(
    vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let current_dir = std::env::current_dir()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| ".".to_string());
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &current_dir) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn directory_native_set_current(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let path_str = unsafe { read_solix_chars(vm_context, address) };
    SolixCValue::bool_val(std::env::set_current_dir(path_str).is_ok())
}

// --- Resource Natives ---

pub unsafe extern "C" fn resource_native_get_resource_root(
    vm_context: *mut c_void,
    _args: *const SolixCValue,
    _arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    let root_str = get_resource_root().to_string_lossy().replace('\\', "/");
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &root_str) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn resource_native_get_path(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let uri_string = unsafe { read_solix_chars(vm_context, address) };
    let resolved = resolve_target_path(&uri_string)
        .to_string_lossy()
        .replace('\\', "/");
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &resolved) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn resource_native_exists(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let uri_string = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&uri_string);
    SolixCValue::bool_val(target.is_file())
}

pub unsafe extern "C" fn resource_native_read_all_text(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let uri_string = unsafe { read_solix_chars(vm_context, address) };
    let target = resolve_target_path(&uri_string);
    let content = fs::read_to_string(target).unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &content) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe fn register_fs_path_natives(collector: *mut c_void, callback: RegisterNativeCallback) {
    let path_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Path_native_combine",
                "solix_io_filesystem_Path_native_combine",
                "solix_io_Path_native_combine",
                "Path::native_combine",
            ],
            path_native_combine,
        ),
        (
            &[
                "Path_native_get_directory_name",
                "solix_io_filesystem_Path_native_get_directory_name",
                "solix_io_Path_native_get_directory_name",
                "Path::native_get_directory_name",
            ],
            path_native_get_directory_name,
        ),
        (
            &[
                "Path_native_get_file_name",
                "solix_io_filesystem_Path_native_get_file_name",
                "solix_io_Path_native_get_file_name",
                "Path::native_get_file_name",
            ],
            path_native_get_file_name,
        ),
        (
            &[
                "Path_native_get_extension",
                "solix_io_filesystem_Path_native_get_extension",
                "solix_io_Path_native_get_extension",
                "Path::native_get_extension",
            ],
            path_native_get_extension,
        ),
        (
            &[
                "Path_native_get_file_name_without_extension",
                "solix_io_filesystem_Path_native_get_file_name_without_extension",
                "solix_io_Path_native_get_file_name_without_extension",
                "Path::native_get_file_name_without_extension",
            ],
            path_native_get_file_name_without_extension,
        ),
        (
            &[
                "Path_native_is_absolute",
                "solix_io_filesystem_Path_native_is_absolute",
                "solix_io_Path_native_is_absolute",
                "Path::native_is_absolute",
            ],
            path_native_is_absolute,
        ),
        (
            &[
                "Path_native_get_temp_path",
                "solix_io_filesystem_Path_native_get_temp_path",
                "solix_io_Path_native_get_temp_path",
                "Path::native_get_temp_path",
            ],
            path_native_get_temp_path,
        ),
        (
            &[
                "Path_native_normalize",
                "solix_io_filesystem_Path_native_normalize",
                "solix_io_Path_native_normalize",
                "Path::native_normalize",
            ],
            path_native_normalize,
        ),
        (
            &[
                "Path_native_get_separator",
                "solix_io_filesystem_Path_native_get_separator",
                "solix_io_Path_native_get_separator",
                "Path::native_get_separator",
            ],
            path_native_get_separator,
        ),
    ];

    for (variants, func) in path_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let file_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "File_native_exists",
                "solix_io_filesystem_File_native_exists",
                "solix_io_File_native_exists",
                "File::native_exists",
            ],
            file_native_exists,
        ),
        (
            &[
                "File_native_read_all_text",
                "solix_io_filesystem_File_native_read_all_text",
                "solix_io_File_native_read_all_text",
                "File::native_read_all_text",
            ],
            file_native_read_all_text,
        ),
        (
            &[
                "File_native_write_all_text",
                "solix_io_filesystem_File_native_write_all_text",
                "solix_io_File_native_write_all_text",
                "File::native_write_all_text",
            ],
            file_native_write_all_text,
        ),
        (
            &[
                "File_native_append_all_text",
                "solix_io_filesystem_File_native_append_all_text",
                "solix_io_File_native_append_all_text",
                "File::native_append_all_text",
            ],
            file_native_append_all_text,
        ),
        (
            &[
                "File_native_delete",
                "solix_io_filesystem_File_native_delete",
                "solix_io_File_native_delete",
                "File::native_delete",
            ],
            file_native_delete,
        ),
        (
            &[
                "File_native_copy",
                "solix_io_filesystem_File_native_copy",
                "solix_io_File_native_copy",
                "File::native_copy",
            ],
            file_native_copy,
        ),
        (
            &[
                "File_native_move",
                "solix_io_filesystem_File_native_move",
                "solix_io_File_native_move",
                "File::native_move",
            ],
            file_native_move,
        ),
        (
            &[
                "File_native_get_size",
                "solix_io_filesystem_File_native_get_size",
                "solix_io_File_native_get_size",
                "File::native_get_size",
            ],
            file_native_get_size,
        ),
        (
            &[
                "File_native_get_last_modified",
                "solix_io_filesystem_File_native_get_last_modified",
                "solix_io_File_native_get_last_modified",
                "File::native_get_last_modified",
            ],
            file_native_get_last_modified,
        ),
    ];

    for (variants, func) in file_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let directory_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Directory_native_exists",
                "solix_io_filesystem_Directory_native_exists",
                "solix_io_Directory_native_exists",
                "Directory::native_exists",
            ],
            directory_native_exists,
        ),
        (
            &[
                "Directory_native_create",
                "solix_io_filesystem_Directory_native_create",
                "solix_io_Directory_native_create",
                "Directory::native_create",
            ],
            directory_native_create,
        ),
        (
            &[
                "Directory_native_delete",
                "solix_io_filesystem_Directory_native_delete",
                "solix_io_Directory_native_delete",
                "Directory::native_delete",
            ],
            directory_native_delete,
        ),
        (
            &[
                "Directory_native_list_entries",
                "solix_io_filesystem_Directory_native_list_entries",
                "solix_io_Directory_native_list_entries",
                "Directory::native_list_entries",
            ],
            directory_native_list_entries,
        ),
        (
            &[
                "Directory_native_get_current",
                "solix_io_filesystem_Directory_native_get_current",
                "solix_io_Directory_native_get_current",
                "Directory::native_get_current",
            ],
            directory_native_get_current,
        ),
        (
            &[
                "Directory_native_set_current",
                "solix_io_filesystem_Directory_native_set_current",
                "solix_io_Directory_native_set_current",
                "Directory::native_set_current",
            ],
            directory_native_set_current,
        ),
    ];

    for (variants, func) in directory_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let resource_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Resource_native_get_resource_root",
                "solix_io_filesystem_Resource_native_get_resource_root",
                "solix_io_Resource_native_get_resource_root",
                "Resource::native_get_resource_root",
            ],
            resource_native_get_resource_root,
        ),
        (
            &[
                "Resource_native_get_path",
                "solix_io_filesystem_Resource_native_get_path",
                "solix_io_Resource_native_get_path",
                "Resource::native_get_path",
            ],
            resource_native_get_path,
        ),
        (
            &[
                "Resource_native_exists",
                "solix_io_filesystem_Resource_native_exists",
                "solix_io_Resource_native_exists",
                "Resource::native_exists",
            ],
            resource_native_exists,
        ),
        (
            &[
                "Resource_native_read_all_text",
                "solix_io_filesystem_Resource_native_read_all_text",
                "solix_io_Resource_native_read_all_text",
                "Resource::native_read_all_text",
            ],
            resource_native_read_all_text,
        ),
    ];

    for (variants, func) in resource_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
