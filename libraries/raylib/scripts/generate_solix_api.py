#!/usr/bin/env python3
"""
generate_solix_api.py

Generates 1-to-1 Solix bindings (`src/raylib.slx`) from Raylib API JSON (`raylib_api.json`).
Follows Solix language conventions and GL-0011 (no single-letter variables).
"""

import json
import os
import sys
import argparse
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
    """Escapes Solix reserved keywords by appending an underscore."""
    if identifier_name in RESERVED_KEYWORDS:
        return f"{identifier_name}_val"
    return identifier_name

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
    result = "".join(snake_chars)
    if result in RESERVED_KEYWORDS:
        return f"{result}_fn"
    return result

def map_c_type_to_solix(c_type_str: str) -> str:
    """Maps a C type string from raylib_api.json to a Solix type."""
    normalized = c_type_str.strip()
    
    # Primitives
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
    
    # Double pointers / complex pointers
    if normalized in ("Rectangle **", "void **"):
        return "uint64"
    
    # Primitive arrays
    if normalized in ("const int *", "int *"):
        return "int32[]"
    if normalized in ("const unsigned int *", "unsigned int *"):
        return "uint32[]"
    if normalized in ("const float *", "float *"):
        return "float32[]"
    if normalized in ("const unsigned char *", "unsigned char *"):
        return "uint8[]"
    if normalized in ("const unsigned short *", "unsigned short *"):
        return "uint16[]"
    
    # Struct arrays / pointers
    if normalized in ("const Vector2 *", "Vector2 *"):
        return "Vector2[]"
    if normalized in ("const Vector3 *", "Vector3 *"):
        return "Vector3[]"
    if normalized in ("const Matrix *", "Matrix *"):
        return "Matrix[]"
    if normalized in ("const GlyphInfo *", "GlyphInfo *"):
        return "GlyphInfo[]"
    if normalized in ("Color *", "const Color *"):
        return "Color[]"
    
    # In-place mutable struct pointers -> Solix reference object
    if normalized in ("Camera *", "Camera3D *"):
        return "Camera3D"
    if normalized in ("Camera2D *",):
        return "Camera2D"
    if normalized in ("Image *",):
        return "Image"
    if normalized in ("Wave *",):
        return "Wave"
    if normalized in ("Mesh *",):
        return "Mesh"
    if normalized in ("Model *",):
        return "Model"
    if normalized in ("Texture2D *", "Texture *"):
        return "Texture2D"
    if normalized in ("Rectangle *",):
        return "Rectangle"
    
    # Opaque pointers & callbacks
    if "Callback" in normalized or normalized in (
        "void *", "const void *", "AutomationEvent *", "AutomationEventList *",
        "BoneInfo *", "Material *", "MaterialMap *", "ModelAnimPose *",
        "ModelAnimation *", "rAudioBuffer *", "rAudioProcessor *"
    ):
        return "uint64"
    
    # Strip any remaining pointer
    if normalized.endswith(" *"):
        base_struct = normalized[:-2].strip()
        if base_struct.startswith("const "):
            base_struct = base_struct[6:].strip()
        return base_struct
    
    # Type aliases
    if normalized == "Texture":
        return "Texture2D"
    if normalized == "Camera":
        return "Camera3D"
    if normalized == "Quaternion":
        return "Vector4"
    if normalized == "RenderTexture":
        return "RenderTexture2D"
    if normalized == "ModelAnimPose":
        return "uint64"
    
    return normalized

def generate_defines(defines_list: list) -> list:
    """Generates Solix constant declarations for Raylib defines."""
    lines = []
    lines.append("// ===========================================================================")
    lines.append("// Raylib Version and Mathematical Constants")
    lines.append("// ===========================================================================")
    
    for define_item in defines_list:
        name = define_item["name"]
        item_type = define_item.get("type", "")
        value = define_item.get("value", "")
        
        # Skip include guards and internal macros
        if item_type in ("GUARD", "MACRO", "UNKNOWN"):
            continue
        
        if item_type == "INT":
            lines.append(f"const int32 {name} = {value};")
        elif item_type == "FLOAT":
            lines.append(f"const float32 {name} = (float32){value};")
        elif item_type == "STRING":
            lines.append(f'const string {name} = "{value}";')
        elif item_type == "FLOAT_MATH":
            if name == "DEG2RAD":
                lines.append(f"const float32 DEG2RAD = (float32)0.017453292519943295;")
            elif name == "RAD2DEG":
                lines.append(f"const float32 RAD2DEG = (float32)57.29577951308232;")
    lines.append("")
    return lines

def generate_enums(enums_list: list) -> list:
    """Generates Solix enum declarations."""
    lines = []
    lines.append("// ===========================================================================")
    lines.append("// Raylib Enums and Bitflags")
    lines.append("// ===========================================================================")
    
    for enum_item in enums_list:
        enum_name = enum_item["name"]
        description = enum_item.get("description", "")
        if description:
            lines.append(f"/// {description}")
        lines.append(f"public enum {enum_name} {{")
        
        members = enum_item.get("values", [])
        member_count = len(members)
        for index, member in enumerate(members):
            member_name = member["name"]
            member_value = member["value"]
            member_desc = member.get("description", "")
            comma = "," if index + 1 < member_count else ""
            if member_desc:
                lines.append(f"    /// {member_desc}")
            lines.append(f"    {member_name} = {member_value}{comma}")
        lines.append("}")
        lines.append("")
    return lines

def generate_structs(structs_list: list, defines_list: list) -> list:
    """Generates Solix class declarations matching Raylib C structs."""
    lines = []
    lines.append("// ===========================================================================")
    lines.append("// Raylib Struct Classes (1-to-1 Solix Classes)")
    lines.append("// ===========================================================================")
    
    color_defines = [define for define in defines_list if define.get("type") == "COLOR"]
    
    for struct_item in structs_list:
        struct_name = struct_item["name"]
        description = struct_item.get("description", "")
        if description:
            lines.append(f"/// {description}")
        lines.append(f"public class {struct_name} {{")
        
        fields = struct_item.get("fields", [])
        for field in fields:
            field_name = sanitize_identifier(field["name"])
            raw_field_type = field["type"]
            
            if "[" in raw_field_type:
                base_type_str = raw_field_type.split("[")[0].strip()
                if base_type_str in ("char", "const char"):
                    solix_type = "string"
                else:
                    elem_type = map_c_type_to_solix(base_type_str)
                    solix_type = f"{elem_type}[]"
            else:
                solix_type = map_c_type_to_solix(raw_field_type)
            
            field_desc = field.get("description", "")
            if field_desc:
                lines.append(f"    /// {field_desc}")
            lines.append(f"    public {solix_type} {field_name};")
        
        lines.append("")
        
        # Default zero constructor
        lines.append(f"    public {struct_name}() {{")
        for field in fields:
            field_name = sanitize_identifier(field["name"])
            raw_field_type = field["type"]
            
            if "[" in raw_field_type:
                base_type_str = raw_field_type.split("[")[0].strip()
                if base_type_str in ("char", "const char"):
                    lines.append(f'        this.{field_name} = "";')
                else:
                    elem_type = map_c_type_to_solix(base_type_str)
                    size_str = raw_field_type.split("[")[1].split("]")[0].strip()
                    lines.append(f"        this.{field_name} = new {elem_type}[{size_str}];")
            else:
                solix_type = map_c_type_to_solix(raw_field_type)
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
                    lines.append(f"        this.{field_name} = new {solix_type[:-2]}[0];")
                else:
                    lines.append(f"        this.{field_name} = new {solix_type}();")
        lines.append("    }")
        
        # Parameterized constructor for structs with <= 8 non-array fields
        if 0 < len(fields) <= 8 and not any("[" in field["type"] for field in fields):
            params_str_list = []
            assign_lines = []
            for field in fields:
                field_name = sanitize_identifier(field["name"])
                solix_type = map_c_type_to_solix(field["type"])
                param_name = f"init_{field_name}"
                params_str_list.append(f"{solix_type} {param_name}")
                assign_lines.append(f"        this.{field_name} = {param_name};")
            
            lines.append("")
            lines.append(f"    public {struct_name}({', '.join(params_str_list)}) {{")
            for assign_line in assign_lines:
                lines.append(assign_line)
            lines.append("    }")
        
        # Color preset static instances
        if struct_name == "Color":
            lines.append("")
            lines.append("    // --- Raylib Palette Preset Constants ---")
            for color_item in color_defines:
                color_name = color_item["name"]
                color_val = color_item["value"]
                try:
                    inner = color_val.split("{")[1].split("}")[0].strip()
                    rgba = [component.strip() for component in inner.split(",")]
                    red_val, green_val, blue_val, alpha_val = rgba[0], rgba[1], rgba[2], rgba[3]
                    lines.append(f"    public static Color {color_name} = new Color((uint8){red_val}, (uint8){green_val}, (uint8){blue_val}, (uint8){alpha_val});")
                except Exception:
                    pass
        
        lines.append("}")
        lines.append("")
    return lines

def generate_aliases(aliases_list: list) -> list:
    """Generates Solix class subclass extensions for Raylib type aliases."""
    lines = []
    lines.append("// ===========================================================================")
    lines.append("// Raylib Type Aliases (Extended Subclasses)")
    lines.append("// ===========================================================================")
    
    for alias_item in aliases_list:
        alias_name = alias_item["name"]
        target_type = alias_item["type"]
        description = alias_item.get("description", "")
        
        if "*" in target_type:
            continue
        
        if description:
            lines.append(f"/// {description}")
        lines.append(f"public class {alias_name} extends {target_type} {{")
        lines.append(f"    public {alias_name}() {{}}")
        lines.append("}")
        lines.append("")
    return lines

def generate_functions(functions_list: list, defines_list: list) -> list:
    """Generates the primary Raylib class and top-level native function wrappers."""
    lines = []
    lines.append("// ===========================================================================")
    lines.append("// Raylib Primary Engine API (Class Raylib & Top-Level Functions)")
    lines.append("// ===========================================================================")
    lines.append("public class Raylib {")
    
    # Class-level constants for quick access via Raylib::PI, Raylib::DEG2RAD, etc.
    lines.append("    public static const float32 PI = (float32)3.141592653589793;")
    lines.append("    public static const float32 DEG2RAD = (float32)0.017453292519943295;")
    lines.append("    public static const float32 RAD2DEG = (float32)57.29577951308232;")
    lines.append("")
    
    for func in functions_list:
        func_name = func["name"]
        raw_return_type = func.get("returnType", "void")
        solix_ret_type = map_c_type_to_solix(raw_return_type)
        description = func.get("description", "")
        params = func.get("params", [])
        
        param_declarations = []
        for param in params:
            raw_param_type = param.get("type", "void")
            param_name = sanitize_identifier(param.get("name", "arg"))
            
            if raw_param_type == "...":
                continue
            
            solix_param_type = map_c_type_to_solix(raw_param_type)
            param_declarations.append(f"{solix_param_type} {param_name}")
        
        param_list_str = ", ".join(param_declarations)
        
        if description:
            lines.append(f"    /// {description}")
        lines.append(f"    public static native {solix_ret_type} {func_name}({param_list_str});")
        
        snake_name = to_snake_case(func_name)
        if snake_name != func_name:
            lines.append(f"    public static native {solix_ret_type} {snake_name}({param_list_str});")
    
    lines.append("}")
    lines.append("")
    
    # Top-level functions
    lines.append("// --- Direct Top-Level Function Exports ---")
    for func in functions_list:
        func_name = func["name"]
        raw_return_type = func.get("returnType", "void")
        solix_ret_type = map_c_type_to_solix(raw_return_type)
        params = func.get("params", [])
        
        param_declarations = []
        for param in params:
            raw_param_type = param.get("type", "void")
            param_name = sanitize_identifier(param.get("name", "arg"))
            if raw_param_type == "...":
                continue
            solix_param_type = map_c_type_to_solix(raw_param_type)
            param_declarations.append(f"{solix_param_type} {param_name}")
        param_list_str = ", ".join(param_declarations)
        
        lines.append(f"public native {solix_ret_type} {func_name}({param_list_str});")
        snake_name = to_snake_case(func_name)
        if snake_name != func_name:
            lines.append(f"public native {solix_ret_type} {snake_name}({param_list_str});")
    
    return lines

def main():
    parser = argparse.ArgumentParser(description="Generate Solix Raylib API from raylib_api.json")
    parser.add_argument("--api", default="libraries/raylib/api/raylib_api.json", help="Path to raylib_api.json")
    parser.add_argument("--output", default="libraries/raylib/src/raylib.slx", help="Path to output raylib.slx")
    args = parser.parse_args()
    
    api_path = Path(args.api)
    output_path = Path(args.output)
    
    if not api_path.exists():
        print(f"Error: API file not found at {api_path}", file=sys.stderr)
        sys.exit(1)
        
    with open(api_path, "r", encoding="utf-8") as file_handle:
        api_data = json.load(file_handle)
        
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    output_lines = [
        "// Auto-generated 1-to-1 Raylib bindings for the Solix Programming Language",
        "// Generated from official raylib_api.json specification",
        "",
        "package raylib;",
        "",
    ]
    
    output_lines.extend(generate_defines(api_data.get("defines", [])))
    output_lines.extend(generate_enums(api_data.get("enums", [])))
    output_lines.extend(generate_structs(api_data.get("structs", []), api_data.get("defines", [])))
    output_lines.extend(generate_aliases(api_data.get("aliases", [])))
    output_lines.extend(generate_functions(api_data.get("functions", []), api_data.get("defines", [])))
    
    with open(output_path, "w", encoding="utf-8") as out_file:
        out_file.write("\n".join(output_lines) + "\n")
        
    print(f"Successfully generated {output_path} ({len(output_lines)} lines)")

if __name__ == "__main__":
    main()
