#!/usr/bin/env python3
"""
generate_bindings.py

Unified Two-Pass Intermediate Representation (IR) C-API Binding Generator.
100% generic, schema-driven generator emitting Solix declarations (.slx)
and Rust C-ABI companion bridge (.rs) with zero hardcoded engine names.

Follows Solix language conventions and GL-0011 (zero single-letter variables).
"""

from __future__ import annotations

import argparse
import ast
from dataclasses import dataclass, field
import json
import os
import re
import sys
from pathlib import Path


# ============================================================================
# Keyword Tables & Sanitization
# ============================================================================

SOLIX_RESERVED_KEYWORDS = {
    "class", "interface", "enum", "alias", "package", "import", "extern", "native",
    "public", "private", "protected", "internal", "static", "virtual", "override",
    "abstract", "final", "inline", "const", "constexpr", "weak", "if", "else",
    "while", "for", "do", "switch", "case", "default", "break", "continue",
    "return", "throw", "try", "catch", "finally", "new", "this", "super",
    "instanceof", "sizeof", "typeof", "as", "is", "operator", "null", "true",
    "false", "void", "bool", "char", "int8", "int16", "int32", "int64",
    "uint8", "uint16", "uint32", "uint64", "float32", "float64", "string",
}

RUST_RESERVED_KEYWORDS = {
    "type", "match", "ref", "move", "fn", "loop", "in", "box", "yield",
    "const", "static", "crate", "super", "self", "Self", "let", "mut",
    "pub", "use", "where", "mod", "as", "struct", "enum", "trait", "impl"
}


def sanitize_solix_ident(identifier_name: str) -> str:
    """Escapes Solix reserved keywords by appending an underscore suffix."""
    if identifier_name in SOLIX_RESERVED_KEYWORDS:
        return f"{identifier_name}_val"
    return identifier_name


def escape_rust_ident(ident_name: str) -> str:
    """Escapes Rust keywords using raw identifier syntax r#ident."""
    if ident_name in RUST_RESERVED_KEYWORDS:
        return f"r#{ident_name}"
    return ident_name


def to_snake_case(pascal_str: str) -> str:
    """Converts a PascalCase string to snake_case."""
    snake_chars = []
    str_length = len(pascal_str)
    for index, char_elem in enumerate(pascal_str):
        if char_elem.isupper():
            if index > 0 and (
                pascal_str[index - 1].islower()
                or (index + 1 < str_length and pascal_str[index + 1].islower())
            ):
                snake_chars.append("_")
            snake_chars.append(char_elem.lower())
        else:
            snake_chars.append(char_elem)
    result = "".join(snake_chars)
    if result in SOLIX_RESERVED_KEYWORDS:
        return f"{result}_fn"
    return result


def resolve_alias_recursively(type_str: str, alias_map: dict[str, str]) -> str:
    """Recursively resolves an alias to its underlying concrete target type."""
    visited = set()
    current_target = type_str.strip()
    while current_target in alias_map and current_target not in visited:
        visited.add(current_target)
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


def eval_safe_math(expression_str: str, known_constants: dict[str, float]) -> float:
    """Evaluates safe arithmetic expressions substituting previously declared constants."""
    normalized_expr = expression_str.strip()
    for const_name, const_val in sorted(
        known_constants.items(), key=lambda pair: len(pair[0]), reverse=True
    ):
        normalized_expr = re.sub(
            r"\b" + re.escape(const_name) + r"\b",
            str(const_val),
            normalized_expr,
        )

    # Strip float literals like 180.0f -> 180.0
    normalized_expr = re.sub(r"(\d+(?:\.\d+)?)[fF]\b", r"\1", normalized_expr)

    parsed_ast = ast.parse(normalized_expr, mode="eval")

    def evaluate_node(ast_node: ast.AST) -> float:
        if isinstance(ast_node, ast.Expression):
            return evaluate_node(ast_node.body)
        if isinstance(ast_node, ast.Constant) and isinstance(
            ast_node.value, (int, float)
        ):
            return float(ast_node.value)
        if isinstance(ast_node, ast.UnaryOp):
            operand_val = evaluate_node(ast_node.operand)
            if isinstance(ast_node.op, ast.USub):
                return -operand_val
            if isinstance(ast_node.op, ast.UAdd):
                return operand_val
        if isinstance(ast_node, ast.BinOp):
            left_val = evaluate_node(ast_node.left)
            right_val = evaluate_node(ast_node.right)
            if isinstance(ast_node.op, ast.Add):
                return left_val + right_val
            if isinstance(ast_node.op, ast.Sub):
                return left_val - right_val
            if isinstance(ast_node.op, ast.Mult):
                return left_val * right_val
            if isinstance(ast_node.op, ast.Div):
                return left_val / right_val
        raise ValueError(f"Unsupported AST node in math evaluation: {ast.dump(ast_node)}")

    return evaluate_node(parsed_ast)


# ============================================================================
# Two-Pass Intermediate Representation (IR) Data Structures
# ============================================================================

@dataclass
class DefineIR:
    name: str
    item_type: str
    raw_value: str
    evaluated_value: str | float | int | None
    description: str


@dataclass
class EnumValueIR:
    name: str
    value: int | str
    description: str


@dataclass
class EnumIR:
    name: str
    description: str
    values: list[EnumValueIR]


@dataclass
class FieldIR:
    name: str
    escaped_solix_name: str
    escaped_rust_name: str
    raw_type: str
    solix_type: str
    rust_type: str
    is_array: bool
    is_pointer: bool
    array_size: int | None
    array_base_type: str | None
    array_elem_rust_type: str | None
    description: str


@dataclass
class StructLiteralPresetIR:
    name: str
    target_struct: str
    raw_args: list[str]
    description: str


@dataclass
class StructIR:
    name: str
    snake_name: str
    description: str
    fields: list[FieldIR]
    presets: list[StructLiteralPresetIR] = field(default_factory=list)


@dataclass
class AliasIR:
    name: str
    snake_name: str
    target_type: str
    resolved_target: str
    is_pointer_alias: bool
    description: str


@dataclass
class ParamIR:
    name: str
    escaped_solix_name: str
    escaped_rust_name: str
    raw_type: str
    solix_type: str
    rust_type: str
    is_variadic: bool
    is_string: bool
    is_callback: bool
    is_struct_val: bool
    is_mutable_struct_ptr: bool
    is_struct_array: bool
    is_primitive_array: bool
    is_mutable_primitive_array: bool
    base_struct_name: str | None
    base_primitive_name: str | None


@dataclass
class CallbackIR:
    name: str
    return_type: str
    rust_return_type: str
    params: list[ParamIR]
    description: str


@dataclass
class FunctionIR:
    name: str
    snake_name: str
    return_type: str
    solix_return_type: str
    rust_return_type: str
    params: list[ParamIR]
    is_variadic: bool
    description: str


@dataclass
class ApiIR:
    defines: list[DefineIR]
    enums: list[EnumIR]
    structs: list[StructIR]
    aliases: list[AliasIR]
    callbacks: list[CallbackIR]
    functions: list[FunctionIR]
    known_constants: dict[str, float]
    struct_map: dict[str, StructIR]
    alias_map: dict[str, str]
    enum_map: dict[str, EnumIR]
    callback_map: dict[str, CallbackIR]


# ============================================================================
# Pass 1: Parse JSON Schema into Pure IR Lists
# ============================================================================

def parse_api_to_ir(api_data: dict, define_guard: str) -> ApiIR:
    """Normalizes raw C-API JSON schema into a fully resolved two-pass IR representation."""
    # 1. Build fundamental raw lookup maps
    raw_structs = api_data.get("structs", [])
    raw_aliases = api_data.get("aliases", [])
    raw_enums = api_data.get("enums", [])
    raw_callbacks = api_data.get("callbacks", [])
    raw_defines = api_data.get("defines", [])
    raw_functions = api_data.get("functions", [])

    raw_struct_names = {item["name"] for item in raw_structs}
    raw_enum_names = {item["name"] for item in raw_enums}
    raw_callback_names = {item["name"] for item in raw_callbacks}
    alias_map: dict[str, str] = {
        item["name"]: item["type"] for item in raw_aliases
    }

    # 2. Parse defines and safe mathematical expressions
    known_constants: dict[str, float] = {
        "PI": 3.14159265358979323846,
    }
    defines_ir: list[DefineIR] = []
    preset_literals_map: dict[str, list[StructLiteralPresetIR]] = {
        struct_name: [] for struct_name in raw_struct_names
    }

    literal_pattern = re.compile(
        r"(?:CLITERAL\s*\(\s*([A-Za-z0-9_]+)\s*\)|\(\s*([A-Za-z0-9_]+)\s*\))\s*\{([^}]+)\}"
    )

    for define_item in raw_defines:
        def_name = str(define_item.get("name", ""))
        def_type = str(define_item.get("type", ""))
        raw_val = str(define_item.get("value", ""))
        def_desc = str(define_item.get("description", ""))

        if def_name == define_guard or def_type in ("GUARD", "MACRO"):
            continue

        # Check for struct-literal defines
        match_lit = literal_pattern.search(raw_val)
        if match_lit:
            target_struct = match_lit.group(1) or match_lit.group(2)
            if target_struct in preset_literals_map:
                inner_comps = [comp.strip() for comp in match_lit.group(3).split(",")]
                preset_literals_map[target_struct].append(
                    StructLiteralPresetIR(
                        name=def_name,
                        target_struct=target_struct,
                        raw_args=inner_comps,
                        description=def_desc,
                    )
                )
            continue

        eval_val = None
        if def_type == "INT":
            try:
                eval_val = int(raw_val, 0)
                known_constants[def_name] = float(eval_val)
            except ValueError:
                eval_val = raw_val
        elif def_type == "FLOAT":
            try:
                eval_val = float(raw_val.rstrip("fF"))
                known_constants[def_name] = eval_val
            except ValueError:
                eval_val = raw_val
        elif def_type == "STRING":
            eval_val = raw_val
        elif def_type == "FLOAT_MATH":
            try:
                eval_val = eval_safe_math(raw_val, known_constants)
                known_constants[def_name] = eval_val
            except Exception:
                pass

        defines_ir.append(
            DefineIR(
                name=def_name,
                item_type=def_type,
                raw_value=raw_val,
                evaluated_value=eval_val,
                description=def_desc,
            )
        )

    # 3. Type mapping helpers
    def map_c_type_to_solix(raw_c_type: str) -> str:
        normalized = raw_c_type.strip()
        if normalized == "void":
            return "void"
        if normalized in ("bool", "_Bool"):
            return "bool"
        if normalized in ("int", "i32"):
            return "int32"
        if normalized in ("unsigned int", "uint", "u32"):
            return "uint32"
        if normalized in ("float", "f32"):
            return "float32"
        if normalized in ("double", "f64"):
            return "float64"
        if normalized in ("char", "int8"):
            return "int8"
        if normalized in ("unsigned char", "uint8", "uchar"):
            return "uint8"
        if normalized in ("short", "int16"):
            return "int16"
        if normalized in ("unsigned short", "uint16", "ushort"):
            return "uint16"
        if normalized in ("long", "long long", "int64"):
            return "int64"
        if normalized in ("unsigned long", "unsigned long long", "uint64"):
            return "uint64"

        if normalized in ("const char *", "char *"):
            return "string"
        if normalized in ("const char **", "char **"):
            return "string[]"

        if "[" in normalized:
            base_part = normalized.split("[")[0].strip()
            if base_part in ("char", "const char"):
                return "string"
            return f"{map_c_type_to_solix(base_part)}[]"

        base_name, is_const, pointer_depth = decompose_c_type(normalized)
        resolved_base = resolve_alias_recursively(base_name, alias_map)

        # Pointer aliases, callbacks, opaque pointers
        if (
            "*" in resolved_base
            or base_name in raw_callback_names
            or resolved_base in raw_callback_names
            or base_name in ("void", "va_list")
            or resolved_base in ("void", "va_list")
            or pointer_depth >= 2
        ):
            return "uint64"

        primitive_solix = {
            "int": "int32", "i32": "int32",
            "unsigned int": "uint32", "uint": "uint32", "u32": "uint32",
            "float": "float32", "f32": "float32",
            "double": "float64", "f64": "float64",
            "char": "int8", "int8": "int8",
            "unsigned char": "uint8", "uint8": "uint8", "uchar": "uint8",
            "short": "int16", "int16": "int16",
            "unsigned short": "uint16", "uint16": "uint16", "ushort": "uint16",
            "long": "int64", "long long": "int64", "int64": "int64",
            "unsigned long": "uint64", "unsigned long long": "uint64", "uint64": "uint64",
        }
        if pointer_depth == 1 and base_name in primitive_solix:
            return f"{primitive_solix[base_name]}[]"

        target_struct = (
            base_name if base_name in raw_struct_names
            else (resolved_base if resolved_base in raw_struct_names else None)
        )
        if target_struct is not None:
            if pointer_depth == 0:
                return base_name
            if pointer_depth == 1:
                return f"{base_name}[]" if is_const else base_name

        if base_name in raw_enum_names:
            return f"{base_name}[]" if pointer_depth == 1 else base_name

        if pointer_depth > 0:
            return "uint64"

        return base_name

    def map_c_type_to_rust(raw_c_type: str) -> str:
        normalized = raw_c_type.strip()
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
        resolved_base = resolve_alias_recursively(base_name, alias_map)

        if base_name in raw_callback_names:
            return base_name
        if resolved_base in raw_callback_names:
            return resolved_base

        if "*" in resolved_base:
            alias_base, _, alias_depth = decompose_c_type(resolved_base)
            target_st = (
                alias_base if alias_base in raw_struct_names
                else resolve_alias_recursively(alias_base, alias_map)
            )
            if target_st in raw_struct_names:
                mut_stars = "*mut " * (pointer_depth + alias_depth)
                return f"{mut_stars}C_{target_st}"
            return "*mut c_void"

        target_struct = (
            base_name if base_name in raw_struct_names
            else (resolved_base if resolved_base in raw_struct_names else None)
        )
        if target_struct is not None:
            if pointer_depth == 0:
                return f"C_{target_struct}"
            if pointer_depth == 1:
                mut_or_const = "*const" if is_const else "*mut"
                return f"{mut_or_const} C_{target_struct}"
            return "*mut c_void"

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

    # 4. Parse Enums
    enums_ir: list[EnumIR] = []
    enum_map: dict[str, EnumIR] = {}
    for enum_item in raw_enums:
        enum_values = [
            EnumValueIR(
                name=val_item.get("name", ""),
                value=val_item.get("value", 0),
                description=val_item.get("description", ""),
            )
            for val_item in enum_item.get("values", [])
        ]
        enum_obj = EnumIR(
            name=enum_item.get("name", ""),
            description=enum_item.get("description", ""),
            values=enum_values,
        )
        enums_ir.append(enum_obj)
        enum_map[enum_obj.name] = enum_obj

    # 5. Parse Structs
    structs_ir: list[StructIR] = []
    struct_map: dict[str, StructIR] = {}
    for struct_item in raw_structs:
        struct_name = struct_item.get("name", "")
        fields_ir: list[FieldIR] = []

        for field_item in struct_item.get("fields", []):
            field_name = field_item.get("name", "")
            raw_field_type = field_item.get("type", "").strip()

            is_array = "[" in raw_field_type
            array_size = None
            array_base_type = None

            if is_array:
                array_base_type = raw_field_type.split("[")[0].strip()
                size_str = raw_field_type.split("[")[1].split("]")[0].strip()
                try:
                    array_size = int(size_str)
                except ValueError:
                    array_size = None
                solix_type = map_c_type_to_solix(raw_field_type)
                rust_base = map_c_type_to_rust(array_base_type)
                rust_type = f"[{rust_base}; {size_str}]"
                array_elem_rust_type = rust_base
                is_pointer_field = False
            else:
                base_fld, is_const_fld, fld_ptr_depth = decompose_c_type(raw_field_type)
                resolved_fld = resolve_alias_recursively(base_fld, alias_map)
                is_pointer_field = (
                    fld_ptr_depth > 0
                    or "*" in raw_field_type
                    or "*" in resolved_fld
                )
                if is_pointer_field:
                    solix_type = "uint64"
                else:
                    solix_type = map_c_type_to_solix(raw_field_type)
                rust_type = map_c_type_to_rust(raw_field_type)
                array_elem_rust_type = None

            fields_ir.append(
                FieldIR(
                    name=field_name,
                    escaped_solix_name=sanitize_solix_ident(field_name),
                    escaped_rust_name=escape_rust_ident(field_name),
                    raw_type=raw_field_type,
                    solix_type=solix_type,
                    rust_type=rust_type,
                    is_array=is_array,
                    is_pointer=is_pointer_field,
                    array_size=array_size,
                    array_base_type=array_base_type,
                    array_elem_rust_type=array_elem_rust_type,
                    description=field_item.get("description", ""),
                )
            )

        struct_obj = StructIR(
            name=struct_name,
            snake_name=to_snake_case(struct_name),
            description=struct_item.get("description", ""),
            fields=fields_ir,
            presets=preset_literals_map.get(struct_name, []),
        )
        structs_ir.append(struct_obj)
        struct_map[struct_name] = struct_obj

    # 6. Parse Aliases
    aliases_ir: list[AliasIR] = []
    for alias_item in raw_aliases:
        alias_name = alias_item.get("name", "")
        target_type = alias_item.get("type", "")
        resolved_target = resolve_alias_recursively(target_type, alias_map)
        is_pointer_alias = "*" in resolved_target

        aliases_ir.append(
            AliasIR(
                name=alias_name,
                snake_name=to_snake_case(alias_name),
                target_type=target_type,
                resolved_target=resolved_target,
                is_pointer_alias=is_pointer_alias,
                description=alias_item.get("description", ""),
            )
        )

    # 7. Parse Callbacks
    callbacks_ir: list[CallbackIR] = []
    callback_map: dict[str, CallbackIR] = {}
    for callback_item in raw_callbacks:
        cb_name = callback_item.get("name", "")
        cb_params_ir: list[ParamIR] = []

        for param_item in callback_item.get("params", []):
            param_name = param_item.get("name", "arg")
            raw_param_type = param_item.get("type", "void")
            cb_params_ir.append(
                ParamIR(
                    name=param_name,
                    escaped_solix_name=sanitize_solix_ident(param_name),
                    escaped_rust_name=escape_rust_ident(to_snake_case(param_name)),
                    raw_type=raw_param_type,
                    solix_type=map_c_type_to_solix(raw_param_type),
                    rust_type=map_c_type_to_rust(raw_param_type),
                    is_variadic=(raw_param_type == "..."),
                    is_string=raw_param_type in ("const char *", "char *"),
                    is_callback=False,
                    is_struct_val=False,
                    is_mutable_struct_ptr=False,
                    is_struct_array=False,
                    is_primitive_array=False,
                    is_mutable_primitive_array=False,
                    base_struct_name=None,
                    base_primitive_name=None,
                )
            )

        raw_ret = callback_item.get("returnType", "void")
        cb_obj = CallbackIR(
            name=cb_name,
            return_type=raw_ret,
            rust_return_type=map_c_type_to_rust(raw_ret),
            params=cb_params_ir,
            description=callback_item.get("description", ""),
        )
        callbacks_ir.append(cb_obj)
        callback_map[cb_name] = cb_obj

    # 8. Parse Functions
    functions_ir: list[FunctionIR] = []
    unload_pointer_functions = {
        "UnloadRandomSequence",
        "UnloadFileData",
        "UnloadImageColors",
        "UnloadImagePalette",
        "UnloadFontData",
        "UnloadCodepoints",
        "UnloadTextLines",
        "UnloadModelAnimations",
        "UnloadWaveSamples",
    }
    for func_item in raw_functions:
        func_name = func_item.get("name", "")
        raw_ret = func_item.get("returnType", "void")
        raw_params = func_item.get("params", [])
        is_variadic = any(param.get("type") == "..." for param in raw_params)

        params_ir: list[ParamIR] = []
        for param_item in raw_params:
            param_name = param_item.get("name", "argument")
            raw_param_type = param_item.get("type", "void").strip()

            base_param, is_const_param, ptr_depth = decompose_c_type(raw_param_type)
            resolved_param = resolve_alias_recursively(base_param, alias_map)

            is_unload_pointer = (
                (func_name in unload_pointer_functions or (func_name.startswith("Unload") and raw_param_type not in ("char *", "const char *")))
                and (ptr_depth > 0 or "*" in raw_param_type or "*" in resolved_param)
            )

            if is_unload_pointer:
                concrete_struct = None
                is_struct_val = False
                is_mutable_struct_ptr = False
                is_struct_array = False
                is_primitive_arr = False
                is_mut_primitive_arr = False
                is_callback = False
                is_str = False
                solix_param_type = "uint64"
                rust_param_type = map_c_type_to_rust(raw_param_type)
            else:
                concrete_struct = (
                    base_param if base_param in raw_struct_names
                    else (resolved_param if resolved_param in raw_struct_names else None)
                )

                is_struct_val = (ptr_depth == 0 and concrete_struct is not None)
                is_mutable_struct_ptr = (
                    ptr_depth == 1 and not is_const_param and concrete_struct is not None
                )
                is_struct_array = (
                    ptr_depth == 1 and is_const_param and concrete_struct is not None
                )

                primitive_names = (
                    "int", "i32", "unsigned int", "uint", "u32", "float", "f32", "double", "f64",
                    "char", "int8", "unsigned char", "uint8", "uchar", "short", "int16", "unsigned short", "uint16", "ushort",
                    "long", "long long", "int64", "unsigned long", "unsigned long long", "uint64",
                )
                is_primitive_arr = (ptr_depth == 1 and base_param in primitive_names)
                is_mut_primitive_arr = (is_primitive_arr and not is_const_param)

                is_callback = (
                    base_param in raw_callback_names or resolved_param in raw_callback_names
                )
                is_str = raw_param_type in ("const char *", "char *")
                solix_param_type = map_c_type_to_solix(raw_param_type)
                rust_param_type = map_c_type_to_rust(raw_param_type)

            params_ir.append(
                ParamIR(
                    name=param_name,
                    escaped_solix_name=sanitize_solix_ident(param_name),
                    escaped_rust_name=escape_rust_ident(to_snake_case(param_name)),
                    raw_type=raw_param_type,
                    solix_type=solix_param_type,
                    rust_type=rust_param_type,
                    is_variadic=(raw_param_type == "..."),
                    is_string=is_str,
                    is_callback=is_callback,
                    is_struct_val=is_struct_val,
                    is_mutable_struct_ptr=is_mutable_struct_ptr,
                    is_struct_array=is_struct_array,
                    is_primitive_array=is_primitive_arr,
                    is_mutable_primitive_array=is_mut_primitive_arr,
                    base_struct_name=concrete_struct,
                    base_primitive_name=base_param if is_primitive_arr else None,
                )
            )

        base_ret, is_const_ret, ret_ptr_depth = decompose_c_type(raw_ret)
        resolved_ret = resolve_alias_recursively(base_ret, alias_map)
        is_ret_pointer = (ret_ptr_depth > 0 or "*" in raw_ret or "*" in resolved_ret)
        is_ret_string = raw_ret.strip() in ("const char *", "char *")

        if is_ret_pointer and not is_ret_string:
            solix_ret_type = "uint64"
        else:
            solix_ret_type = map_c_type_to_solix(raw_ret)

        functions_ir.append(
            FunctionIR(
                name=func_name,
                snake_name=to_snake_case(func_name),
                return_type=raw_ret,
                solix_return_type=solix_ret_type,
                rust_return_type=map_c_type_to_rust(raw_ret),
                params=params_ir,
                is_variadic=is_variadic,
                description=func_item.get("description", ""),
            )
        )

    return ApiIR(
        defines=defines_ir,
        enums=enums_ir,
        structs=structs_ir,
        aliases=aliases_ir,
        callbacks=callbacks_ir,
        functions=functions_ir,
        known_constants=known_constants,
        struct_map=struct_map,
        alias_map=alias_map,
        enum_map=enum_map,
        callback_map=callback_map,
    )


# ============================================================================
# Pass 2: Solix Declarations Emitter (--out-slx)
# ============================================================================

def emit_solix_declarations(ir: ApiIR, package_name: str, class_name: str) -> str:
    """Emits complete 1-to-1 Solix API declarations from intermediate representation lists."""
    lines: list[str] = [
        "// Auto-generated 1-to-1 C API bindings for the Solix Programming Language",
        "// Generated via unified Two-Pass IR generator with zero hardcoded types.",
        "",
        f"package {package_name};",
        "",
        "// ===========================================================================",
        "// Global Constants & Evaluated Defines",
        "// ===========================================================================",
    ]

    for define_obj in ir.defines:
        if define_obj.item_type == "INT":
            lines.append(f"const int32 {define_obj.name} = {define_obj.evaluated_value};")
        elif define_obj.item_type == "FLOAT":
            lines.append(f"const float32 {define_obj.name} = (float32){define_obj.evaluated_value};")
        elif define_obj.item_type == "STRING":
            lines.append(f'const string {define_obj.name} = "{define_obj.raw_value}";')
        elif define_obj.item_type == "FLOAT_MATH" and define_obj.evaluated_value is not None:
            lines.append(f"const float32 {define_obj.name} = (float32){define_obj.evaluated_value};")

    lines.append("")
    lines.append("// ===========================================================================")
    lines.append("// Enums & Bitflags")
    lines.append("// ===========================================================================")

    for enum_obj in ir.enums:
        if enum_obj.description:
            lines.append(f"/// {enum_obj.description}")
        lines.append(f"public enum {enum_obj.name} {{")
        total_vals = len(enum_obj.values)
        for val_idx, val_item in enumerate(enum_obj.values):
            comma = "," if val_idx + 1 < total_vals else ""
            if val_item.description:
                lines.append(f"    /// {val_item.description}")
            lines.append(f"    {val_item.name} = {val_item.value}{comma}")
        lines.append("}")
        lines.append("")

    lines.append("// ===========================================================================")
    lines.append("// Struct Classes")
    lines.append("// ===========================================================================")

    for struct_obj in ir.structs:
        if struct_obj.description:
            lines.append(f"/// {struct_obj.description}")
        lines.append(f"public class {struct_obj.name} {{")

        for field_obj in struct_obj.fields:
            if field_obj.description:
                lines.append(f"    /// {field_obj.description}")
            lines.append(f"    public {field_obj.solix_type} {field_obj.escaped_solix_name};")
        lines.append("")

        # Default zero constructor
        lines.append(f"    public {struct_obj.name}() {{")
        for field_obj in struct_obj.fields:
            field_name = field_obj.escaped_solix_name
            if field_obj.is_array:
                if field_obj.array_base_type in ("char", "const char"):
                    lines.append(f'        this.{field_name} = "";')
                else:
                    elem_type = field_obj.solix_type[:-2]
                    size_str = str(field_obj.array_size) if field_obj.array_size else "0"
                    lines.append(f"        this.{field_name} = new {elem_type}[{size_str}];")
            else:
                solix_type = field_obj.solix_type
                if solix_type in ("int32", "int16", "int8", "int64"):
                    lines.append(f"        this.{field_name} = 0;")
                elif solix_type in ("uint32", "uint16", "uint8", "uint64"):
                    lines.append(f"        this.{field_name} = ({solix_type})0;")
                elif solix_type == "float32":
                    lines.append(f"        this.{field_name} = (float32)0.0;")
                elif solix_type == "float64":
                    lines.append(f"        this.{field_name} = 0.0;")
                elif solix_type == "bool":
                    lines.append(f"        this.{field_name} = false;")
                elif solix_type == "string":
                    lines.append(f'        this.{field_name} = "";')
                elif solix_type.endswith("[]"):
                    base_elem = solix_type[:-2]
                    lines.append(f"        this.{field_name} = new {base_elem}[0];")
                else:
                    lines.append(f"        this.{field_name} = new {solix_type}();")
        lines.append("    }")

        # Parameterized constructor for small structs (<= 8 fields, no fixed arrays)
        if 0 < len(struct_obj.fields) <= 8 and not any(fld.is_array for fld in struct_obj.fields):
            params_str_list = []
            assign_lines = []
            for field_obj in struct_obj.fields:
                param_var = f"init_{field_obj.escaped_solix_name}"
                params_str_list.append(f"{field_obj.solix_type} {param_var}")
                assign_lines.append(f"        this.{field_obj.escaped_solix_name} = {param_var};")

            lines.append("")
            lines.append(f"    public {struct_obj.name}({', '.join(params_str_list)}) {{")
            for assign_line in assign_lines:
                lines.append(assign_line)
            lines.append("    }")

        # Preset static constants
        if struct_obj.presets:
            lines.append("")
            lines.append("    // --- Preset Constants ---")
            for preset_item in struct_obj.presets:
                typed_args = []
                for comp_idx, raw_arg in enumerate(preset_item.raw_args):
                    if comp_idx < len(struct_obj.fields):
                        field_solix_type = struct_obj.fields[comp_idx].solix_type
                        if field_solix_type in (
                            "uint8", "uint16", "uint32", "uint64",
                            "int8", "int16", "int32", "int64", "float32",
                        ):
                            typed_args.append(f"({field_solix_type}){raw_arg}")
                        else:
                            typed_args.append(raw_arg)
                    else:
                        typed_args.append(raw_arg)

                if preset_item.description:
                    lines.append(f"    /// {preset_item.description}")
                lines.append(
                    f"    public static {struct_obj.name} {preset_item.name} = new {struct_obj.name}({', '.join(typed_args)});"
                )

        lines.append("}")
        lines.append("")

    lines.append("// ===========================================================================")
    lines.append("// Type Aliases (Extended Subclasses)")
    lines.append("// ===========================================================================")

    for alias_obj in ir.aliases:
        if not alias_obj.is_pointer_alias and alias_obj.resolved_target in ir.struct_map:
            if alias_obj.description:
                lines.append(f"/// {alias_obj.description}")
            lines.append(f"public class {alias_obj.name} extends {alias_obj.resolved_target} {{")
            lines.append(f"    public {alias_obj.name}() {{}}")
            lines.append("}")
            lines.append("")

    lines.append("// ===========================================================================")
    lines.append(f"// Primary Engine API (Class {class_name} & Top-Level Functions)")
    lines.append("// ===========================================================================")
    lines.append(f"public class {class_name} {{")

    # Class-level constants
    for define_obj in ir.defines:
        if define_obj.item_type == "INT":
            lines.append(f"    public static const int32 {define_obj.name} = {define_obj.evaluated_value};")
        elif define_obj.item_type == "FLOAT":
            lines.append(f"    public static const float32 {define_obj.name} = (float32){define_obj.evaluated_value};")
        elif define_obj.item_type == "STRING":
            lines.append(f'    public static const string {define_obj.name} = "{define_obj.raw_value}";')
        elif define_obj.item_type == "FLOAT_MATH" and define_obj.evaluated_value is not None:
            lines.append(f"    public static const float32 {define_obj.name} = (float32){define_obj.evaluated_value};")
    lines.append("")

    for func_obj in ir.functions:
        param_decls = [
            f"{param.solix_type} {param.escaped_solix_name}"
            for param in func_obj.params
            if not param.is_variadic
        ]
        param_list_str = ", ".join(param_decls)

        if func_obj.description:
            lines.append(f"    /// {func_obj.description}")
        lines.append(f"    public static native {func_obj.solix_return_type} {func_obj.name}({param_list_str});")
        if func_obj.snake_name != func_obj.name:
            lines.append(f"    public static native {func_obj.solix_return_type} {func_obj.snake_name}({param_list_str});")

    lines.append("}")
    lines.append("")

    # Direct top-level functions
    lines.append("// --- Direct Top-Level Function Exports ---")
    for func_obj in ir.functions:
        param_decls = [
            f"{param.solix_type} {param.escaped_solix_name}"
            for param in func_obj.params
            if not param.is_variadic
        ]
        param_list_str = ", ".join(param_decls)

        lines.append(f"public native {func_obj.solix_return_type} {func_obj.name}({param_list_str});")
        if func_obj.snake_name != func_obj.name:
            lines.append(f"public native {func_obj.solix_return_type} {func_obj.snake_name}({param_list_str});")

    return "\n".join(lines) + "\n"


# ============================================================================
# Pass 2: Rust C-ABI Companion Bridge Emitter (--out-rs)
# ============================================================================

def emit_rust_bridge(ir: ApiIR, package_name: str, class_name: str) -> str:
    """Emits complete 1-to-1 Rust FFI companion bridge from intermediate representation lists."""
    lines: list[str] = [
        "//! Auto-generated 1-to-1 Rust native bridge extension for Solix C-API bindings.",
        "//! Generated via unified Two-Pass IR generator with zero hardcoded engine types.",
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
        "// ===========================================================================",
        "// Callback Function Signatures",
        "// ===========================================================================",
    ]

    for cb_obj in ir.callbacks:
        param_decls = [f"{param.escaped_rust_name}: {param.rust_type}" for param in cb_obj.params]
        ret_suffix = "" if cb_obj.rust_return_type == "()" else f" -> {cb_obj.rust_return_type}"
        lines.append(
            f"pub type {cb_obj.name} = Option<unsafe extern \"C\" fn({', '.join(param_decls)}){ret_suffix}>;"
        )

    lines.append("")
    lines.append("// ===========================================================================")
    lines.append("// C ABI Struct Definitions & Aliases")
    lines.append("// ===========================================================================")

    for alias_obj in ir.aliases:
        if alias_obj.is_pointer_alias:
            alias_base, _, alias_depth = decompose_c_type(alias_obj.resolved_target)
            if alias_base in ir.struct_map:
                mut_stars = "*mut " * alias_depth
                lines.append(f"pub type C_{alias_obj.name} = {mut_stars}C_{alias_base};")
            else:
                lines.append(f"pub type C_{alias_obj.name} = *mut c_void;")
        elif alias_obj.resolved_target in ir.struct_map:
            lines.append(f"pub type C_{alias_obj.name} = C_{alias_obj.resolved_target};")
        else:
            lines.append(f"pub type C_{alias_obj.name} = *mut c_void;")

    lines.append("")

    for struct_obj in ir.structs:
        lines.append("#[repr(C)]")
        lines.append("#[derive(Clone, Copy)]")
        lines.append(f"pub struct C_{struct_obj.name} {{")
        for field_obj in struct_obj.fields:
            lines.append(f"    pub {field_obj.escaped_rust_name}: {field_obj.rust_type},")
        lines.append("}")
        lines.append("")
        lines.append(f"impl Default for C_{struct_obj.name} {{")
        lines.append("    fn default() -> Self {")
        lines.append("        unsafe { std::mem::zeroed() }")
        lines.append("    }")
        lines.append("}")
        lines.append("")

    lines.append("// ===========================================================================", )
    lines.append("// Struct Unpack, Pack & Writeback Helpers")
    lines.append("// ===========================================================================")

    for struct_obj in ir.structs:
        snake_name = struct_obj.snake_name
        field_count = len(struct_obj.fields)

        # 1. Writeback
        lines.append(f"unsafe fn writeback_{snake_name}(vm_context: *mut c_void, addr: u32, val: &C_{struct_obj.name}) {{")
        lines.append("    if addr == 0 { return; }")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")

        for field_idx, field_obj in enumerate(struct_obj.fields):
            field_name = field_obj.escaped_rust_name
            raw_type = field_obj.raw_type

            if raw_type in ("bool", "_Bool"):
                lines.append(f"    ((*context_ptr).set_field_i32)(vm_context, addr, {field_idx}, if val.{field_name} {{ 1 }} else {{ 0 }});")
            elif raw_type in ("unsigned char", "uint8", "uchar", "char", "int8", "short", "int16", "unsigned short", "uint16", "ushort", "int", "i32", "unsigned int", "uint", "u32"):
                lines.append(f"    ((*context_ptr).set_field_i32)(vm_context, addr, {field_idx}, val.{field_name} as i32);")
            elif raw_type in ("float", "f32"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, (val.{field_name} as f64).to_bits());")
            elif raw_type in ("double", "f64"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, val.{field_name}.to_bits());")
            elif raw_type in ("long", "long long", "int64", "unsigned long", "unsigned long long", "uint64"):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, val.{field_name} as u64);")
            elif field_obj.is_array:
                base_elem = field_obj.array_base_type or ""
                size_str = str(field_obj.array_size or 0)
                if base_elem in ("char", "const char"):
                    lines.append(f"    let mut str_len_{field_idx} = 0usize;")
                    lines.append(f"    while str_len_{field_idx} < {size_str} && val.{field_name}[str_len_{field_idx}] != 0 {{")
                    lines.append(f"        str_len_{field_idx} += 1;")
                    lines.append("    }")
                    lines.append(f"    let str_addr_{field_idx} = ((*context_ptr).alloc_string)(vm_context, val.{field_name}.as_ptr() as *const u8, str_len_{field_idx} as u32);")
                    lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, str_addr_{field_idx} as u64);")
                elif base_elem in ir.struct_map or resolve_alias_recursively(base_elem, ir.alias_map) in ir.struct_map:
                    resolved_struct = (
                        base_elem if base_elem in ir.struct_map
                        else resolve_alias_recursively(base_elem, ir.alias_map)
                    )
                    nested_snake = to_snake_case(resolved_struct)
                    lines.append(f"    let arr_addr_{field_idx} = ((*context_ptr).alloc_array)(vm_context, {size_str}, 0);")
                    lines.append(f"    for elem_idx in 0..{size_str}u32 {{")
                    lines.append(f"        let child_addr = pack_{nested_snake}(vm_context, val.{field_name}[elem_idx as usize]);")
                    lines.append(f"        ((*context_ptr).array_set_u64)(vm_context, arr_addr_{field_idx}, elem_idx, child_addr as u64);")
                    lines.append("    }")
                    lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, arr_addr_{field_idx} as u64);")
                else:
                    lines.append(f"    let arr_addr_{field_idx} = ((*context_ptr).alloc_array)(vm_context, {size_str}, 0);")
                    lines.append(f"    for elem_idx in 0..{size_str}u32 {{")
                    if base_elem in ("float", "f32"):
                        lines.append(f"        ((*context_ptr).array_set_u64)(vm_context, arr_addr_{field_idx}, elem_idx, (val.{field_name}[elem_idx as usize] as f64).to_bits());")
                    elif base_elem in ("double", "f64"):
                        lines.append(f"        ((*context_ptr).array_set_u64)(vm_context, arr_addr_{field_idx}, elem_idx, val.{field_name}[elem_idx as usize].to_bits());")
                    else:
                        lines.append(f"        ((*context_ptr).array_set_u64)(vm_context, arr_addr_{field_idx}, elem_idx, val.{field_name}[elem_idx as usize] as u64);")
                    lines.append("    }")
                    lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, arr_addr_{field_idx} as u64);")
            elif field_obj.is_pointer:
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, val.{field_name} as usize as u64);")
            elif raw_type in ir.struct_map or resolve_alias_recursively(raw_type, ir.alias_map) in ir.struct_map:
                resolved_struct = (
                    raw_type if raw_type in ir.struct_map
                    else resolve_alias_recursively(raw_type, ir.alias_map)
                )
                nested_snake = to_snake_case(resolved_struct)
                lines.append(f"    let child_{field_idx} = pack_{nested_snake}(vm_context, val.{field_name});")
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, child_{field_idx} as u64);")
            elif raw_type.endswith("*") or raw_type.startswith("void") or "*" in resolve_alias_recursively(raw_type, ir.alias_map):
                lines.append(f"    ((*context_ptr).write_field_u64)(vm_context, addr, {field_idx}, val.{field_name} as usize as u64);")

        lines.append("}")
        lines.append("")

        # 2. Pack
        lines.append(f"unsafe fn pack_{snake_name}(vm_context: *mut c_void, val: C_{struct_obj.name}) -> u32 {{")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
        lines.append(f'    let class_name_str = c"{struct_obj.name}";')
        lines.append("    let class_vtable_id = ((*context_ptr).get_class_vtable_id)(vm_context, class_name_str.as_ptr());")
        lines.append(f"    let addr = ((*context_ptr).alloc_object)(vm_context, class_vtable_id, {field_count});")
        lines.append("    if addr > 0 {")
        lines.append(f"        writeback_{snake_name}(vm_context, addr, &val);")
        lines.append("    }")
        lines.append("    addr")
        lines.append("}")
        lines.append("")

        # 3. Unpack
        lines.append(f"unsafe fn unpack_{snake_name}(vm_context: *mut c_void, addr: u32) -> C_{struct_obj.name} {{")
        lines.append(f"    if addr == 0 {{ return C_{struct_obj.name}::default(); }}")
        lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
        lines.append(f"    let mut result = C_{struct_obj.name}::default();")

        for field_idx, field_obj in enumerate(struct_obj.fields):
            field_name = field_obj.escaped_rust_name
            raw_type = field_obj.raw_type

            if raw_type in ("bool", "_Bool"):
                lines.append(f"    result.{field_name} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_idx}) != 0;")
            elif raw_type in ("unsigned char", "uint8", "uchar"):
                lines.append(f"    result.{field_name} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_idx}) as u8;")
            elif raw_type in ("char", "int8"):
                lines.append(f"    result.{field_name} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_idx}) as c_char;")
            elif raw_type in ("short", "int16"):
                lines.append(f"    result.{field_name} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_idx}) as c_short;")
            elif raw_type in ("unsigned short", "uint16", "ushort"):
                lines.append(f"    result.{field_name} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_idx}) as c_ushort;")
            elif raw_type in ("int", "i32"):
                lines.append(f"    result.{field_name} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_idx}) as c_int;")
            elif raw_type in ("unsigned int", "uint", "u32"):
                lines.append(f"    result.{field_name} = ((*context_ptr).get_field_i32)(vm_context, addr, {field_idx}) as c_uint;")
            elif raw_type in ("float", "f32"):
                lines.append(f"    result.{field_name} = f64::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, {field_idx})) as f32;")
            elif raw_type in ("double", "f64"):
                lines.append(f"    result.{field_name} = f64::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}));")
            elif raw_type in ("long", "long long", "int64"):
                lines.append(f"    result.{field_name} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}) as c_longlong;")
            elif raw_type in ("unsigned long", "unsigned long long", "uint64"):
                lines.append(f"    result.{field_name} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}) as c_ulonglong;")
            elif field_obj.is_array:
                base_elem = field_obj.array_base_type or ""
                size_str = str(field_obj.array_size or 0)
                if base_elem in ("char", "const char"):
                    lines.append(f"    let str_addr_{field_idx} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}) as u32;")
                    lines.append(f"    if str_addr_{field_idx} > 0 {{")
                    lines.append(f"        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_{field_idx}, result.{field_name}.as_mut_ptr() as *mut u8, ({size_str} - 1) as u32) as usize;")
                    lines.append(f"        result.{field_name}[read_len] = 0;")
                    lines.append("    }")
                elif base_elem in ir.struct_map or resolve_alias_recursively(base_elem, ir.alias_map) in ir.struct_map:
                    resolved_struct = (
                        base_elem if base_elem in ir.struct_map
                        else resolve_alias_recursively(base_elem, ir.alias_map)
                    )
                    nested_snake = to_snake_case(resolved_struct)
                    lines.append(f"    let arr_addr_{field_idx} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}) as u32;")
                    lines.append(f"    if arr_addr_{field_idx} > 0 {{")
                    lines.append(f"        let arr_len = ((*context_ptr).array_len)(vm_context, arr_addr_{field_idx}).min({size_str});")
                    lines.append("        for elem_idx in 0..arr_len {")
                    lines.append(f"            let child_addr = ((*context_ptr).array_get_u64)(vm_context, arr_addr_{field_idx}, elem_idx) as u32;")
                    lines.append(f"            result.{field_name}[elem_idx as usize] = unpack_{nested_snake}(vm_context, child_addr);")
                    lines.append("        }")
                    lines.append("    }")
                else:
                    lines.append(f"    let arr_addr_{field_idx} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}) as u32;")
                    lines.append(f"    if arr_addr_{field_idx} > 0 {{")
                    lines.append(f"        let arr_len = ((*context_ptr).array_len)(vm_context, arr_addr_{field_idx}).min({size_str});")
                    lines.append("        for elem_idx in 0..arr_len {")
                    lines.append(f"            let raw_val = ((*context_ptr).array_get_u64)(vm_context, arr_addr_{field_idx}, elem_idx);")
                    if base_elem in ("float", "f32"):
                        lines.append(f"            result.{field_name}[elem_idx as usize] = f64::from_bits(raw_val) as f32;")
                    elif base_elem in ("double", "f64"):
                        lines.append(f"            result.{field_name}[elem_idx as usize] = f64::from_bits(raw_val);")
                    else:
                        elem_rust_type = field_obj.array_elem_rust_type or "u64"
                        lines.append(f"            result.{field_name}[elem_idx as usize] = raw_val as {elem_rust_type};")
                    lines.append("        }")
                    lines.append("    }")
            elif field_obj.is_pointer:
                lines.append(f"    result.{field_name} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}) as usize as {field_obj.rust_type};")
            elif raw_type in ir.struct_map or resolve_alias_recursively(raw_type, ir.alias_map) in ir.struct_map:
                resolved_struct = (
                    raw_type if raw_type in ir.struct_map
                    else resolve_alias_recursively(raw_type, ir.alias_map)
                )
                nested_snake = to_snake_case(resolved_struct)
                lines.append(f"    result.{field_name} = unpack_{nested_snake}(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}) as u32);")
            elif raw_type.endswith("*") or raw_type.startswith("void") or "*" in resolve_alias_recursively(raw_type, ir.alias_map):
                lines.append(f"    result.{field_name} = ((*context_ptr).read_field_u64)(vm_context, addr, {field_idx}) as usize as {field_obj.rust_type};")

        lines.append("    result")
        lines.append("}")
        lines.append("")

    # Alias forwarders
    for alias_obj in ir.aliases:
        if not alias_obj.is_pointer_alias and alias_obj.resolved_target in ir.struct_map:
            snake_alias = alias_obj.snake_name
            snake_target = to_snake_case(alias_obj.resolved_target)
            lines.append(f"unsafe fn unpack_{snake_alias}(vm_context: *mut c_void, addr: u32) -> C_{alias_obj.name} {{ unpack_{snake_target}(vm_context, addr) }}")
            lines.append(f"unsafe fn pack_{snake_alias}(vm_context: *mut c_void, val: C_{alias_obj.name}) -> u32 {{ pack_{snake_target}(vm_context, val) }}")
            lines.append(f"unsafe fn writeback_{snake_alias}(vm_context: *mut c_void, addr: u32, val: &C_{alias_obj.name}) {{ writeback_{snake_target}(vm_context, addr, val); }}")

    lines.append("")
    lines.append("// ===========================================================================")
    lines.append("// Upstream Extern \"C\" Declarations")
    lines.append("// ===========================================================================")
    lines.append("unsafe extern \"C\" {")

    for func_obj in ir.functions:
        param_decls = []
        for param_idx, param in enumerate(func_obj.params):
            if param.is_variadic:
                param_decls.append("...")
            else:
                param_decls.append(f"arg_{param_idx}: {param.rust_type}")
        ret_suffix = "" if func_obj.rust_return_type == "()" else f" -> {func_obj.rust_return_type}"
        lines.append(f"    pub fn {func_obj.name}({', '.join(param_decls)}){ret_suffix};")

    lines.append("}")
    lines.append("")
    lines.append("// ===========================================================================")
    lines.append("// Solix C-ABI Native Wrapper Functions")
    lines.append("// ===========================================================================")

    registration_entries: list[tuple[str, str]] = []

    for func_obj in ir.functions:
        wrapper_name = f"{class_name}_{func_obj.name}"
        raw_ret = func_obj.return_type

        lines.append("#[unsafe(no_mangle)]")
        lines.append(f"pub unsafe extern \"C\" fn {wrapper_name}(")
        lines.append("    vm_context: *mut c_void,")
        lines.append("    args: *const SolixCValue,")
        lines.append("    arg_count: u32,")
        lines.append("    out_error: *mut SolixCValue,")
        lines.append(") -> SolixCValue {")

        call_args: list[str] = []
        pre_call_lines: list[str] = []
        post_call_lines: list[str] = []

        non_variadic_params = [param for param in func_obj.params if not param.is_variadic]

        for param_idx, param in enumerate(non_variadic_params):
            arg_expr = f"(*args.add({param_idx}))"
            raw_param = param.raw_type

            if raw_param in ("int", "i32"):
                call_args.append(f"{arg_expr}.payload as c_int")
            elif raw_param in ("unsigned int", "uint", "u32"):
                call_args.append(f"{arg_expr}.payload as c_uint")
            elif raw_param in ("short", "int16"):
                call_args.append(f"{arg_expr}.payload as c_short")
            elif raw_param in ("unsigned short", "uint16", "ushort"):
                call_args.append(f"{arg_expr}.payload as c_ushort")
            elif raw_param in ("char", "int8"):
                call_args.append(f"{arg_expr}.payload as c_char")
            elif raw_param in ("unsigned char", "uint8", "uchar"):
                call_args.append(f"{arg_expr}.payload as u8")
            elif raw_param in ("long", "long long", "int64"):
                call_args.append(f"{arg_expr}.payload as c_longlong")
            elif raw_param in ("unsigned long", "unsigned long long", "uint64"):
                call_args.append(f"{arg_expr}.payload as c_ulonglong")
            elif raw_param in ("float", "f32"):
                call_args.append(f"f64::from_bits({arg_expr}.payload) as f32")
            elif raw_param in ("double", "f64"):
                call_args.append(f"f64::from_bits({arg_expr}.payload)")
            elif raw_param in ("bool", "_Bool"):
                call_args.append(f"{arg_expr}.payload != 0")
            elif param.is_string:
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
            elif param.is_callback:
                call_args.append(
                    f"if {arg_expr}.payload == 0 {{ None }} else {{ Some(std::mem::transmute({arg_expr}.payload as usize)) }}"
                )
            elif param.is_struct_val and param.base_struct_name:
                struct_snake = to_snake_case(param.base_struct_name)
                call_args.append(f"unpack_{struct_snake}(vm_context, {arg_expr}.payload as u32)")
            elif param.is_mutable_struct_ptr and param.base_struct_name:
                struct_snake = to_snake_case(param.base_struct_name)
                var_name = f"{struct_snake}_mut_{param_idx}"
                addr_name = f"{struct_snake}_addr_{param_idx}"
                pre_call_lines.append(f"    let {addr_name} = {arg_expr}.payload as u32;")
                pre_call_lines.append(f"    let mut {var_name} = unpack_{struct_snake}(vm_context, {addr_name});")
                call_args.append(f"&mut {var_name}")
                post_call_lines.append(f"    writeback_{struct_snake}(vm_context, {addr_name}, &{var_name});")
            elif param.is_struct_array and param.base_struct_name:
                concrete_st = param.base_struct_name
                struct_snake = to_snake_case(concrete_st)
                vec_name = f"struct_vec_{param_idx}"
                arr_name = f"arr_addr_{param_idx}"
                pre_call_lines.append(f"    let {arr_name} = {arg_expr}.payload as u32;")
                pre_call_lines.append(f"    let mut {vec_name}: Vec<C_{concrete_st}> = Vec::new();")
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
            elif param.is_primitive_array and param.base_primitive_name:
                base_prim = param.base_primitive_name
                rust_elem = param.rust_type.replace("*const ", "").replace("*mut ", "")
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
                if base_prim in ("float", "f32"):
                    pre_call_lines.append(f"            {vec_name}.push(f64::from_bits(raw_val) as f32);")
                elif base_prim in ("double", "f64"):
                    pre_call_lines.append(f"            {vec_name}.push(f64::from_bits(raw_val));")
                else:
                    pre_call_lines.append(f"            {vec_name}.push(raw_val as {rust_elem});")
                pre_call_lines.append("        }")
                pre_call_lines.append("    }")
                call_args.append(f"if {arr_name} > 0 {{ {vec_name}.as_mut_ptr() }} else {{ std::ptr::null_mut() }}")

                if param.is_mutable_primitive_array:
                    post_call_lines.append(f"    if {arr_name} > 0 {{")
                    post_call_lines.append("        let context_ptr = vm_context as *mut SolixVmContext;")
                    post_call_lines.append(f"        let total_count = ((*context_ptr).array_len)(vm_context, {arr_name});")
                    post_call_lines.append("        for elem_idx in 0..total_count {")
                    post_call_lines.append(f"            let updated_elem = {vec_name}[elem_idx as usize];")
                    if base_prim in ("float", "f32"):
                        post_call_lines.append(f"            ((*context_ptr).array_set_u64)(vm_context, {arr_name}, elem_idx, (updated_elem as f64).to_bits());")
                    elif base_prim in ("double", "f64"):
                        post_call_lines.append(f"            ((*context_ptr).array_set_u64)(vm_context, {arr_name}, elem_idx, updated_elem.to_bits());")
                    else:
                        post_call_lines.append(f"            ((*context_ptr).array_set_u64)(vm_context, {arr_name}, elem_idx, updated_elem as u64);")
                    post_call_lines.append("        }")
                    post_call_lines.append("    }")
            else:
                call_args.append(f"{arg_expr}.payload as usize as {param.rust_type}")

        for pre_line in pre_call_lines:
            lines.append(pre_line)

        if func_obj.is_variadic:
            last_arg = call_args.pop() if call_args else 'c"".as_ptr()'
            if call_args:
                call_invocation = f"{func_obj.name}({', '.join(call_args)}, c\"%s\".as_ptr(), {last_arg})"
            else:
                call_invocation = f"{func_obj.name}(c\"%s\".as_ptr(), {last_arg})"
        else:
            call_invocation = f"{func_obj.name}({', '.join(call_args)})"

        base_ret, is_const_ret, ret_ptr_depth = decompose_c_type(raw_ret)
        resolved_ret = resolve_alias_recursively(base_ret, ir.alias_map)

        if raw_ret == "void":
            lines.append(f"    {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append("    SolixCValue::null()")
        elif raw_ret in ("bool", "_Bool"):
            lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append("    SolixCValue::bool_val(call_result)")
        elif raw_ret in ("int", "i32", "unsigned int", "uint", "u32", "short", "int16", "unsigned short", "uint16", "ushort", "char", "int8", "unsigned char", "uint8", "uchar"):
            lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append("    SolixCValue::i32_val(call_result as i32)")
        elif raw_ret in ("long", "long long", "int64", "unsigned long", "unsigned long long", "uint64"):
            lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append("    SolixCValue::i64_val(call_result as i64)")
        elif raw_ret in ("float", "f32"):
            lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append("    SolixCValue::f64_val(call_result as f64)")
        elif raw_ret in ("double", "f64"):
            lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append("    SolixCValue::f64_val(call_result)")
        elif raw_ret in ("const char *", "char *"):
            lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append("    if call_result.is_null() { return SolixCValue::null(); }")
            lines.append("    let c_str = CStr::from_ptr(call_result);")
            lines.append("    let bytes = c_str.to_bytes();")
            lines.append("    let context_ptr = vm_context as *mut SolixVmContext;")
            lines.append("    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);")
            lines.append("    SolixCValue::ref_val(addr)")
        elif ret_ptr_depth == 0 and (base_ret in ir.struct_map or resolved_ret in ir.struct_map):
            concrete_st = base_ret if base_ret in ir.struct_map else resolved_ret
            struct_snake = to_snake_case(concrete_st)
            lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append(f"    SolixCValue::ref_val(pack_{struct_snake}(vm_context, call_result))")
        else:
            lines.append(f"    let call_result = {call_invocation};")
            for post_line in post_call_lines:
                lines.append(post_line)
            lines.append("    SolixCValue::i64_val(call_result as usize as i64)")

        lines.append("}")
        lines.append("")

        snake_name = func_obj.snake_name
        func_name = func_obj.name
        registration_entries.append((f"{class_name}::{func_name}", wrapper_name))
        registration_entries.append((f"{class_name}::{snake_name}", wrapper_name))
        registration_entries.append((f"{package_name}::{func_name}", wrapper_name))
        registration_entries.append((f"{package_name}::{snake_name}", wrapper_name))
        registration_entries.append((func_name, wrapper_name))
        registration_entries.append((snake_name, wrapper_name))

    lines.append("// ===========================================================================")
    lines.append("// Native Symbol Dynamic Registration Hook")
    lines.append("// ===========================================================================")
    lines.append("#[unsafe(no_mangle)]")
    lines.append("pub unsafe extern \"C\" fn solix_register_natives(")
    lines.append("    collector_context: *mut c_void,", )
    lines.append("    register_callback: RegisterNativeCallback,")
    lines.append(") {")

    seen_symbols = set()
    for symbol_path, wrapper_fn_name in registration_entries:
        if symbol_path in seen_symbols:
            continue
        seen_symbols.add(symbol_path)
        lines.append(
            f'    register_callback(collector_context, c"{symbol_path}".as_ptr(), {wrapper_fn_name});'
        )

    lines.append("}")
    lines.append("")

    return "\n".join(lines) + "\n"


# ============================================================================
# Main Entry Point
# ============================================================================

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Unified Two-Pass IR C-API Binding Generator for Solix and Rust FFI."
    )
    parser.add_argument(
        "--api",
        required=True,
        help="Path to rlparser C-API JSON schema (e.g. build/.cache/api/raylib_api.json)",
    )
    parser.add_argument(
        "--out-slx",
        required=True,
        help="Output path for generated Solix source file (e.g. src/raylib.slx)",
    )
    parser.add_argument(
        "--out-rs",
        required=True,
        help="Output path for generated Rust companion bridge (e.g. natives/raylib/src/lib.rs)",
    )
    parser.add_argument(
        "--package",
        default="raylib",
        help="Package namespace identifier for Solix (default: raylib)",
    )
    parser.add_argument(
        "--class-name",
        default="Raylib",
        help="Primary class name for Solix engine wrapper (default: Raylib)",
    )
    parser.add_argument(
        "--define-guard",
        default="RLAPI",
        help="API macro define guard to skip (default: RLAPI)",
    )

    cli_arguments = parser.parse_args()

    api_path = Path(cli_arguments.api)
    out_slx_path = Path(cli_arguments.out_slx)
    out_rs_path = Path(cli_arguments.out_rs)

    if not api_path.exists():
        print(f"Error: Schema file not found at '{api_path}'", file=sys.stderr)
        sys.exit(1)

    with open(api_path, "r", encoding="utf-8") as schema_file:
        api_data = json.load(schema_file)

    # Pass 1: Normalize into pure IR representation
    ir = parse_api_to_ir(api_data, cli_arguments.define_guard)

    # Pass 2: Emit Solix declarations
    out_slx_path.parent.mkdir(parents=True, exist_ok=True)
    slx_code = emit_solix_declarations(ir, cli_arguments.package, cli_arguments.class_name)
    with open(out_slx_path, "w", encoding="utf-8") as slx_file:
        slx_file.write(slx_code)
    print(f"  [codegen] Emitted Solix API: {out_slx_path} ({len(slx_code.splitlines())} lines)")

    # Pass 2: Emit Rust FFI bridge
    out_rs_path.parent.mkdir(parents=True, exist_ok=True)
    rs_code = emit_rust_bridge(ir, cli_arguments.package, cli_arguments.class_name)
    with open(out_rs_path, "w", encoding="utf-8") as rs_file:
        rs_file.write(rs_code)
    print(f"  [codegen] Emitted Rust bridge: {out_rs_path} ({len(rs_code.splitlines())} lines)")


if __name__ == "__main__":
    main()
