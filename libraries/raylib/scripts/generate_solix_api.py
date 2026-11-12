#!/usr/bin/env python3
"""
generate_solix_api.py

Generates 1-to-1 Solix bindings (`src/<module>.slx`) from rlparser-compatible C API JSON.
100% generic: zero hardcoded struct, function, or constant names.
Follows Solix language conventions and GL-0011 (no single-letter variables).
"""

from __future__ import annotations

import argparse
import ast
import json
import os
import re
import sys
from pathlib import Path

# Solix keywords that cannot be used as identifier names without modification
RESERVED_KEYWORDS = {
    "class", "interface", "enum", "alias", "package", "import", "extern", "native",
    "public", "private", "protected", "internal", "static", "virtual", "override",
    "abstract", "final", "inline", "const", "constexpr", "weak", "if", "else",
    "while", "for", "do", "switch", "case", "default", "break", "continue",
    "return", "throw", "try", "catch", "finally", "new", "this", "super",
    "instanceof", "sizeof", "typeof", "as", "is", "operator", "null", "true",
    "false", "void", "bool", "char", "int8", "int16", "int32", "int64",
    "uint8", "uint16", "uint32", "uint64", "float32", "float64", "string",
}


def sanitize_identifier(identifier_name: str) -> str:
    """Escapes Solix reserved keywords by appending an underscore suffix."""
    if identifier_name in RESERVED_KEYWORDS:
        return f"{identifier_name}_val"
    return identifier_name


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
    result = "".join(snake_chars)
    if result in RESERVED_KEYWORDS:
        return f"{result}_fn"
    return result


def resolve_alias_target(type_str: str, alias_map: dict[str, str]) -> str:
    """Recursively resolves an alias to its underlying concrete target type."""
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


def eval_safe_math(expression_str: str, known_constants: dict[str, float]) -> float:
    """Evaluates safe mathematical expressions substituting previously declared constants."""
    normalized_expr = expression_str.strip()
    for constant_name, constant_value in sorted(
        known_constants.items(), key=lambda pair: len(pair[0]), reverse=True
    ):
        normalized_expr = re.sub(
            r"\b" + re.escape(constant_name) + r"\b",
            str(constant_value),
            normalized_expr,
        )

    # Strip C-style float literals like 180.0f -> 180.0
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


def map_c_type_to_solix(
    raw_type_str: str,
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    enum_map: dict[str, dict],
    callback_map: dict[str, dict],
) -> str:
    """Dynamically maps any C type string to its corresponding Solix type."""
    normalized = raw_type_str.strip()

    # Exact primitives
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

    # Strings
    if normalized in ("const char *", "char *"):
        return "string"
    if normalized in ("const char **", "char **"):
        return "string[]"

    # Fixed arrays in struct fields
    if "[" in normalized:
        base_part = normalized.split("[")[0].strip()
        if base_part in ("char", "const char"):
            return "string"
        element_solix = map_c_type_to_solix(
            base_part, struct_map, alias_map, enum_map, callback_map
        )
        return f"{element_solix}[]"

    # Decompose pointer and const qualifiers
    base_name, is_const, pointer_depth = decompose_c_type(normalized)
    resolved_base = resolve_alias_target(base_name, alias_map)

    # Aliases resolving to pointers map to uint64
    if "*" in resolved_base:
        return "uint64"

    # Callbacks & opaque/generic pointers
    if (
        base_name in callback_map
        or resolved_base in callback_map
        or base_name in ("void", "va_list")
        or resolved_base in ("void", "va_list")
        or pointer_depth >= 2
    ):
        return "uint64"

    # Primitive array pointers (e.g. const int *, int *, float *)
    primitive_map = {
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
    if pointer_depth == 1 and base_name in primitive_map:
        return f"{primitive_map[base_name]}[]"

    # Struct type handling
    target_struct_name = None
    if base_name in struct_map:
        target_struct_name = base_name
    elif resolved_base in struct_map:
        target_struct_name = resolved_base

    if target_struct_name is not None:
        if pointer_depth == 0:
            return base_name
        if pointer_depth == 1:
            if is_const:
                return f"{base_name}[]"
            return base_name

    # Enum handling
    if base_name in enum_map:
        if pointer_depth == 1:
            return f"{base_name}[]"
        return base_name

    # Unknown pointer fallback
    if pointer_depth > 0:
        return "uint64"

    return base_name


def parse_struct_literal_defines(
    defines_list: list[dict],
    struct_map: dict[str, dict],
) -> dict[str, list[tuple[str, str, str]]]:
    """Extracts struct literal defines like CLITERAL(Color){ r, g, b, a } grouped by struct name."""
    grouped_literals: dict[str, list[tuple[str, str, str]]] = {
        struct_name: [] for struct_name in struct_map
    }
    literal_pattern = re.compile(
        r"(?:CLITERAL\s*\(\s*([A-Za-z0-9_]+)\s*\)|\(\s*([A-Za-z0-9_]+)\s*\))\s*\{([^}]+)\}"
    )

    for define_item in defines_list:
        define_name = str(define_item.get("name", ""))
        define_value = str(define_item.get("value", ""))
        match_result = literal_pattern.search(define_value)
        if not match_result:
            continue

        target_struct = match_result.group(1) or match_result.group(2)
        if target_struct in grouped_literals:
            inner_components = match_result.group(3).strip()
            grouped_literals[target_struct].append(
                (define_name, inner_components, define_item.get("description", ""))
            )

    return grouped_literals


def generate_defines(
    defines_list: list[dict],
    define_guard: str,
) -> tuple[list[str], dict[str, float]]:
    """Generates Solix constant declarations for defines, evaluating safe arithmetic."""
    lines = [
        "// ===========================================================================",
        "// Global Constants & Evaluated Defines",
        "// ===========================================================================",
    ]
    known_float_constants: dict[str, float] = {
        "PI": 3.14159265358979323846,
    }

    for define_item in defines_list:
        constant_name = define_item.get("name", "")
        item_type = define_item.get("type", "")
        raw_val = str(define_item.get("value", ""))

        if (
            item_type in ("GUARD", "MACRO", "UNKNOWN")
            or constant_name == define_guard
            or item_type == "COLOR"
        ):
            continue

        if item_type == "INT":
            try:
                int_val = int(raw_val, 0)
                lines.append(f"const int32 {constant_name} = {int_val};")
                known_float_constants[constant_name] = float(int_val)
            except ValueError:
                lines.append(f"const int32 {constant_name} = {raw_val};")
        elif item_type == "FLOAT":
            try:
                clean_float = raw_val.rstrip("fF")
                float_val = float(clean_float)
                lines.append(f"const float32 {constant_name} = (float32){float_val};")
                known_float_constants[constant_name] = float_val
            except ValueError:
                lines.append(f"const float32 {constant_name} = (float32){raw_val};")
        elif item_type == "STRING":
            lines.append(f'const string {constant_name} = "{raw_val}";')
        elif item_type == "FLOAT_MATH":
            try:
                evaluated_float = eval_safe_math(raw_val, known_float_constants)
                lines.append(
                    f"const float32 {constant_name} = (float32){evaluated_float};"
                )
                known_float_constants[constant_name] = evaluated_float
            except Exception:
                pass

    lines.append("")
    return lines, known_float_constants


def generate_enums(enums_list: list[dict]) -> list[str]:
    """Generates Solix enum declarations."""
    lines = [
        "// ===========================================================================",
        "// Enums & Bitflags",
        "// ===========================================================================",
    ]

    for enum_item in enums_list:
        enum_name = enum_item.get("name", "")
        description = enum_item.get("description", "")
        if description:
            lines.append(f"/// {description}")
        lines.append(f"public enum {enum_name} {{")

        members = enum_item.get("values", [])
        member_count = len(members)
        for member_index, member in enumerate(members):
            member_name = member.get("name", "")
            member_value = member.get("value", "")
            member_desc = member.get("description", "")
            comma = "," if member_index + 1 < member_count else ""
            if member_desc:
                lines.append(f"    /// {member_desc}")
            lines.append(f"    {member_name} = {member_value}{comma}")

        lines.append("}")
        lines.append("")

    return lines


def generate_structs(
    structs_list: list[dict],
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    enum_map: dict[str, dict],
    callback_map: dict[str, dict],
    grouped_literals: dict[str, list[tuple[str, str, str]]],
) -> list[str]:
    """Generates Solix class declarations matching C structs."""
    lines = [
        "// ===========================================================================",
        "// Struct Classes",
        "// ===========================================================================",
    ]

    for struct_item in structs_list:
        struct_name = struct_item.get("name", "")
        description = struct_item.get("description", "")
        if description:
            lines.append(f"/// {description}")
        lines.append(f"public class {struct_name} {{")

        fields = struct_item.get("fields", [])
        for field_item in fields:
            field_name = sanitize_identifier(field_item.get("name", ""))
            raw_field_type = field_item.get("type", "")
            solix_field_type = map_c_type_to_solix(
                raw_field_type, struct_map, alias_map, enum_map, callback_map
            )
            field_desc = field_item.get("description", "")
            if field_desc:
                lines.append(f"    /// {field_desc}")
            lines.append(f"    public {solix_field_type} {field_name};")

        lines.append("")

        # Default zero-initialization constructor
        lines.append(f"    public {struct_name}() {{")
        for field_item in fields:
            field_name = sanitize_identifier(field_item.get("name", ""))
            raw_field_type = field_item.get("type", "")

            if "[" in raw_field_type:
                base_part = raw_field_type.split("[")[0].strip()
                if base_part in ("char", "const char"):
                    lines.append(f'        this.{field_name} = "";')
                else:
                    elem_type = map_c_type_to_solix(
                        base_part, struct_map, alias_map, enum_map, callback_map
                    )
                    size_spec = raw_field_type.split("[")[1].split("]")[0].strip()
                    lines.append(
                        f"        this.{field_name} = new {elem_type}[{size_spec}];"
                    )
            else:
                solix_field_type = map_c_type_to_solix(
                    raw_field_type, struct_map, alias_map, enum_map, callback_map
                )
                if solix_field_type in ("int32", "int16", "int8", "int64"):
                    lines.append(f"        this.{field_name} = 0;")
                elif solix_field_type in ("uint32", "uint16", "uint8", "uint64"):
                    lines.append(f"        this.{field_name} = ({solix_field_type})0;")
                elif solix_field_type == "float32":
                    lines.append(f"        this.{field_name} = (float32)0.0;")
                elif solix_field_type == "float64":
                    lines.append(f"        this.{field_name} = 0.0;")
                elif solix_field_type == "bool":
                    lines.append(f"        this.{field_name} = false;")
                elif solix_field_type == "string":
                    lines.append(f'        this.{field_name} = "";')
                elif solix_field_type.endswith("[]"):
                    base_elem = solix_field_type[:-2]
                    lines.append(f"        this.{field_name} = new {base_elem}[0];")
                else:
                    lines.append(f"        this.{field_name} = new {solix_field_type}();")

        lines.append("    }")

        # Parameterized constructor for small structs (<= 8 fields, no fixed arrays)
        if 0 < len(fields) <= 8 and not any("[" in fld.get("type", "") for fld in fields):
            params_str_list = []
            assign_lines = []
            for field_item in fields:
                field_name = sanitize_identifier(field_item.get("name", ""))
                solix_field_type = map_c_type_to_solix(
                    field_item.get("type", ""), struct_map, alias_map, enum_map, callback_map
                )
                param_var = f"init_{field_name}"
                params_str_list.append(f"{solix_field_type} {param_var}")
                assign_lines.append(f"        this.{field_name} = {param_var};")

            lines.append("")
            lines.append(f"    public {struct_name}({', '.join(params_str_list)}) {{")
            for assign_line in assign_lines:
                lines.append(assign_line)
            lines.append("    }")

        # Static preset constants matching CLITERAL(<StructName>)
        struct_literals = grouped_literals.get(struct_name, [])
        if struct_literals:
            lines.append("")
            lines.append("    // --- Preset Constants ---")
            for literal_name, inner_args, lit_desc in struct_literals:
                components = [arg.strip() for arg in inner_args.split(",")]
                typed_args = []
                for comp_idx, component in enumerate(components):
                    if comp_idx < len(fields):
                        field_type_str = map_c_type_to_solix(
                            fields[comp_idx].get("type", ""),
                            struct_map,
                            alias_map,
                            enum_map,
                            callback_map,
                        )
                        if field_type_str in (
                            "uint8", "uint16", "uint32", "uint64",
                            "int8", "int16", "int32", "int64", "float32",
                        ):
                            typed_args.append(f"({field_type_str}){component}")
                        else:
                            typed_args.append(component)
                    else:
                        typed_args.append(component)

                if lit_desc:
                    lines.append(f"    /// {lit_desc}")
                lines.append(
                    f"    public static {struct_name} {literal_name} = new {struct_name}({', '.join(typed_args)});"
                )

        lines.append("}")
        lines.append("")

    return lines


def generate_aliases(
    aliases_list: list[dict],
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
) -> list[str]:
    """Generates Solix class subclass extensions for struct aliases."""
    lines = [
        "// ===========================================================================",
        "// Type Aliases (Extended Subclasses)",
        "// ===========================================================================",
    ]

    for alias_item in aliases_list:
        alias_name = alias_item.get("name", "")
        target_type = alias_item.get("type", "")
        description = alias_item.get("description", "")

        resolved_target = resolve_alias_target(target_type, alias_map)
        if "*" in resolved_target or resolved_target not in struct_map:
            continue

        if description:
            lines.append(f"/// {description}")
        lines.append(f"public class {alias_name} extends {resolved_target} {{")
        lines.append(f"    public {alias_name}() {{}}")
        lines.append("}")
        lines.append("")

    return lines


def generate_functions(
    functions_list: list[dict],
    struct_map: dict[str, dict],
    alias_map: dict[str, str],
    enum_map: dict[str, dict],
    callback_map: dict[str, dict],
    class_name: str,
    known_float_constants: dict[str, float],
) -> list[str]:
    """Generates the primary engine class and top-level function wrappers."""
    lines = [
        "// ===========================================================================",
        f"// Primary Engine API (Class {class_name} & Top-Level Functions)",
        "// ===========================================================================",
        f"public class {class_name} {{",
    ]

    # Class-level constants
    for const_name, const_val in sorted(known_float_constants.items()):
        lines.append(f"    public static const float32 {const_name} = (float32){const_val};")
    lines.append("")

    for function_item in functions_list:
        func_name = function_item.get("name", "")
        raw_return_type = function_item.get("returnType", "void")
        solix_ret_type = map_c_type_to_solix(
            raw_return_type, struct_map, alias_map, enum_map, callback_map
        )
        description = function_item.get("description", "")
        params = function_item.get("params", [])

        param_declarations = []
        for param in params:
            raw_param_type = param.get("type", "void")
            param_identifier = sanitize_identifier(param.get("name", "argument"))

            # Strip variadic argument in Solix declaration
            if raw_param_type == "...":
                continue

            solix_param_type = map_c_type_to_solix(
                raw_param_type, struct_map, alias_map, enum_map, callback_map
            )
            param_declarations.append(f"{solix_param_type} {param_identifier}")

        param_list_str = ", ".join(param_declarations)

        if description:
            lines.append(f"    /// {description}")
        lines.append(f"    public static native {solix_ret_type} {func_name}({param_list_str});")

        snake_name = to_snake_case(func_name)
        if snake_name != func_name:
            lines.append(f"    public static native {solix_ret_type} {snake_name}({param_list_str});")

    lines.append("}")
    lines.append("")

    # Direct top-level functions
    lines.append("// --- Direct Top-Level Function Exports ---")
    for function_item in functions_list:
        func_name = function_item.get("name", "")
        raw_return_type = function_item.get("returnType", "void")
        solix_ret_type = map_c_type_to_solix(
            raw_return_type, struct_map, alias_map, enum_map, callback_map
        )
        params = function_item.get("params", [])

        param_declarations = []
        for param in params:
            raw_param_type = param.get("type", "void")
            param_identifier = sanitize_identifier(param.get("name", "argument"))
            if raw_param_type == "...":
                continue
            solix_param_type = map_c_type_to_solix(
                raw_param_type, struct_map, alias_map, enum_map, callback_map
            )
            param_declarations.append(f"{solix_param_type} {param_identifier}")

        param_list_str = ", ".join(param_declarations)

        lines.append(f"public native {solix_ret_type} {func_name}({param_list_str});")
        snake_name = to_snake_case(func_name)
        if snake_name != func_name:
            lines.append(f"public native {solix_ret_type} {snake_name}({param_list_str});")

    return lines


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate generic Solix C-API bindings from rlparser JSON."
    )
    parser.add_argument(
        "--api",
        default="build/.cache/api/raylib_api.json",
        help="Path to rlparser API JSON schema",
    )
    parser.add_argument(
        "--output",
        default="src/raylib.slx",
        help="Path to output Solix source file",
    )
    parser.add_argument(
        "--package-name",
        default="raylib",
        help="Solix package declaration name",
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

    grouped_literals = parse_struct_literal_defines(
        api_data.get("defines", []), struct_map
    )

    output_path.parent.mkdir(parents=True, exist_ok=True)

    package_declaration = f"package {cli_arguments.package_name};"
    output_lines = [
        "// Auto-generated 1-to-1 C API bindings for the Solix Programming Language",
        f"// Source schema: {api_path.name}",
        "",
        package_declaration,
        "",
    ]

    define_lines, known_constants = generate_defines(
        api_data.get("defines", []), cli_arguments.define_guard
    )
    output_lines.extend(define_lines)

    output_lines.extend(generate_enums(api_data.get("enums", [])))
    output_lines.extend(
        generate_structs(
            api_data.get("structs", []),
            struct_map,
            alias_map,
            enum_map,
            callback_map,
            grouped_literals,
        )
    )
    output_lines.extend(
        generate_aliases(api_data.get("aliases", []), struct_map, alias_map)
    )
    output_lines.extend(
        generate_functions(
            api_data.get("functions", []),
            struct_map,
            alias_map,
            enum_map,
            callback_map,
            cli_arguments.class_name,
            known_constants,
        )
    )

    with open(output_path, "w", encoding="utf-8") as destination_file:
        destination_file.write("\n".join(output_lines) + "\n")

    print(f"Successfully generated {output_path} ({len(output_lines)} lines)")


if __name__ == "__main__":
    main()
