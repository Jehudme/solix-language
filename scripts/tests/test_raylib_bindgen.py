#!/usr/bin/env python3
"""Unit tests for Raylib 1-to-1 Solix and `solix-ffi` Rust binding generation."""

from __future__ import annotations

from pathlib import Path
import sys
import unittest

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(REPO_ROOT / "libraries" / "raylib" / "scripts"))

import generate_bindings  # noqa: E402


SAMPLE_SCHEMA = {
    "defines": [
        {
            "name": "RAYLIB_VERSION_MAJOR",
            "type": "INT",
            "value": 6,
            "description": "Raylib major version",
        },
        {
            "name": "PI",
            "type": "FLOAT",
            "value": 3.141592653589793,
            "description": "Archimedes constant",
        },
    ],
    "structs": [
        {
            "name": "Vector2",
            "description": "Vector2, 2 components",
            "fields": [
                {"name": "x", "type": "float", "description": "Vector x component"},
                {"name": "y", "type": "float", "description": "Vector y component"},
            ],
        }
    ],
    "aliases": [],
    "enums": [
        {
            "name": "TraceLogLevel",
            "description": "Trace log level",
            "values": [
                {"name": "LOG_NONE", "value": 7, "description": "Disable logging"},
            ],
        }
    ],
    "callbacks": [],
    "functions": [
        {
            "name": "InitWindow",
            "description": "Initialize window and OpenGL context",
            "returnType": "void",
            "params": [
                {"name": "width", "type": "int"},
                {"name": "height", "type": "int"},
                {"name": "title", "type": "const char *"},
            ],
        },
        {
            "name": "CheckCollisionPointRec",
            "description": "Check if point is inside rectangle",
            "returnType": "bool",
            "params": [
                {"name": "point", "type": "Vector2"},
                {"name": "rec", "type": "Vector2"},
            ],
        },
    ],
}


class TestRaylibBindgen(unittest.TestCase):
    """Validates canonical PascalCase root exports and `solix-ffi` SDK emission."""

    def test_solix_output_emits_root_pascal_case_without_wrapper_or_snake_case(self) -> None:
        api_ir = generate_bindings.parse_api_to_ir(SAMPLE_SCHEMA, "RLAPI")
        solix_source = generate_bindings.emit_solix_declarations(api_ir, "raylib", "Raylib")

        # Positive invariants: package root constants, doc comments, and canonical PascalCase functions
        self.assertIn("package raylib;", solix_source)
        self.assertIn("const int32 RAYLIB_VERSION_MAJOR = 6;", solix_source)
        self.assertIn("/// Initialize window and OpenGL context", solix_source)
        self.assertIn(
            "public native void InitWindow(int32 width, int32 height, string title);",
            solix_source,
        )
        self.assertIn(
            "public native bool CheckCollisionPointRec(Vector2 point, Vector2 rec);",
            solix_source,
        )

        # Negative invariants: no Raylib wrapper class and no snake_case duplicates
        self.assertNotIn("public class Raylib", solix_source)
        self.assertNotIn("class Raylib", solix_source)
        self.assertNotIn("init_window(", solix_source)
        self.assertNotIn("check_collision_point_rec(", solix_source)

    def test_rust_bridge_uses_solix_ffi_and_canonical_exports(self) -> None:
        api_ir = generate_bindings.parse_api_to_ir(SAMPLE_SCHEMA, "RLAPI")
        rust_source = generate_bindings.emit_rust_bridge(api_ir, "raylib", "Raylib")

        # Positive invariants: official solix-ffi SDK types & registration macro
        self.assertIn("use solix_ffi::{SolixCValue, SolixVmContext, solix_export};", rust_source)
        self.assertIn("solix_export! {", rust_source)
        self.assertIn('"raylib::InitWindow" => Raylib_InitWindow,', rust_source)
        self.assertIn('"InitWindow" => Raylib_InitWindow,', rust_source)
        self.assertIn("SolixCValue::from_bool(call_result)", rust_source)
        self.assertIn("SolixVmContext::validate(vm_context)", rust_source)

        # Negative invariants: no hand-rolled FFI types, no Raylib:: class symbols, no snake_case aliases
        self.assertNotIn("pub struct SolixCValue", rust_source)
        self.assertNotIn("pub struct SolixVmContext", rust_source)
        self.assertNotIn("Raylib::InitWindow", rust_source)
        self.assertNotIn('"raylib::init_window"', rust_source)
        self.assertNotIn('"init_window"', rust_source)
        self.assertNotIn("Raylib::check_collision_point_rec", rust_source)

    def test_rust_bridge_hardened_marshalling_large_struct_utf8_and_c_unwind(self) -> None:
        extended_schema = {
            "defines": [],
            "structs": [
                {
                    "name": "BigTextureMeta",
                    "description": "10-field struct testing >= 8 fields, unsigned ints, and char[N]",
                    "fields": [
                        {"name": "f0", "type": "int"},
                        {"name": "f1", "type": "int"},
                        {"name": "f2", "type": "int"},
                        {"name": "f3", "type": "int"},
                        {"name": "f4", "type": "int"},
                        {"name": "f5", "type": "int"},
                        {"name": "f6", "type": "int"},
                        {"name": "f7", "type": "int"},
                        {"name": "flags", "type": "unsigned int"},
                        {"name": "label", "type": "char[32]"},
                    ],
                }
            ],
            "aliases": [],
            "enums": [],
            "callbacks": [],
            "functions": [
                {
                    "name": "GetMetaName",
                    "description": "Returns C string",
                    "returnType": "const char *",
                    "params": [{"name": "meta", "type": "BigTextureMeta"}],
                }
            ],
        }
        api_ir = generate_bindings.parse_api_to_ir(extended_schema, "RLAPI")
        rust_source = generate_bindings.emit_rust_bridge(api_ir, "raylib", "Raylib")

        # C-unwind ABI, null/arity guards, explicit by_index field accessors, unsigned 64-bit writeback
        self.assertIn('pub unsafe extern "C-unwind" fn Raylib_GetMetaName(', rust_source)
        self.assertIn("args.is_null() || (arg_count as usize) < 1", rust_source)
        self.assertIn("ctx.set_field_i32_by_index(addr, 0,", rust_source)
        self.assertIn("ctx.write_field_u64_by_index(addr, 8, val.flags as u64);", rust_source)
        self.assertIn("ctx.read_field_u64_by_index(addr, 8) as c_uint;", rust_source)
        self.assertIn("String::from_utf8_lossy(raw_slice_9)", rust_source)
        self.assertIn("if str_addr_9 != 0 {", rust_source)
        self.assertIn("c_str.to_string_lossy()", rust_source)
        self.assertIn("if addr == 0 { return SolixCValue::null(); }", rust_source)
        # Ensure ambiguous legacy helpers are not emitted for struct marshalling
        self.assertNotIn("ctx.set_field_i32(", rust_source)
        self.assertNotIn("ctx.get_field_i32(", rust_source)
        self.assertNotIn("ctx.read_field_u64(", rust_source)
        self.assertNotIn("ctx.write_field_u64(", rust_source)

    def test_solix_ergonomic_operators_helpers_and_raii_wrappers(self) -> None:
        ergonomic_schema = {
            "defines": [],
            "structs": [
                {
                    "name": "Vector2",
                    "description": "2D vector",
                    "fields": [{"name": "x", "type": "float"}, {"name": "y", "type": "float"}],
                },
                {
                    "name": "Vector3",
                    "description": "3D vector",
                    "fields": [
                        {"name": "x", "type": "float"},
                        {"name": "y", "type": "float"},
                        {"name": "z", "type": "float"},
                    ],
                },
                {
                    "name": "Color",
                    "description": "RGBA color",
                    "fields": [
                        {"name": "r", "type": "unsigned char"},
                        {"name": "g", "type": "unsigned char"},
                        {"name": "b", "type": "unsigned char"},
                        {"name": "a", "type": "unsigned char"},
                    ],
                },
                {
                    "name": "Rectangle",
                    "description": "Rectangle",
                    "fields": [
                        {"name": "x", "type": "float"},
                        {"name": "y", "type": "float"},
                        {"name": "width", "type": "float"},
                        {"name": "height", "type": "float"},
                    ],
                },
                {
                    "name": "Texture",
                    "description": "GPU texture",
                    "fields": [{"name": "id", "type": "unsigned int"}],
                },
                {
                    "name": "Camera3D",
                    "description": "3D camera",
                    "fields": [{"name": "fovy", "type": "float"}],
                },
            ],
            "aliases": [{"name": "Texture2D", "type": "Texture", "description": "Texture2D"}],
            "enums": [],
            "callbacks": [],
            "functions": [
                {"name": "BeginDrawing", "description": "", "returnType": "void", "params": []},
                {"name": "EndDrawing", "description": "", "returnType": "void", "params": []},
                {
                    "name": "BeginMode3D",
                    "description": "",
                    "returnType": "void",
                    "params": [{"name": "camera", "type": "Camera3D"}],
                },
                {"name": "EndMode3D", "description": "", "returnType": "void", "params": []},
                {
                    "name": "UnloadTexture",
                    "description": "",
                    "returnType": "void",
                    "params": [{"name": "texture", "type": "Texture2D"}],
                },
            ],
        }
        api_ir = generate_bindings.parse_api_to_ir(ergonomic_schema, "RLAPI")
        solix_source = generate_bindings.emit_solix_declarations(api_ir, "raylib", "Raylib")

        self.assertIn("public Vector2 operator+(Vector2 other)", solix_source)
        self.assertIn("public Vector3 cross(Vector3 other)", solix_source)
        self.assertIn("public static Color from_hex(int32 rgba_hex)", solix_source)
        self.assertIn("public bool contains(Vector2 point)", solix_source)
        self.assertIn("public class ManagedTexture {", solix_source)
        self.assertIn("~ManagedTexture()", solix_source)
        self.assertIn("public void with_drawing(void(*)() draw_callback)", solix_source)
        self.assertIn("public void with_mode_3d(Camera3D camera, void(*)() draw_callback)", solix_source)
        self.assertIn("alias Texture2D = Texture;", solix_source)
        self.assertNotIn("public class Texture2D extends Texture", solix_source)

    def test_merge_api_schemas_deduplicates_shared_types_and_filters_rlgl_artifacts(self) -> None:
        rl_schema = {
            "defines": [{"name": "PI", "type": "FLOAT", "value": 3.14159}],
            "structs": [
                {"name": "Vector2", "fields": [{"name": "x", "type": "float"}, {"name": "y", "type": "float"}]},
                {"name": "Matrix", "fields": [{"name": "m0", "type": "float"}]},
            ],
            "aliases": [],
            "enums": [],
            "callbacks": [],
            "functions": [
                {"name": "InitWindow", "returnType": "void", "params": []},
            ],
        }
        rlgl_schema = {
            "defines": [{"name": "PI", "type": "FLOAT", "value": 3.14159}, {"name": "RL_LINES", "type": "INT", "value": 1}],
            "structs": [
                {"name": "Matrix", "fields": [{"name": "m0", "type": "float"}]},
                {"name": "rl_float16", "fields": [{"name": "", "type": "static rl_float16 foo"}]},
                {"name": "rlglData", "fields": [{"name": "vertexCounter", "type": "int"}]},
                {
                    "name": "rlVertexBuffer",
                    "fields": [
                        {"name": "elementCount", "type": "int"},
                        {"name": "indices", "type": "#if defined(OPENGL_33)"},
                        {"name": "indices", "type": "unsigned int *"},
                        {"name": "indices", "type": "unsigned short *"},
                    ],
                },
            ],
            "aliases": [],
            "enums": [],
            "callbacks": [],
            "functions": [
                {"name": "rlMatrixMode", "returnType": "void", "params": [{"name": "mode", "type": "int"}]},
            ],
        }
        raymath_schema = {
            "defines": [{"name": "RMAPI", "type": "GUARD", "value": ""}],
            "structs": [
                {"name": "Vector2", "fields": [{"name": "x", "type": "float"}, {"name": "y", "type": "float"}]},
            ],
            "aliases": [{"name": "Quaternion", "type": "Vector4"}],
            "enums": [],
            "callbacks": [],
            "functions": [
                {"name": "Vector2Add", "returnType": "Vector2", "params": [{"name": "v1", "type": "Vector2"}, {"name": "v2", "type": "Vector2"}]},
            ],
        }

        merged = generate_bindings.merge_api_schemas([rl_schema, rlgl_schema, raymath_schema])
        struct_names = [item["name"] for item in merged["structs"]]
        self.assertEqual(struct_names.count("Vector2"), 1)
        self.assertEqual(struct_names.count("Matrix"), 1)
        self.assertNotIn("rl_float16", struct_names)
        self.assertNotIn("rlglData", struct_names)
        self.assertIn("rlVertexBuffer", struct_names)

        vb_struct = next(item for item in merged["structs"] if item["name"] == "rlVertexBuffer")
        self.assertEqual([field["name"] for field in vb_struct["fields"]], ["elementCount", "indices"])
        self.assertEqual(vb_struct["fields"][1]["type"], "unsigned int *")

        ir = generate_bindings.parse_api_to_ir(merged, ["RLAPI", "RMAPI"])
        solix_source = generate_bindings.emit_solix_declarations(ir, "raylib", "Raylib")
        self.assertIn("public native void rlMatrixMode(int32 mode);", solix_source)
        self.assertIn("public native Vector2 Vector2Add(Vector2 v1, Vector2 v2);", solix_source)
        self.assertNotIn("RMAPI", solix_source)


if __name__ == "__main__":
    unittest.main()
