#!/usr/bin/env python3
"""
generate_rust_bridge.py

Generates 1-to-1 Rust FFI companion bridge (`natives/<module>/src/lib.rs`) from rlparser C API JSON.
100% generic: zero hardcoded struct, function, or callback names.
Follows Solix language conventions and GL-0011 (no single-letter variables).
"""

from __future__ import annotations

import argparse
import json
import os
import sys
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
    str_len = len(pascal_str)
    for index, char_elem in enumerate(pascal_str):
        if char_elem.isupper():
            if index > 0 and (
                pascal_str[index - 1].islower()
                or (index + 1 < str_len and pascal_str[index + 1].islower())
            ):
                snake_chars.append("_")
            snake_chars.append(char_elem.lower())
        else:
            snake_chars.append(char_elem)
    return "".join(snake_chars)


def resolve_alias_target(type_str: str, alias_map: dict[str, str]) -> str:
    """Recursively resolves an alias to its concrete target type."""
    visited_aliases = set()
    current_target = type_str.strip()
    while current_target in alias_map and current_target not in visited_aliases:
        visited_aliases.add(current_target)
        current_target = alias_map[current_target].strip()
    return current_target


def decompose_c_type(raw_type_str: str) -> tuple[str, bool, int]:
    """Decomposes a C type into (base_type_name, is_const, pointer_depth)."""
    cleaned = raw_type_str.strip()
    is_const = False
    if cleaned.startswith("const "):
        is_const = True
        cleaned = cleaned[6:].strip()
    elif " const" in cleaned:
        is_const = True
        cleaned = cleaned.replace(" const", "").strip()

    pointer_depth = 0
    while cleaned.endswith("*"):
        pointer_depth += 1
        cleaned = cleaned[:-1].strip()

    return cleaned, is_const, pointer_depth


def map_c_type_to_rust(
    raw_type_str: str,
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    callback_map: dict[str, dict],
) -> str:
    """Maps a C type string from the schema to its Rust FFI counterpart."""
    normalized = raw_type_str.strip()

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
    if normalized in ("void *", "const void *", "void**", "void **", "va_list"):
        return "*mut c_void"

    base_name, is_const, pointer_depth = decompose_c_type(normalized)
    resolved_base = resolve_alias_target(base_name, alias_map)

    # Callbacks
    if base_name in callback_map:
        return base_name
    if resolved_base in callback_map:
        return resolved_base

    # Struct types
    target_struct = None
    if base_name in struct_map:
        target_struct = base_name
    elif resolved_base in struct_map:
        target_struct = resolved_base

    if target_struct is not None:
        if pointer_depth == 0:
            return f"C_{target_struct}"
        if pointer_depth == 1:
            mut_or_const = "*const" if is_const else "*mut"
            return f"{mut_or_const} C_{target_struct}"
        return "*mut c_void"

    # Primitive pointers
    primitive_rust = {
        "int": "c_int", "i32": "c_int",
        "unsigned int": "c_uint", "uint": "c_uint", "u32": "c_uint",
        "float": "f32", "f32": "f32",
        "double": "f64", "f64": "f64",
        "char": "c_char", "int8": "c_char",
        "unsigned char": "u8", "uint8": "u8", "uchar": "u8",
        "short": "c_short", "int16": "c_short",
        "unsigned short": "c_ushort", "uint16": "c_ushort", "ushort": "c_ushort",
        "long": "c_longlong", "long long": "c_longlong", "int64": "c_longlong",
        "unsigned long": "c_ulonglong", "unsigned long long": "c_ulonglong", "uint64": "c_ulonglong",
    }
    if pointer_depth == 1 and base_name in primitive_rust:
        mut_or_const = "*const" if is_const else "*mut"
        return f"{mut_or_const} {primitive_rust[base_name]}"

    if pointer_depth > 0:
        return "*mut c_void"

    return f"C_{base_name}"


def generate_header() -> list[str]:
    """Generates the preamble, SolixCValue, and SolixVmContext layout for the Rust bridge."""
    return [
        "//! Auto-generated 1-to-1 Rust native bridge extension for Solix C-API bindings.",
        "//! Schema-driven codegen: zero hardcoded engine types.",
        "",
        "#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_variables, dead_code, unused_imports, unused_mut, clippy::all)]",
        "",
        "use std::ffi::{c_char, c_int, c_longlong, c_short, c_uint, c_ulonglong, c_ushort, c_void, CStr};",
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
        "/// Context structure passed to native callbacks providing safe field and heap accessor function pointers.",
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
        "    pub get_class_vtable_id: unsafe extern \"C\" fn(vm_ptr: *mut c_void, name_ptr: *const c_char) -> u32,",
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


def generate_callbacks(
    callbacks_list: list[dict],
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    callback_map: dict[str, dict],
) -> list[str]:
    """Generates dynamic type aliases for upstream callbacks."""
    lines = [
        "// ===========================================================================",
        "// Upstream Callback Function Signatures",
        "// ===========================================================================",
    ]
    for callback_item in callbacks_list:
        callback_name = callback_item.get("name", "")
        raw_return_type = callback_item.get("returnType", "void")
        rust_return = map_c_type_to_rust(
            raw_return_type, struct_map, alias_map, callback_map
        )

        param_declarations = []
        for param in callback_item.get("params", []):
            param_identifier = escape_rust_ident(to_snake_case(param.get("name", "arg")))
            raw_param_type = param.get("type", "void")
            rust_param_type = map_c_type_to_rust(
                raw_param_type, struct_map, alias_map, callback_map
            )
            param_declarations.append(f"{param_identifier}: {rust_param_type}")

        ret_suffix = "" if rust_return == "()" else f" -> {rust_return}"
        lines.append(
            f"pub type {callback_name} = Option<unsafe extern \"C\" fn({', '.join(param_declarations)}){ret_suffix}>;"
        )

    lines.append("")
    return lines


def generate_c_structs(
    structs_list: list[dict],
    aliases_list: list[dict],
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    callback_map: dict[str, dict],
) -> list[str]:
    """Generates #[repr(C)] struct definitions and type aliases."""
    lines = [
        "// ===========================================================================",
        "// C ABI Struct Definitions & Aliases",
        "// ===========================================================================",
    ]

    # Type aliases
    for alias_item in aliases_list:
        alias_name = alias_item.get("name", "")
        raw_target = alias_item.get("type", "")
        resolved_target = resolve_alias_target(raw_target, alias_map)

        if "*" in resolved_target:
            base_ptr, _, ptr_depth = decompose_c_type(resolved_target)
            if base_ptr in struct_map:
                mut_stars = "*mut " * ptr_depth
                lines.append(f"pub type C_{alias_name} = {mut_stars}C_{base_ptr};")
            else:
                lines.append(f"pub type C_{alias_name} = *mut c_void;")
        elif resolved_target in struct_map:
            lines.append(f"pub type C_{alias_name} = C_{resolved_target};")
        else:
            lines.append(f"pub type C_{alias_name} = *mut c_void;")

    lines.append("")

    for struct_item in structs_list:
        struct_name = struct_item.get("name", "")
        lines.append("#[repr(C)]")
        lines.append("#[derive(Clone, Copy)]")
        lines.append(f"pub struct C_{struct_name} {{")

        for field_item in struct_item.get("fields", []):
            field_name = escape_rust_ident(field_item.get("name", ""))
            raw_field_type = field_item.get("type", "").strip()

            if "[" in raw_field_type:
                base_type_str = raw_field_type.split("[")[0].strip()
                size_str = raw_field_type.split("[")[1].split("]")[0].strip()
                rust_base = map_c_type_to_rust(
                    base_type_str, struct_map, alias_map, callback_map
                )
                lines.append(f"    pub {field_name}: [{rust_base}; {size_str}],")
            else:
                rust_type = map_c_type_to_rust(
                    raw_field_type, struct_map, alias_map, callback_map
                )
                lines.append(f"    pub {field_name}: {rust_type},")

        lines.append("}")
        lines.append("")

        lines.append(f"impl Default for C_{struct_name} {{")
        lines.append("    fn default() -> Self {")
        lines.append("        unsafe { std::mem::zeroed() }")
        lines.append("    }")
        lines.append("}")
        lines.append("")

    return lines


def generate_struct_marshal_helpers(
    structs_list: list[dict],
    aliases_list: list[dict],
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    callback_map: dict[str, dict],
) -> list[str]:
    """Generates unpack, pack, and writeback helpers for all structs and aliases."""
    lines = [
        "// ===========================================================================",
        "// Struct Unpack, Pack & Writeback Helpers",
        "// ===========================================================================",
        "",
    ]

    for struct_item in structs_list:
        struct_name = struct_item.get("name", "")
        snake_name = to_snake_case(struct_name)
        fields = struct_item.get("fields", [])
        field_count = len(fields)

        # 1. Writeback function
        lines.append(f"unsafe fn writeback_{snake_name}(vm_context: *mut c_void, addr: u32, val: &C_{struct_name}) {{")
        lines.append("    if addr == 0 { return; }")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")

        for field_index, field in enumerate(fields):
            field_ident = escape_rust_ident(field.get("name", ""))
            raw_type = field.get("type", "").strip()

            if raw_type in ("bool", "_Bool"):
                lines.append(f"    ((*context_ptr).set_field_i32)(vm_context, addr, {field_index}, if val.{field_ident} {{ 1 }} else {{ 0 }});")
            elif raw_type in ("unsigned char", "uint8", "uchar", "char", "int8", "short", "int16", "unsigned short", "uint16", "ushort", "int", "i32", "unsigned int", "uint", "u32"):
                lines.append(f"    ((*context_ptr).set_field_i32)(vm_context, addr, {field_index}, val.{field_ident} as i32);")
            elif raw_type in ("float", "f32"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_index}, val.{field_ident}.to_bits() as u64);")
            elif raw_type in ("double", "f64"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_index}, val.{field_ident}.to_bits());")
            elif raw_type in ("long", "long long", "int64", "unsigned long", "unsigned long long", "uint64"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_index}, val.{field_ident} as u64);")
            elif "[" in raw_type:
                base_elem = raw_type.split("[")[0].strip()
                size_str = raw_type.split("[")[1].split("]")[0].strip()
                if base_elem in ("char", "const char"):
                    lines.append(f"    let mut str_len_{field_index} = 0usize;")
                    lines.append(f"    while str_len_{field_index} < {size_str} && val.{field_ident}[str_len_{field_index}] != 0 {{")
                    lines.append(f"        str_len_{field_index} += 1;")
                    lines.append("    }")
                    lines.append(f"    let str_addr_{field_index} = ((*context_ptr).alloc_string)(vm_context, val.{field_ident}.as_ptr() as *const u8, str_len_{field_index} as u32);")
                    lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_index}, str_addr_{field_index} as u64);")
                elif base_elem in struct_map or resolve_alias_target(base_elem, alias_map) in struct_map:
                    resolved_struct = base_elem if base_elem in struct_map else resolve_alias_target(base_elem, alias_map)
                    nested_snake = to_snake_case(resolved_struct)
                    lines.append(f"    let arr_addr_{field_index} = ((*context_ptr).alloc_array)(vm_context, {size_str}, 0);")
                    lines.append(f"    for elem_idx in 0..{size_str}u32 {{")
                    lines.append(f"        let child_addr = pack_{nested_snake}(vm_context, val.{field_ident}[elem_idx as usize]);")
                    lines.append(f"        ((*context_ptr).array_set_u64)(vm_context, arr_addr_{field_index}, elem_idx, child_addr as u64);")
                    lines.append("    }")
                    lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_index}, arr_addr_{field_index} as u64);")
                else:
                    lines.append(f"    let arr_addr_{field_index} = ((*context_ptr).alloc_array)(vm_context, {size_str}, 0);")
                    lines.append(f"    for elem_idx in 0..{size_str}u32 {{")
                    if base_elem in ("float", "f32"):
                        lines.append(f"        ((*context_ptr).array_set_u64)(vm_context, arr_addr_{field_index}, elem_idx, val.{field_ident}[elem_idx as usize].to_bits() as u64);")
                    elif base_elem in ("double", "f64"):
                        lines.append(f"        ((*context_ptr).array_set_u64)(vm_context, arr_addr_{field_index}, elem_idx, val.{field_ident}[elem_idx as usize].to_bits());")
                    else:
                        lines.append(f"        ((*context_ptr).array_set_u64)(vm_context, arr_addr_{field_index}, elem_idx, val.{field_ident}[elem_idx as usize] as u64);")
                    lines.append("    }")
                    lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_index}, arr_addr_{field_index} as u64);")
            elif raw_type in struct_map or resolve_alias_target(raw_type, alias_map) in struct_map:
                resolved_struct = raw_type if raw_type in struct_map else resolve_alias_target(raw_type, alias_map)
                nested_snake = to_snake_case(resolved_struct)
                lines.append(f"    let child_{field_index} = pack_{nested_snake}(vm_context, val.{field_ident});")
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_index}, child_{field_index} as u64);")
            elif raw_type.endswith("*") or raw_type.startswith("void") or "*" in resolve_alias_target(raw_type, alias_map):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_index}, val.{field_ident} as usize as u64);")

        lines.append("}")
        lines.append("")

        # 2. Pack function (allocates object and calls writeback)
        lines.append(f"unsafe fn pack_{snake_name}(vm_context: *mut c_void, val: C_{struct_name}) -> u32 {{")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
        lines.append(f'    let class_name_str = c"{struct_name}";')
        lines.append("    let class_vtable_id = ((*context_ptr).get_class_vtable_id)(vm_context, class_name_str.as_ptr());")
        lines.append(f"    let addr = ((*context_ptr).alloc_object)(vm_context, class_vtable_id, {field_count});")
        lines.append("    if addr > 0 {")
        lines.append(f"        writeback_{snake_name}(vm_context, addr, &val);")
        lines.append("    }")
        lines.append("    addr")
        lines.append("}")
        lines.append("")

        # 3. Unpack function
        lines.append(f"unsafe fn unpack_{snake_name}(vm_context: *mut c_void, addr: u32) -> C_{struct_name} {{")
        lines.append(f"    if addr == 0 {{ return C_{struct_name}::default(); }}")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
        lines.append(f"    let mut result = C_{struct_name}::default();")

        for field_index, field in enumerate(fields):
            field_ident = escape_rust_ident(field.get("name", ""))
            raw_type = field.get("type", "").strip()

            if raw_type in ("bool", "_Bool"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_index}) != 0;")
            elif raw_type in ("unsigned char", "uint8", "uchar"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_index}) as u8;")
            elif raw_type in ("char", "int8"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_index}) as c_char;")
            elif raw_type in ("short", "int16"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_index}) as c_short;")
            elif raw_type in ("unsigned short", "uint16", "ushort"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_index}) as c_ushort;")
            elif raw_type in ("int", "i32"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_index}) as c_int;")
            elif raw_type in ("unsigned int", "uint", "u32"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_index}) as c_uint;")
            elif raw_type in ("float", "f32"):
                lines.append(f"    result.{field_ident} = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, {field_index}) as u32);")
            elif raw_type in ("double", "f64"):
                lines.append(f"    result.{field_ident} = f64::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, {field_index}));")
            elif raw_type in ("long", "long long", "int64"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_index}) as c_longlong;")
            elif raw_type in ("unsigned long", "unsigned long long", "uint64"):
                lines.append(f"    result.{field_ident} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_index}) as c_ulonglong;")
            elif "[" in raw_type:
                base_elem = raw_type.split("[")[0].strip()
                size_str = raw_type.split("[")[1].split("]")[0].strip()
                if base_elem in ("char", "const char"):
                    lines.append(f"    let str_addr_{field_index} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_index}) as u32;")
                    lines.append(f"    if str_addr_{field_index} > 0 {{")
                    lines.append(f"        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_{field_index}, result.{field_ident}.as_mut_ptr() as *mut u8, ({size_str} - 1) as u32) as usize;")
                    lines.append(f"        result.{field_ident}[read_len] = 0;")
                    lines.append("    }")
                elif base_elem in struct_map or resolve_alias_target(base_elem, alias_map) in struct_map:
                    resolved_struct = base_elem if base_elem in struct_map else resolve_alias_target(base_elem, alias_map)
                    nested_snake = to_snake_case(resolved_struct)
                    lines.append(f"    let arr_addr_{field_index} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_index}) as u32;")
                    lines.append(f"    if arr_addr_{field_index} > 0 {{")
                    lines.append(f"        let arr_len = ((*context_ptr).array_len)(vm_context, arr_addr_{field_index}).min({size_str});")
                    lines.append("        for elem_idx in 0..arr_len {")
                    lines.append(f"            let child_addr = ((*context_ptr).array_get_u64)(vm_context, arr_addr_{field_index}, elem_idx) as u32;")
                    lines.append(f"            result.{field_ident}[elem_idx as usize] = unpack_{nested_snake}(vm_context, child_addr);")
                    lines.append("        }")
                    lines.append("    }")
                else:
                    lines.append(f"    let arr_addr_{field_index} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_index}) as u32;")
                    lines.append(f"    if arr_addr_{field_index} > 0 {{")
                    lines.append(f"        let arr_len = ((*context_ptr).array_len)(vm_context, arr_addr_{field_index}).min({size_str});")
                    lines.append("        for elem_idx in 0..arr_len {")
                    lines.append(f"            let raw_val = ((*context_ptr).array_get_u64)(vm_context, arr_addr_{field_index}, elem_idx);")
                    if base_elem in ("float", "f32"):
                        lines.append(f"            result.{field_ident}[elem_idx as usize] = f32::from_bits(raw_val as u32);")
                    elif base_elem in ("double", "f64"):
                        lines.append(f"            result.{field_ident}[elem_idx as usize] = f64::from_bits(raw_val);")
                    else:
                        rust_elem_cast = map_c_type_to_rust(base_elem, struct_map, alias_map, callback_map)
                        lines.append(f"            result.{field_ident}[elem_idx as usize] = raw_val as {rust_elem_cast};")
                    lines.append("        }")
                    lines.append("    }")
            elif raw_type in struct_map or resolve_alias_target(raw_type, alias_map) in struct_map:
                resolved_struct = raw_type if raw_type in struct_map else resolve_alias_target(raw_type, alias_map)
                nested_snake = to_snake_case(resolved_struct)
                lines.append(f"    result.{field_ident} = unpack_{nested_snake}(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, {field_index}) as u32);")
            elif raw_type.endswith("*") or raw_type.startswith("void") or "*" in resolve_alias_target(raw_type, alias_map):
                rust_cast = map_c_type_to_rust(raw_type, struct_map, alias_map, callback_map)
                lines.append(f"    result.{field_ident} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_index}) as usize as {rust_cast};")

        lines.append("    result")
        lines.append("}")
        lines.append("")

    # Alias unpack/pack/writeback forwarders
    for alias_item in aliases_list:
        alias_name = alias_item.get("name", "")
        raw_target = alias_item.get("type", "")
        resolved_target = resolve_alias_target(raw_target, alias_map)

        if "*" not in resolved_target and resolved_target in struct_map:
            snake_alias = to_snake_case(alias_name)
            snake_target = to_snake_case(resolved_target)
            lines.append(f"unsafe fn unpack_{snake_alias}(vm_context: *mut c_void, addr: u32) -> C_{alias_name} {{ unpack_{snake_target}(vm_context, addr) }}")
            lines.append(f"unsafe fn pack_{snake_alias}(vm_context: *mut c_void, val: C_{alias_name}) -> u32 {{ pack_{snake_target}(vm_context, val) }}")
            lines.append(f"unsafe fn writeback_{snake_alias}(vm_context: *mut c_void, addr: u32, val: &C_{alias_name}) {{ writeback_{snake_target}(vm_context, addr, val); }}")

    lines.append("")
    return lines


def generate_extern_c_declarations(
    functions_list: list[dict],
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    callback_map: dict[str, dict],
) -> list[str]:
    """Generates extern \"C\" declarations for upstream C functions."""
    lines = [
        "// ===========================================================================",
        "// Upstream Extern \"C\" Declarations",
        "// ===========================================================================",
        "unsafe extern \"C\" {",
    ]

    for func in functions_list:
        func_name = func.get("name", "")
        raw_ret = func.get("returnType", "void")
        rust_ret = map_c_type_to_rust(raw_ret, struct_map, alias_map, callback_map)

        param_decls = []
        for param_idx, param in enumerate(func.get("params", [])):
            raw_param_type = param.get("type", "")
            if raw_param_type == "...":
                param_decls.append("...")
            else:
                param_var = f"arg_{param_idx}"
                rust_type = map_c_type_to_rust(
                    raw_param_type, struct_map, alias_map, callback_map
                )
                param_decls.append(f"{param_var}: {rust_type}")

        ret_suffix = "" if rust_ret == "()" else f" -> {rust_ret}"
        lines.append(f"    pub fn {func_name}({', '.join(param_decls)}){ret_suffix};")

    lines.append("}")
    lines.append("")
    return lines


def generate_wrappers(
    functions_list: list[dict],
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    callback_map: dict[str, dict],
    class_name: str,
    package_name: str,
) -> tuple[list[str], list[tuple[str, str]]]:
    """Generates Solix C-ABI wrapper functions and symbol registration lists."""
    wrapper_lines = [
        "// ===========================================================================",
        "// Solix C-ABI Native Wrapper Functions",
        "// ===========================================================================",
    ]
    registration_entries: list[tuple[str, str]] = []

    for func in functions_list:
        func_name = func.get("name", "")
        wrapper_name = f"{class_name}_{func_name}"
        raw_ret = func.get("returnType", "void").strip()
        params = func.get("params", [])

        is_variadic = any(param.get("type") == "..." for param in params)

        wrapper_lines.append("#[unsafe(no_mangle)]")
        wrapper_lines.append(f"pub unsafe extern \"C\" fn {wrapper_name}(")
        wrapper_lines.append("    vm_context: *mut c_void,")
        wrapper_lines.append("    args: *const SolixCValue,")
        wrapper_lines.append("    arg_count: u32,")
        wrapper_lines.append("    out_error: *mut SolixCValue,")
        wrapper_lines.append(") -> SolixCValue {")

        call_args = []
        pre_call_lines = []
        post_call_lines = []

        # Filter out "..." parameter
        non_variadic_params = [param for param in params if param.get("type") != "..."]

        for param_idx, param in enumerate(non_variadic_params):
            arg_expr = f"(*args.add({param_idx}))"
            raw_param_type = param.get("type", "").strip()
            base_param, is_const_param, ptr_depth = decompose_c_type(raw_param_type)
            resolved_param = resolve_alias_target(base_param, alias_map)

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
                buf_var = f"str_buf_{param_idx}"
                ptr_var = f"str_ptr_{param_idx}"
                addr_var = f"str_addr_{param_idx}"
                pre_call_lines.append(f"    let {addr_var} = {arg_expr}.payload as u32;")
                pre_call_lines.append(f"    let mut {buf_var} = [0u8; 1024];")
                pre_call_lines.append(f"    let {ptr_var} = if {addr_var} > 0 {{")
                pre_call_lines.append("        let context_ptr = vm_context as *mut SolixVmContext;")
                pre_call_lines.append(f"        let read_len = ((*context_ptr).read_bytes)(vm_context, {addr_var}, {buf_var}.as_mut_ptr(), 1023) as usize;")
                pre_call_lines.append(f"        {buf_var}[read_len] = 0;")
                pre_call_lines.append(f"        {buf_var}.as_ptr() as *const c_char")
                pre_call_lines.append("    } else {")
                pre_call_lines.append('        c"".as_ptr()')
                pre_call_lines.append("    };")
                call_args.append(ptr_var)
            elif base_param in callback_map or resolved_param in callback_map:
                call_args.append(
                    f"if {arg_expr}.payload == 0 {{ None }} else {{ Some(std::mem::transmute({arg_expr}.payload as usize)) }}"
                )
            # Struct values
            elif ptr_depth == 0 and (base_param in struct_map or resolved_param in struct_map):
                concrete_struct = base_param if base_param in struct_map else resolved_param
                struct_snake = to_snake_case(concrete_struct)
                call_args.append(f"unpack_{struct_snake}(vm_context, {arg_expr}.payload as u32)")
            # Mutable struct pointers (in-place writeback)
            elif ptr_depth == 1 and not is_const_param and (base_param in struct_map or resolved_param in struct_map):
                concrete_struct = base_param if base_param in struct_map else resolved_param
                struct_snake = to_snake_case(concrete_struct)
                var_name = f"{struct_snake}_mut_{param_idx}"
                addr_name = f"{struct_snake}_addr_{param_idx}"
                pre_call_lines.append(f"    let {addr_name} = {arg_expr}.payload as u32;")
                pre_call_lines.append(f"    let mut {var_name} = unpack_{struct_snake}(vm_context, {addr_name});")
                call_args.append(f"&mut {var_name}")
                post_call_lines.append(f"    writeback_{struct_snake}(vm_context, {addr_name}, &{var_name});")
            # Struct arrays (const T *)
            elif ptr_depth == 1 and is_const_param and (base_param in struct_map or resolved_param in struct_map):
                concrete_struct = base_param if base_param in struct_map else resolved_param
                struct_snake = to_snake_case(concrete_struct)
                vec_name = f"struct_vec_{param_idx}"
                arr_name = f"arr_addr_{param_idx}"
                pre_call_lines.append(f"    let {arr_name} = {arg_expr}.payload as u32;")
                pre_call_lines.append(f"    let mut {vec_name}: Vec<C_{concrete_struct}> = Vec::new();")
                pre_call_lines.append(f"    if {arr_name} > 0 {{")
                pre_call_lines.append("        let context_ptr = vm_context as *mut SolixVmContext;")
                pre_call_lines.append(f"        let total_count = ((*context_ptr).array_len)(vm_context, {arr_name});")
                pre_call_lines.append(f"        {vec_name}.reserve_exact(total_count as usize);")
                pre_call_lines.append("        for elem_idx in 0..total_count {")
                pre_call_lines.append(f"            let child_addr = ((*context_ptr).array_get_u64)(vm_context, {arr_name}, elem_idx) as u32;")
                pre_call_lines.append(f"            {vec_name}.push(unpack_{struct_snake}(vm_context, child_addr));")
                pre_call_lines.append("        }")
                pre_call_lines.append("    }")
                call_args.append(f"if {arr_name} > 0 {{ {vec_name}.as_ptr() }} else {{ std::ptr::null() }}")
            # Primitive arrays (const int *, int *, float *, etc.)
            elif ptr_depth == 1 and base_param in (
                "int", "i32", "unsigned int", "uint", "u32", "float", "f32", "double", "f64",
                "char", "int8", "unsigned char", "uint8", "uchar", "short", "int16", "unsigned short", "uint16", "ushort"
            ):
                rust_elem = map_c_type_to_rust(base_param, struct_map, alias_map, callback_map)
                vec_name = f"prim_vec_{param_idx}"
                arr_name = f"arr_addr_{param_idx}"
                pre_call_lines.append(f"    let {arr_name} = {arg_expr}.payload as u32;")
                pre_call_lines.append(f"    let mut {vec_name}: Vec<{rust_elem}> = Vec::new();")
                pre_call_lines.append(f"    if {arr_name} > 0 {{")
                pre_call_lines.append("        let context_ptr = vm_context as *mut SolixVmContext;")
                pre_call_lines.append(f"        let total_count = ((*context_ptr).array_len)(vm_context, {arr_name});")
                pre_call_lines.append(f"        {vec_name}.reserve_exact(total_count as usize);")
                pre_call_lines.append("        for elem_idx in 0..total_count {")
                pre_call_lines.append(f"            let raw_val = ((*context_ptr).array_get_u64)(vm_context, {arr_name}, elem_idx);")
                if base_param in ("float", "f32"):
                    pre_call_lines.append(f"            {vec_name}.push(f32::from_bits(raw_val as u32));")
                elif base_param in ("double", "f64"):
                    pre_call_lines.append(f"            {vec_name}.push(f64::from_bits(raw_val));")
                else:
                    pre_call_lines.append(f"            {vec_name}.push(raw_val as {rust_elem});")
                pre_call_lines.append("        }")
                pre_call_lines.append("    }")
                call_args.append(f"if {arr_name} > 0 {{ {vec_name}.as_mut_ptr() }} else {{ std::ptr::null_mut() }}")

                # Writeback for mutable primitive arrays
                if not is_const_param:
                    post_call_lines.append(f"    if {arr_name} > 0 {{")
                    post_call_lines.append("        let context_ptr = vm_context as *mut SolixVmContext;")
                    post_call_lines.append(f"        let total_count = ((*context_ptr).array_len)(vm_context, {arr_name});")
                    post_call_lines.append("        for elem_idx in 0..total_count {")
                    post_call_lines.append(f"            let updated_elem = {vec_name}[elem_idx as usize];")
                    if base_param in ("float", "f32"):
                        post_call_lines.append(f"            ((*context_ptr).array_set_u64)(vm_context, {arr_name}, elem_idx, updated_elem.to_bits() as u64);")
                    elif base_param in ("double", "f64"):
                        post_call_lines.append(f"            ((*context_ptr).array_set_u64)(vm_context, {arr_name}, elem_idx, updated_elem.to_bits());")
                    else:
                        post_call_lines.append(f"            ((*context_ptr).array_set_u64)(vm_context, {arr_name}, elem_idx, updated_elem as u64);")
                    post_call_lines.append("        }")
                    post_call_lines.append("    }")
            else:
                rust_cast = map_c_type_to_rust(raw_param_type, struct_map, alias_map, callback_map)
                call_args.append(f"{arg_expr}.payload as usize as {rust_cast}")

        # Emit pre-call code
        for pre_line in pre_call_lines:
            wrapper_lines.append(pre_line)

        # Variadic invocation handling
        if is_variadic:
            last_arg = call_args.pop() if call_args else 'c"".as_ptr()'
            if call_args:
                call_invocation = f"{func_name}({', '.join(call_args)}, c\"%s\".as_ptr(), {last_arg})"
            else:
                call_invocation = f"{func_name}(c\"%s\".as_ptr(), {last_arg})"
        else:
            call_invocation = f"{func_name}({', '.join(call_args)})"

        # Return value marshaling
        base_ret, is_const_ret, ret_ptr_depth = decompose_c_type(raw_ret)
        resolved_ret = resolve_alias_target(base_ret, alias_map)

        if raw_ret == "void":
            wrapper_lines.append(f"    {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append("    SolixCValue::null()")
        elif raw_ret in ("bool", "_Bool"):
            wrapper_lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append("    SolixCValue::bool_val(call_result)")
        elif raw_ret in ("int", "i32", "unsigned int", "uint", "u32", "short", "int16", "unsigned short", "uint16", "ushort", "char", "int8", "unsigned char", "uint8", "uchar"):
            wrapper_lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append("    SolixCValue::i32_val(call_result as i32)")
        elif raw_ret in ("long", "long long", "int64", "unsigned long", "unsigned long long", "uint64"):
            wrapper_lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append("    SolixCValue::i64_val(call_result as i64)")
        elif raw_ret in ("float", "f32"):
            wrapper_lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append("    SolixCValue::f64_val(call_result as f64)")
        elif raw_ret in ("double", "f64"):
            wrapper_lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append("    SolixCValue::f64_val(call_result)")
        elif raw_ret in ("const char *", "char *"):
            wrapper_lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append("    if call_result.is_null() { return SolixCValue::null(); }")
            wrapper_lines.append("    let c_str = CStr::from_ptr(call_result);")
            wrapper_lines.append("    let bytes = c_str.to_bytes();")
            wrapper_lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
            wrapper_lines.append("    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);")
            wrapper_lines.append("    SolixCValue::ref_val(addr)")
        elif ret_ptr_depth == 0 and (base_ret in struct_map or resolved_ret in struct_map):
            concrete_struct = base_ret if base_ret in struct_map else resolved_ret
            struct_snake = to_snake_case(concrete_struct)
            wrapper_lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append(f"    SolixCValue::ref_val(pack_{struct_snake}(vm_context, call_result))")
        else:
            wrapper_lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                wrapper_lines.append(post_line)
            wrapper_lines.append("    SolixCValue::i64_val(call_result as usize as i64)")

        wrapper_lines.append("}")
        wrapper_lines.append("")

        snake_name = to_snake_case(func_name)
        registration_entries.append((f"{class_name}::{func_name}", wrapper_name))
        registration_entries.append((f"{class_name}::{snake_name}", wrapper_name))
        registration_entries.append((f"{package_name}::{func_name}", wrapper_name))
        registration_entries.append((f"{package_name}::{snake_name}", wrapper_name))
        registration_entries.append((func_name, wrapper_name))
        registration_entries.append((snake_name, wrapper_name))

    return wrapper_lines, registration_entries


def generate_registration(registration_entries: list[tuple[str, str]]) -> list[str]:
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
        lines.append(
            f'    register_callback(collector_context, c"{symbol_path}".as_ptr(), {wrapper_name});'
        )

    lines.append("}")
    lines.append("")
    return lines


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate generic Rust FFI companion bridge from rlparser JSON."
    )
    parser.add_argument(
        "--api",
        default="build/.cache/api/raylib_api.json",
        help="Path to rlparser API JSON schema",
    )
    parser.add_argument(
        "--output",
        default="natives/raylib/src/lib.rs",
        help="Path to output Rust source file",
    )
    parser.add_argument(
        "--package-name",
        default="raylib",
        help="Solix package name",
    )
    parser.add_argument(
        "--class-name",
        default="Raylib",
        help="Solix primary class container name",
    )
    parser.add_argument(
        "--define-guard",
        default="RLAPI",
        help="API macro define guard to skip",
    )
    cli_arguments = parser.parse_args()

    api_path = Path(cli_arguments.api)
    output_path = Path(cli_arguments.output)

    if not api_path.exists():
        print(f"Error: API schema not found at '{api_path}'", file=sys.stderr)
        sys.exit(1)

    with open(api_path, "r", encoding="utf-8") as schema_file:
        api_data = json.load(schema_file)

    struct_map = {
        struct_item["name"]: struct_item
        for struct_item in api_data.get("structs", [])
    }
    alias_map = {
        alias_item["name"]: alias_item["type"]
        for alias_item in api_data.get("aliases", [])
    }
    enum_map = {
        enum_item["name"]: enum_item
        for enum_item in api_data.get("enums", [])
    }
    callback_map = {
        callback_item["name"]: callback_item
        for callback_item in api_data.get("callbacks", [])
    }

    structs_list = api_data.get("structs", [])
    aliases_list = api_data.get("aliases", [])
    functions_list = api_data.get("functions", [])
    callbacks_list = api_data.get("callbacks", [])

    output_lines = []
    output_lines.extend(generate_header())
    output_lines.extend(
        generate_callbacks(callbacks_list, struct_map, alias_map, callback_map)
    )
    output_lines.extend(
        generate_c_structs(
            structs_list, aliases_list, struct_map, alias_map, callback_map
        )
    )
    output_lines.extend(
        generate_struct_marshal_helpers(
            structs_list, aliases_list, struct_map, alias_map, callback_map
        )
    )
    output_lines.extend(
        generate_extern_c_declarations(
            functions_list, struct_map, alias_map, callback_map
        )
    )

    wrapper_lines, registration_entries = generate_wrappers(
        functions_list,
        struct_map,
        alias_map,
        callback_map,
        cli_arguments.class_name,
        cli_arguments.package_name,
    )
    output_lines.extend(wrapper_lines)
    output_lines.extend(generate_registration(registration_entries))

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as destination_file:
        destination_file.write("\n".join(output_lines) + "\n")

    print(f"Successfully generated {output_path} ({len(output_lines)} lines)")


if __name__ == "__main__":
    main()
