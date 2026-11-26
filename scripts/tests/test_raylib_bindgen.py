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


if __name__ == "__main__":
    unittest.main()
