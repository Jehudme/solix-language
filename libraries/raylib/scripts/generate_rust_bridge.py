#!/usr/bin/env python3
"""
generate_rust_bridge.py

Generates 1-to-1 Rust FFI companion bridge (`natives/raylib/src/lib.rs`) from Raylib API JSON (`raylib_api.json`).
Follows Solix language conventions and GL-0011 (no single-letter variables).
"""

import json
import os
import sys
import argparse
from pathlib import Path

RUST_KEYWORDS = {
    "type", "match", "ref", "move", "fn", "loop", "in", "box", "yield",
    "const", "static", "crate", "super", "self", "Self", "let", "mut",
    "pub", "use", "where", "mod", "as", "struct", "enum", "trait", "impl"
}

def escape_rust_ident(ident_name: str) -> str:
    """Escapes Rust keywords using raw identifier syntax r#name."""
    if ident_name in RUST_KEYWORDS:
        return f"r#{ident_name}"
    return ident_name

def to_snake_case(pascal_str: str) -> str:
    """Converts a PascalCase string to snake_case."""
    snake_chars = []
    length = len(pascal_str)
    for index, char in enumerate(pascal_str):
        if char.isupper():
            if index > 0 and (pascal_str[index - 1].islower() or (index + 1 < length and pascal_str[index + 1].islower())):
                snake_chars.append('_')
            snake_chars.append(char.lower())
        else:
            snake_chars.append(char)
    return "".join(snake_chars)

def map_c_type_to_rust(c_type_str: str) -> str:
    """Maps a C type string from raylib_api.json to a Rust FFI type."""
    normalized = c_type_str.strip()
    if normalized == "void":
        return "()"
    if normalized in ("bool", "_Bool"):
        return "bool"
    if normalized in ("int", "i32"):
        return "c_int"
    if normalized in ("unsigned int", "uint", "u32"):
        return "c_uint"
    if normalized in ("float", "f32"):
        return "f32"
    if normalized in ("double", "f64"):
        return "f64"
    if normalized in ("char", "int8"):
        return "c_char"
    if normalized in ("unsigned char", "uint8", "uchar"):
        return "u8"
    if normalized in ("short", "int16"):
        return "c_short"
    if normalized in ("unsigned short", "uint16", "ushort"):
        return "c_ushort"
    if normalized in ("long", "long long", "int64"):
        return "c_longlong"
    if normalized in ("unsigned long", "unsigned long long", "uint64"):
        return "c_ulonglong"
    if normalized in ("const char *", "char *"):
        return "*const c_char"
    if normalized in ("const char **", "char **"):
        return "*mut *mut c_char"
    if normalized in ("void *", "const void *", "void**", "void **"):
        return "*mut c_void"
    if normalized in ("const int *", "int *"):
        return "*mut c_int"
    if normalized in ("const unsigned int *", "unsigned int *"):
        return "*mut c_uint"
    if normalized in ("const float *", "float *"):
        return "*mut f32"
    if normalized in ("const unsigned char *", "unsigned char *"):
        return "*mut u8"
    if normalized in ("const unsigned short *", "unsigned short *"):
        return "*mut c_ushort"
    if normalized in ("Rectangle **",):
        return "*mut *mut C_Rectangle"
    if normalized in ("rAudioBuffer *", "rAudioProcessor *"):
        return "*mut c_void"
    if normalized in ("TraceLogCallback", "LoadFileDataCallback", "SaveFileDataCallback", "LoadFileTextCallback", "SaveFileTextCallback", "AudioCallback"):
        return normalized
    if normalized == "ModelAnimPose":
        return "*mut C_Transform"
    if normalized == "ModelAnimPose *":
        return "*mut *mut C_Transform"
    if normalized == "...":
        return "..."
    
    # Known struct pointers
    if normalized.endswith(" *"):
        base_struct = normalized[:-2].strip()
        is_const = False
        if base_struct.startswith("const "):
            base_struct = base_struct[6:].strip()
            is_const = True
        mut_or_const = "*const" if is_const else "*mut"
        return f"{mut_or_const} C_{base_struct}"
    
    # Value structs
    return f"C_{normalized}"

def generate_header() -> list:
    """Generates the preamble, type definitions, callbacks, and SolixVmContext layout for the Rust bridge."""
    return [
        "//! Auto-generated 1-to-1 Rust native bridge extension for Solix Raylib bindings.",
        "//! Generated from official raylib_api.json specification.",
        "",
        "#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_variables, dead_code, unused_imports, unused_mut, clippy::all)]",
        "",
        "use std::ffi::{c_char, c_int, c_longlong, c_short, c_uint, c_ulonglong, c_ushort, c_void, CStr};",
        "",
        "// ===========================================================================",
        "// Upstream Raylib Callback Function Signatures",
        "// ===========================================================================",
        "pub type TraceLogCallback = Option<unsafe extern \"C\" fn(log_level: c_int, text: *const c_char, args: *mut c_void)>;",
        "pub type LoadFileDataCallback = Option<unsafe extern \"C\" fn(file_name: *const c_char, data_size: *mut c_int) -> *mut u8>;",
        "pub type SaveFileDataCallback = Option<unsafe extern \"C\" fn(file_name: *const c_char, data: *const c_void, data_size: c_int) -> bool>;",
        "pub type LoadFileTextCallback = Option<unsafe extern \"C\" fn(file_name: *const c_char) -> *mut c_char>;",
        "pub type SaveFileTextCallback = Option<unsafe extern \"C\" fn(file_name: *const c_char, text: *const c_char) -> bool>;",
        "pub type AudioCallback = Option<unsafe extern \"C\" fn(buffer_data: *mut c_void, frames: c_uint)>;",
        "",
        "/// Stable C-ABI tagged value passed across native shared library boundaries.",
        "#[repr(C)]",
        "#[derive(Debug, Clone, Copy, PartialEq)]",
        "pub struct SolixCValue {",
        "    pub tag: u32,",
        "    pub flags: u32,",
        "    pub payload: u64,",
        "}",
        "",
        "impl SolixCValue {",
        "    #[inline]",
        "    pub const fn null() -> Self {",
        "        Self { tag: 0, flags: 0, payload: 0 }",
        "    }",
        "    #[inline]",
        "    pub const fn bool_val(val: bool) -> Self {",
        "        Self { tag: 1, flags: 0, payload: if val { 1 } else { 0 } }",
        "    }",
        "    #[inline]",
        "    pub const fn i32_val(val: i32) -> Self {",
        "        Self { tag: 2, flags: 0, payload: (val as i64) as u64 }",
        "    }",
        "    #[inline]",
        "    pub const fn i64_val(val: i64) -> Self {",
        "        Self { tag: 3, flags: 0, payload: val as u64 }",
        "    }",
        "    #[inline]",
        "    pub const fn f64_val(val: f64) -> Self {",
        "        Self { tag: 4, flags: 0, payload: val.to_bits() }",
        "    }",
        "    #[inline]",
        "    pub const fn ref_val(addr: u32) -> Self {",
        "        Self { tag: 5, flags: 0, payload: addr as u64 }",
        "    }",
        "}",
        "",
        "/// Context structure passed to native C callbacks providing safe field and heap accessor function pointers.",
        "#[repr(C)]",
        "pub struct SolixVmContext {",
        "    pub magic: u64,",
        "    pub vm_ptr: *mut c_void,",
        "    pub get_field_i32: unsafe extern \"C\" fn(*mut c_void, u32, u32) -> i32,",
        "    pub set_field_i32: unsafe extern \"C\" fn(*mut c_void, u32, u32, i32),",
        "    pub read_bytes: unsafe extern \"C\" fn(vm_ptr: *mut c_void, addr: u32, out_buf: *mut u8, cap: u32) -> u32,",
        "    pub alloc_string: unsafe extern \"C\" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,",
        "    pub alloc_chars: unsafe extern \"C\" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,",
        "    pub alloc_byte_array: unsafe extern \"C\" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,",
        "    pub alloc_array: unsafe extern \"C\" fn(vm_ptr: *mut c_void, len: u32, type_tag: u8) -> u32,",
        "    pub array_len: unsafe extern \"C\" fn(vm_ptr: *mut c_void, arr_addr: u32) -> u32,",
        "    pub array_get_u64: unsafe extern \"C\" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32) -> u64,",
        "    pub array_set_u64: unsafe extern \"C\" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32, raw_val: u64),",
        "    pub alloc_object: unsafe extern \"C\" fn(vm_ptr: *mut c_void, class_vtable_id: u32, field_count: u32) -> u32,",
        "    pub read_field_u64: unsafe extern \"C\" fn(vm_ptr: *mut c_void, obj_addr: u32, offset: u32) -> u64,",
        "    pub write_field_u64: unsafe extern \"C\" fn(vm_ptr: *mut c_void, obj_addr: u32, offset: u32, raw_val: u64),",
        "}",
        "",
        "pub type NativeCFunctionPtr = unsafe extern \"C\" fn(",
        "    *mut c_void,",
        "    *const SolixCValue,",
        "    u32,",
        "    *mut SolixCValue,",
        ") -> SolixCValue;",
        "",
        "pub type RegisterNativeCallback = unsafe extern \"C\" fn(",
        "    *mut c_void,",
        "    *const c_char,",
        "    NativeCFunctionPtr,",
        ");",
        "",
    ]

def generate_c_structs(structs_list: list) -> list:
    """Generates #[repr(C)] struct definitions matching Raylib C structs."""
    lines = [
        "// ===========================================================================",
        "// Raylib C ABI Struct Definitions",
        "// ===========================================================================",
    ]
    
    # Custom type aliases
    lines.append("pub type C_Texture2D = C_Texture;")
    lines.append("pub type C_TextureCubemap = C_Texture;")
    lines.append("pub type C_RenderTexture2D = C_RenderTexture;")
    lines.append("pub type C_Camera = C_Camera3D;")
    lines.append("pub type C_Quaternion = C_Vector4;")
    lines.append("pub type C_ModelAnimPose = *mut C_Transform;")
    lines.append("")
    
    for struct_item in structs_list:
        struct_name = struct_item["name"]
        lines.append("#[repr(C)]")
        lines.append("#[derive(Clone, Copy)]")
        lines.append(f"pub struct C_{struct_name} {{")
        
        for field in struct_item.get("fields", []):
            field_name = escape_rust_ident(field["name"])
            raw_field_type = field["type"].strip()
            
            if "[" in raw_field_type:
                base_type_str = raw_field_type.split("[")[0].strip()
                size_str = raw_field_type.split("[")[1].split("]")[0].strip()
                rust_base = map_c_type_to_rust(base_type_str)
                lines.append(f"    pub {field_name}: [{rust_base}; {size_str}],")
            else:
                rust_type = map_c_type_to_rust(raw_field_type)
                lines.append(f"    pub {field_name}: {rust_type},")
        lines.append("}")
        lines.append("")
        
        # Default implementation for structs
        lines.append(f"impl Default for C_{struct_name} {{")
        lines.append("    fn default() -> Self {")
        lines.append(f"        unsafe {{ std::mem::zeroed() }}")
        lines.append("    }")
        lines.append("}")
        lines.append("")
        
    return lines

def generate_struct_marshal_helpers(structs_list: list) -> list:
    """Generates unpack, pack, and writeback helpers dynamically for all Raylib structs."""
    lines = [
        "// ===========================================================================",
        "// Dynamic Struct Unpack, Pack & Writeback Helpers",
        "// ===========================================================================",
        "",
    ]

    struct_names = set(item["name"] for item in structs_list)

    for struct_item in structs_list:
        struct_name = struct_item["name"]
        snake_name = to_snake_case(struct_name)
        fields = struct_item.get("fields", [])
        field_count = len(fields)

        # 1. Generate writeback function
        lines.append(f"unsafe fn writeback_{snake_name}(vm_context: *mut c_void, addr: u32, val: &C_{struct_name}) {{")
        lines.append("    if addr == 0 { return; }")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")

        for index, field in enumerate(fields):
            field_ident = escape_rust_ident(field["name"])
            raw_type = field["type"].strip()

            if raw_type in ("bool", "_Bool"):
                lines.append(f"    ((*context_ptr).set_field_i32)(vm_context, addr, {index}, if val.{field_ident} {{ 1 }} else {{ 0 }});")
            elif raw_type in ("unsigned char", "uint8", "uchar", "char", "int8", "short", "int16", "unsigned short", "uint16", "ushort", "int", "i32", "unsigned int", "uint", "u32"):
                lines.append(f"    ((*context_ptr).set_field_i32)(vm_context, addr, {index}, val.{field_ident} as i32);")
            elif raw_type in ("float", "f32"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {index}, val.{field_ident}.to_bits() as u64);")
            elif raw_type in ("double", "f64"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {index}, val.{field_ident}.to_bits());")
            elif raw_type in ("long", "long long", "int64", "unsigned long", "unsigned long long", "uint64"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {index}, val.{field_ident} as u64);")
            elif raw_type.endswith("*") or raw_type.startswith("void"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {index}, val.{field_ident} as usize as u64);")
            elif "[" in raw_type:
                # Fixed array field: arrays are managed separately
                continue
            elif raw_type in struct_names:
                nested_snake = to_snake_case(raw_type)
                lines.append(f"    let child_{index} = pack_{nested_snake}(vm_context, val.{field_ident});")
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {index}, child_{index} as u64);")
            elif raw_type in ("Quaternion", "Texture2D", "TextureCubemap", "RenderTexture2D", "Camera"):
                alias_target = (
                    "vector4" if raw_type == "Quaternion"
                    else ("texture" if raw_type in ("Texture2D", "TextureCubemap")
                    else ("render_texture" if raw_type == "RenderTexture2D"
                    else "camera3d"))
                )
                lines.append(f"    let child_{index} = pack_{alias_target}(vm_context, val.{field_ident});")
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {index}, child_{index} as u64);")

        lines.append("}")
        lines.append("")

        # 2. Generate pack function (allocates and calls writeback)
        lines.append(f"unsafe fn pack_{snake_name}(vm_context: *mut c_void, val: C_{struct_name}) -> u32 {{")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
        lines.append(f"    let addr = ((*context_ptr).alloc_object)(vm_context, 0, {field_count});")
        lines.append("    if addr > 0 {")
        lines.append(f"        writeback_{snake_name}(vm_context, addr, &val);")
        lines.append("    }")
        lines.append("    addr")
        lines.append("}")
        lines.append("")

        # 3. Generate unpack function
        lines.append(f"unsafe fn unpack_{snake_name}(vm_context: *mut c_void, addr: u32) -> C_{struct_name} {{")
        lines.append(f"    if addr == 0 {{ return C_{struct_name}::default(); }}")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
        lines.append(f"    let mut result = C_{struct_name}::default();")

        for index, field in enumerate(fields):
            field_ident = escape_rust_ident(field["name"])
            raw_type = field["type"].strip()

            if raw_type in ("bool", "_Bool"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {index}) != 0;")
            elif raw_type in ("unsigned char", "uint8", "uchar"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {index}) as u8;")
            elif raw_type in ("char", "int8"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {index}) as c_char;")
            elif raw_type in ("short", "int16"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {index}) as c_short;")
            elif raw_type in ("unsigned short", "uint16", "ushort"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {index}) as c_ushort;")
            elif raw_type in ("int", "i32"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {index}) as c_int;")
            elif raw_type in ("unsigned int", "uint", "u32"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {index}) as c_uint;")
            elif raw_type in ("float", "f32"):
                lines.append(f"    result.{field_ident} = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, {index}) as u32);")
            elif raw_type in ("double", "f64"):
                lines.append(f"    result.{field_ident} = f64::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, {index}));")
            elif raw_type in ("long", "long long", "int64"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).read_field_u64)(vm_context, addr, {index}) as c_longlong;")
            elif raw_type in ("unsigned long", "unsigned long long", "uint64"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).read_field_u64)(vm_context, addr, {index}) as c_ulonglong;")
            elif raw_type.endswith("*") or raw_type.startswith("void"):
                rust_cast = map_c_type_to_rust(raw_type)
                lines.append(f"    result.{field_ident} = ((*context_ptr).read_field_u64)(vm_context, addr, {index}) as {rust_cast};")
            elif "[" in raw_type:
                # Array field: keep zeroed default
                pass
            elif raw_type in struct_names:
                nested_snake = to_snake_case(raw_type)
                lines.append(f"    result.{field_ident} = unpack_{nested_snake}(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, {index}) as u32);")
            elif raw_type in ("Quaternion", "Texture2D", "TextureCubemap", "RenderTexture2D", "Camera"):
                alias_target = (
                    "vector4" if raw_type == "Quaternion"
                    else ("texture" if raw_type in ("Texture2D", "TextureCubemap")
                    else ("render_texture" if raw_type == "RenderTexture2D"
                    else "camera3d"))
                )
                lines.append(f"    result.{field_ident} = unpack_{alias_target}(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, {index}) as u32);")

        lines.append("    result")
        lines.append("}")
        lines.append("")

    # Alias unpack/pack forwarders
    lines.append("unsafe fn unpack_quaternion(vm_context: *mut c_void, addr: u32) -> C_Quaternion { unpack_vector4(vm_context, addr) }")
    lines.append("unsafe fn pack_quaternion(vm_context: *mut c_void, val: C_Quaternion) -> u32 { pack_vector4(vm_context, val) }")
    lines.append("unsafe fn writeback_quaternion(vm_context: *mut c_void, addr: u32, val: &C_Quaternion) { writeback_vector4(vm_context, addr, val); }")
    lines.append("unsafe fn unpack_texture2d(vm_context: *mut c_void, addr: u32) -> C_Texture2D { unpack_texture(vm_context, addr) }")
    lines.append("unsafe fn pack_texture2d(vm_context: *mut c_void, val: C_Texture2D) -> u32 { pack_texture(vm_context, val) }")
    lines.append("unsafe fn writeback_texture2d(vm_context: *mut c_void, addr: u32, val: &C_Texture2D) { writeback_texture(vm_context, addr, val); }")
    lines.append("unsafe fn unpack_texture_cubemap(vm_context: *mut c_void, addr: u32) -> C_TextureCubemap { unpack_texture(vm_context, addr) }")
    lines.append("unsafe fn pack_texture_cubemap(vm_context: *mut c_void, val: C_TextureCubemap) -> u32 { pack_texture(vm_context, val) }")
    lines.append("unsafe fn writeback_texture_cubemap(vm_context: *mut c_void, addr: u32, val: &C_TextureCubemap) { writeback_texture(vm_context, addr, val); }")
    lines.append("unsafe fn unpack_render_texture2d(vm_context: *mut c_void, addr: u32) -> C_RenderTexture2D { unpack_render_texture(vm_context, addr) }")
    lines.append("unsafe fn pack_render_texture2d(vm_context: *mut c_void, val: C_RenderTexture2D) -> u32 { pack_render_texture(vm_context, val) }")
    lines.append("unsafe fn writeback_render_texture2d(vm_context: *mut c_void, addr: u32, val: &C_RenderTexture2D) { writeback_render_texture(vm_context, addr, val); }")
    lines.append("unsafe fn unpack_camera(vm_context: *mut c_void, addr: u32) -> C_Camera { unpack_camera3d(vm_context, addr) }")
    lines.append("unsafe fn pack_camera(vm_context: *mut c_void, val: C_Camera) -> u32 { pack_camera3d(vm_context, val) }")
    lines.append("unsafe fn writeback_camera(vm_context: *mut c_void, addr: u32, val: &C_Camera) { writeback_camera3d(vm_context, addr, val); }")
    lines.append("")

    return lines

def generate_extern_c_declarations(functions_list: list) -> list:
    """Generates unsafe extern \"C\" declarations for upstream Raylib functions."""
    lines = [
        "// ===========================================================================",
        "// Upstream Raylib Extern \"C\" Declarations",
        "// ===========================================================================",
        "unsafe extern \"C\" {",
    ]
    for func in functions_list:
        func_name = func["name"]
        raw_ret = func.get("returnType", "void")
        rust_ret = map_c_type_to_rust(raw_ret)
        
        param_decls = []
        for index, param in enumerate(func.get("params", [])):
            param_name = f"arg_{index}"
            raw_param_type = param["type"]
            if raw_param_type == "...":
                param_decls.append("...")
            else:
                rust_type = map_c_type_to_rust(raw_param_type)
                param_decls.append(f"{param_name}: {rust_type}")
            
        ret_suffix = "" if rust_ret == "()" else f" -> {rust_ret}"
        lines.append(f"    pub fn {func_name}({', '.join(param_decls)}){ret_suffix};")
        
    lines.append("}")
    lines.append("")
    return lines

def generate_wrappers(functions_list: list, structs_list: list) -> tuple[list, list]:
    """Generates #[unsafe(no_mangle)] wrapper functions and symbol registration lists."""
    wrapper_lines = [
        "// ===========================================================================",
        "// Solix C-ABI Native Wrapper Functions",
        "// ===========================================================================",
    ]
    registration_entries = []
    struct_names = set(item["name"] for item in structs_list)
    alias_names = {"Quaternion", "Texture2D", "TextureCubemap", "RenderTexture2D", "Camera"}
    callback_names = {"TraceLogCallback", "LoadFileDataCallback", "SaveFileDataCallback", "LoadFileTextCallback", "SaveFileTextCallback", "AudioCallback"}
    
    for func in functions_list:
        func_name = func["name"]
        wrapper_name = f"Raylib_{func_name}"
        raw_ret = func.get("returnType", "void").strip()
        params = func.get("params", [])
        
        wrapper_lines.append("#[unsafe(no_mangle)]")
        wrapper_lines.append(f"pub unsafe extern \"C\" fn {wrapper_name}(")
        wrapper_lines.append("    vm_context: *mut c_void,")
        wrapper_lines.append("    args: *const SolixCValue,")
        wrapper_lines.append("    arg_count: u32,")
        wrapper_lines.append("    out_error: *mut SolixCValue,")
        wrapper_lines.append(") -> SolixCValue {")
        
        call_args = []
        string_buffers = []
        writeback_lines = []
        
        # Handle special variadic functions
        if func_name == "TraceLog":
            string_buffers.append(
                "    let log_level = (*args.add(0)).payload as c_int;\n"
                "    let str_addr = (*args.add(1)).payload as u32;\n"
                "    let mut str_buf = [0u8; 1024];\n"
                "    let str_ptr = if str_addr > 0 {\n"
                "        let context_ptr = vm_context as *mut SolixVmContext;\n"
                "        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr, str_buf.as_mut_ptr(), 1023) as usize;\n"
                "        str_buf[read_len] = 0;\n"
                "        str_buf.as_ptr() as *const c_char\n"
                "    } else {\n"
                '        c"".as_ptr()\n'
                "    };"
            )
            for string_buf in string_buffers:
                wrapper_lines.append(string_buf)
            wrapper_lines.append('    TraceLog(log_level, c"%s".as_ptr(), str_ptr);')
            wrapper_lines.append("    SolixCValue::null()")
            wrapper_lines.append("}")
            wrapper_lines.append("")
            snake_name = to_snake_case(func_name)
            registration_entries.append((f"Raylib::{func_name}", wrapper_name))
            registration_entries.append((f"Raylib::{snake_name}", wrapper_name))
            registration_entries.append((f"raylib::{func_name}", wrapper_name))
            registration_entries.append((f"raylib::{snake_name}", wrapper_name))
            registration_entries.append((func_name, wrapper_name))
            registration_entries.append((snake_name, wrapper_name))
            continue

        if func_name == "TextFormat":
            string_buffers.append(
                "    let str_addr = (*args.add(0)).payload as u32;\n"
                "    let mut str_buf = [0u8; 1024];\n"
                "    let str_ptr = if str_addr > 0 {\n"
                "        let context_ptr = vm_context as *mut SolixVmContext;\n"
                "        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr, str_buf.as_mut_ptr(), 1023) as usize;\n"
                "        str_buf[read_len] = 0;\n"
                "        str_buf.as_ptr() as *const c_char\n"
                "    } else {\n"
                '        c"".as_ptr()\n'
                "    };"
            )
            for string_buf in string_buffers:
                wrapper_lines.append(string_buf)
            wrapper_lines.append('    let call_result = TextFormat(c"%s".as_ptr(), str_ptr);')
            wrapper_lines.append("    if call_result.is_null() { return SolixCValue::null(); }")
            wrapper_lines.append("    let c_str = CStr::from_ptr(call_result);")
            wrapper_lines.append("    let bytes = c_str.to_bytes();")
            wrapper_lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
            wrapper_lines.append("    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);")
            wrapper_lines.append("    SolixCValue::ref_val(addr)")
            wrapper_lines.append("}")
            wrapper_lines.append("")
            snake_name = to_snake_case(func_name)
            registration_entries.append((f"Raylib::{func_name}", wrapper_name))
            registration_entries.append((f"Raylib::{snake_name}", wrapper_name))
            registration_entries.append((f"raylib::{func_name}", wrapper_name))
            registration_entries.append((f"raylib::{snake_name}", wrapper_name))
            registration_entries.append((func_name, wrapper_name))
            registration_entries.append((snake_name, wrapper_name))
            continue

        for index, param in enumerate(params):
            arg_expr = f"(*args.add({index}))"
            raw_param_type = param["type"].strip()
            
            if raw_param_type in ("int", "i32"):
                call_args.append(f"{arg_expr}.payload as c_int")
            elif raw_param_type in ("unsigned int", "uint", "u32"):
                call_args.append(f"{arg_expr}.payload as c_uint")
            elif raw_param_type in ("short", "int16"):
                call_args.append(f"{arg_expr}.payload as c_short")
            elif raw_param_type in ("unsigned short", "uint16", "ushort"):
                call_args.append(f"{arg_expr}.payload as c_ushort")
            elif raw_param_type in ("char", "int8"):
                call_args.append(f"{arg_expr}.payload as c_char")
            elif raw_param_type in ("unsigned char", "uint8", "uchar"):
                call_args.append(f"{arg_expr}.payload as u8")
            elif raw_param_type in ("long", "long long", "int64"):
                call_args.append(f"{arg_expr}.payload as c_longlong")
            elif raw_param_type in ("unsigned long", "unsigned long long", "uint64"):
                call_args.append(f"{arg_expr}.payload as c_ulonglong")
            elif raw_param_type in ("float", "f32"):
                call_args.append(f"f64::from_bits({arg_expr}.payload) as f32")
            elif raw_param_type in ("double", "f64"):
                call_args.append(f"f64::from_bits({arg_expr}.payload)")
            elif raw_param_type in ("bool", "_Bool"):
                call_args.append(f"{arg_expr}.payload != 0")
            elif raw_param_type in ("const char *", "char *"):
                buf_name = f"str_buf_{index}"
                ptr_name = f"str_ptr_{index}"
                addr_name = f"str_addr_{index}"
                string_buffers.append(
                    f"    let {addr_name} = {arg_expr}.payload as u32;\n"
                    f"    let mut {buf_name} = [0u8; 1024];\n"
                    f"    let {ptr_name} = if {addr_name} > 0 {{\n"
                    f"        let context_ptr = vm_context as *mut SolixVmContext;\n"
                    f"        let read_len = ((*context_ptr).read_bytes)(vm_context, {addr_name}, {buf_name}.as_mut_ptr(), 1023) as usize;\n"
                    f"        {buf_name}[read_len] = 0;\n"
                    f"        {buf_name}.as_ptr() as *const c_char\n"
                    f"    }} else {{\n"
                    f'        c"".as_ptr()\n'
                    f"    }};"
                )
                call_args.append(ptr_name)
            elif raw_param_type in callback_names:
                call_args.append(f"if {arg_expr}.payload == 0 {{ None }} else {{ Some(std::mem::transmute({arg_expr}.payload as usize)) }}")
            elif raw_param_type in struct_names or raw_param_type in alias_names:
                struct_snake = to_snake_case(raw_param_type)
                call_args.append(f"unpack_{struct_snake}(vm_context, {arg_expr}.payload as u32)")
            # In-place mutable struct pointers (e.g. Camera *, Image *, Mesh *, Model *, Wave *)
            elif raw_param_type.endswith(" *") and raw_param_type[:-2].strip() in struct_names:
                base_struct = raw_param_type[:-2].strip()
                struct_snake = to_snake_case(base_struct)
                var_name = f"{struct_snake}_mut_{index}"
                addr_name = f"{struct_snake}_addr_{index}"
                string_buffers.append(
                    f"    let {addr_name} = {arg_expr}.payload as u32;\n"
                    f"    let mut {var_name} = unpack_{struct_snake}(vm_context, {addr_name});"
                )
                call_args.append(f"&mut {var_name}")
                writeback_lines.append(f"    writeback_{struct_snake}(vm_context, {addr_name}, &{var_name});")
            else:
                # Raw integer or pointer cast
                rust_cast = map_c_type_to_rust(raw_param_type)
                call_args.append(f"{arg_expr}.payload as {rust_cast}")
        
        for string_buf in string_buffers:
            wrapper_lines.append(string_buf)
            
        call_expr = f"{func_name}({', '.join(call_args)})"
        
        # Handle return values
        if raw_ret == "void":
            wrapper_lines.append(f"    {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append("    SolixCValue::null()")
        elif raw_ret in ("bool", "_Bool"):
            wrapper_lines.append(f"    let call_result = {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append("    SolixCValue::bool_val(call_result)")
        elif raw_ret in ("int", "i32", "unsigned int", "uint", "u32", "short", "int16", "unsigned short", "uint16", "ushort", "char", "int8", "unsigned char", "uint8", "uchar"):
            wrapper_lines.append(f"    let call_result = {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append("    SolixCValue::i32_val(call_result as i32)")
        elif raw_ret in ("long", "long long", "int64", "unsigned long", "unsigned long long", "uint64"):
            wrapper_lines.append(f"    let call_result = {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append("    SolixCValue::i64_val(call_result as i64)")
        elif raw_ret in ("float", "f32"):
            wrapper_lines.append(f"    let call_result = {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append("    SolixCValue::f64_val(call_result as f64)")
        elif raw_ret in ("double", "f64"):
            wrapper_lines.append(f"    let call_result = {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append("    SolixCValue::f64_val(call_result)")
        elif raw_ret in ("const char *", "char *"):
            wrapper_lines.append(f"    let call_result = {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append("    if call_result.is_null() { return SolixCValue::null(); }")
            wrapper_lines.append("    let c_str = CStr::from_ptr(call_result);")
            wrapper_lines.append("    let bytes = c_str.to_bytes();")
            wrapper_lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
            wrapper_lines.append("    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);")
            wrapper_lines.append("    SolixCValue::ref_val(addr)")
        elif raw_ret in struct_names or raw_ret in alias_names:
            struct_snake = to_snake_case(raw_ret)
            wrapper_lines.append(f"    let call_result = {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append(f"    SolixCValue::ref_val(pack_{struct_snake}(vm_context, call_result))")
        else:
            # Fallback raw pointer address
            wrapper_lines.append(f"    let call_result = {call_expr};")
            for writeback_line in writeback_lines:
                wrapper_lines.append(writeback_line)
            wrapper_lines.append("    SolixCValue::i64_val(call_result as usize as i64)")
            
        wrapper_lines.append("}")
        wrapper_lines.append("")
        
        # Registration entries for all naming styles
        snake_name = to_snake_case(func_name)
        registration_entries.append((f"Raylib::{func_name}", wrapper_name))
        registration_entries.append((f"Raylib::{snake_name}", wrapper_name))
        registration_entries.append((f"raylib::{func_name}", wrapper_name))
        registration_entries.append((f"raylib::{snake_name}", wrapper_name))
        registration_entries.append((func_name, wrapper_name))
        registration_entries.append((snake_name, wrapper_name))
        
    return wrapper_lines, registration_entries

def generate_registration(registration_entries: list) -> list:
    """Generates the solix_register_natives entrypoint function."""
    lines = [
        "// ===========================================================================",
        "// Native Symbol Dynamic Registration Hook",
        "// ===========================================================================",
        "#[unsafe(no_mangle)]",
        "pub unsafe extern \"C\" fn solix_register_natives(",
        "    collector_context: *mut c_void,",
        "    register_callback: RegisterNativeCallback,",
        ") {",
    ]
    seen_symbols = set()
    for symbol_path, wrapper_name in registration_entries:
        if symbol_path in seen_symbols:
            continue
        seen_symbols.add(symbol_path)
        lines.append(f'    register_callback(collector_context, c"{symbol_path}".as_ptr(), {wrapper_name});')
        
    lines.append("}")
    lines.append("")
    return lines

def main():
    parser = argparse.ArgumentParser(description="Generate Rust FFI bridge from raylib_api.json")
    parser.add_argument("--api", default="libraries/raylib/api/raylib_api.json", help="Path to raylib_api.json")
    parser.add_argument("--output", default="libraries/raylib/natives/raylib/src/lib.rs", help="Path to output lib.rs")
    args = parser.parse_args()
    
    api_path = Path(args.api)
    output_path = Path(args.output)
    
    if not api_path.exists():
        print(f"Error: API file not found at {api_path}", file=sys.stderr)
        sys.exit(1)
        
    with open(api_path, "r", encoding="utf-8") as file_handle:
        api_data = json.load(file_handle)
        
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    structs_list = api_data.get("structs", [])
    functions_list = api_data.get("functions", [])

    output_lines = []
    output_lines.extend(generate_header())
    output_lines.extend(generate_c_structs(structs_list))
    output_lines.extend(generate_struct_marshal_helpers(structs_list))
    output_lines.extend(generate_extern_c_declarations(functions_list))
    
    wrapper_lines, registration_entries = generate_wrappers(functions_list, structs_list)
    output_lines.extend(wrapper_lines)
    output_lines.extend(generate_registration(registration_entries))
    
    with open(output_path, "w", encoding="utf-8") as out_file:
        out_file.write("\n".join(output_lines) + "\n")
        
    print(f"Successfully generated {output_path} ({len(output_lines)} lines)")

if __name__ == "__main__":
    main()
