#!/usr/bin/env python3
"""Solix Project Generator Script.

Programmatically scaffolds modern Solix projects configured with manifest.toml,
Python build automation, Solix source files, test harnesses, and Rust cdylib bridges.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import shutil
import sys


def to_pascal_case(name: str) -> str:
    """Converts a snake_case, kebab-case, or space-separated name to PascalCase."""
    words = re.split(r"[^a-zA-Z0-9]+", name)
    cleaned = "".join(word.capitalize() for word in words if word)
    return cleaned if cleaned else "App"


def to_snake_case(name: str) -> str:
    """Converts a name to a safe snake_case identifier."""
    words = re.split(r"[^a-zA-Z0-9]+", name)
    cleaned = "_".join(word.lower() for word in words if word)
    return cleaned if cleaned else "app"


def generate_project(
    target_path: Path,
    name: str | None = None,
    version: str = "0.1.0",
    description: str = "A Solix project",
    license_id: str = "Apache-2.0",
    authors: list[str] | None = None,
    main_entry: str = "Main::main",
    test_entry: str = "TestRunner::test",
    is_library: bool = False,
    no_native: bool = False,
    dependencies: list[str] | None = None,
    force: bool = False,
) -> Path:
    """Programmatically creates a complete, ready-to-build Solix project."""
    target_path = Path(target_path).resolve()
    project_name = name if name else target_path.name
    if not project_name:
        project_name = "untitled"

    if authors is None:
        authors = []
    if dependencies is None:
        dependencies = []

    manifest_file = target_path / "manifest.toml"
    if manifest_file.exists() and not force:
        raise FileExistsError(
            f"Target directory '{target_path}' already contains a Solix project (manifest.toml exists). Use --force to overwrite."
        )

    target_path.mkdir(parents=True, exist_ok=True)

    # 1. Write .gitignore
    gitignore_content = "\n".join([
        "/cache/",
        "/build/",
        "/target/",
        "*.slxb",
        "*.slxarc",
        "",
    ])
    (target_path / ".gitignore").write_text(gitignore_content, encoding="utf-8")

    # 2. Parse dependencies into key-value pairs
    parsed_dependencies: dict[str, str] = {}
    for dep in dependencies:
        if "@" in dep:
            dep_name, dep_ver = dep.split("@", 1)
        elif ":" in dep:
            dep_name, dep_ver = dep.split(":", 1)
        elif "=" in dep:
            dep_name, dep_ver = dep.split("=", 1)
        else:
            dep_name, dep_ver = dep, "*"
        parsed_dependencies[dep_name.strip()] = dep_ver.strip()

    # 3. Write manifest.toml
    authors_formatted = ", ".join(f'"{author}"' for author in authors)
    deps_lines = []
    for dep_name, dep_ver in sorted(parsed_dependencies.items()):
        deps_lines.append(f'{dep_name} = "{dep_ver}"')
    deps_block = "\n".join(deps_lines) if deps_lines else "# solix = \"2.0.0\""

    manifest_content = f"""[info]
name = "{project_name}"
version = "{version}"
description = "{description}"
license = "{license_id}"
authors = [{authors_formatted}]

[entrypoints]
main = "{main_entry}"
test = "{test_entry}"

[dependencies]
{deps_block}
"""
    manifest_file.write_text(manifest_content, encoding="utf-8")

    # 4. Create directory layout
    scripts_dir = target_path / "scripts"
    src_dir = target_path / "src"
    tests_dir = target_path / "tests"
    resources_dir = target_path / "resources"
    lib_dir = target_path / "lib"
    natives_dir = target_path / "natives"

    scripts_dir.mkdir(parents=True, exist_ok=True)
    src_dir.mkdir(parents=True, exist_ok=True)
    tests_dir.mkdir(parents=True, exist_ok=True)
    resources_dir.mkdir(parents=True, exist_ok=True)
    lib_dir.mkdir(parents=True, exist_ok=True)
    natives_dir.mkdir(parents=True, exist_ok=True)

    # 5. Write scripts/build.py
    build_py_content = """#!/usr/bin/env python3
import sys
from pathlib import Path
import solix_toolbox as tb

def main():
    profile = "debug"
    for arg in sys.argv[1:]:
        if arg in ("debug", "release", "test"):
            profile = arg
        elif arg.startswith("--profile="):
            profile = arg.split("=", 1)[1]
        elif arg == "--release":
            profile = "release"
        elif arg == "--test":
            profile = "test"
        elif arg == "--debug":
            profile = "debug"
    tb.auto_build(profile)

if __name__ == "__main__":
    main()
"""
    build_py_path = scripts_dir / "build.py"
    build_py_path.write_text(build_py_content, encoding="utf-8")
    build_py_path.chmod(0o755)

    # 6. Copy or write scripts/solix_toolbox.py
    current_toolbox = Path(__file__).resolve().parent / "solix_toolbox.py"
    if current_toolbox.exists():
        shutil.copy2(current_toolbox, scripts_dir / "solix_toolbox.py")
    else:
        # Fallback to templates/untitled-rust/scripts/solix_toolbox.py if available
        template_toolbox = Path(__file__).resolve().parent.parent / "templates" / "untitled-rust" / "scripts" / "solix_toolbox.py"
        if template_toolbox.exists():
            shutil.copy2(template_toolbox, scripts_dir / "solix_toolbox.py")

    # 7. Write src/main.slx or src/lib.slx
    pascal_name = to_pascal_case(project_name)
    snake_name = to_snake_case(project_name)

    if is_library:
        lib_slx_content = f"""package {snake_name};

public class {pascal_name} {{
    public static int32 version() {{
        return 1;
    }}
}}
"""
        (src_dir / "lib.slx").write_text(lib_slx_content, encoding="utf-8")
    else:
        main_slx_content = """class Main {
    public static int32 main() {
        return 0;
    }
}
"""
        (src_dir / "main.slx").write_text(main_slx_content, encoding="utf-8")

    # 8. Write tests/test_main.slx
    test_slx_content = """class TestRunner {
    public static int32 test() {
        return 0;
    }
}
"""
    (tests_dir / "test_main.slx").write_text(test_slx_content, encoding="utf-8")

    # 9. Write natives/ (Rust cdylib or empty keep)
    if no_native:
        (natives_dir / ".gitkeep").touch()
    else:
        bridge_name = f"{snake_name}_bridge"
        bridge_dir = natives_dir / bridge_name
        bridge_src_dir = bridge_dir / "src"
        bridge_src_dir.mkdir(parents=True, exist_ok=True)

        cargo_toml_content = f"""[package]
name = "{bridge_name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
"""
        (bridge_dir / "Cargo.toml").write_text(cargo_toml_content, encoding="utf-8")

        bridge_lib_content = f"""//! Rust native bridge extension crate for Solix package '{project_name}'.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {{
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}}

pub type NativeCFunctionPtr = unsafe extern "C" fn(
    *mut std::ffi::c_void,
    *const SolixCValue,
    u32,
    *mut SolixCValue,
) -> SolixCValue;

pub type RegisterNativeCallback = unsafe extern "C" fn(
    *mut std::ffi::c_void,
    *const std::ffi::c_char,
    NativeCFunctionPtr,
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn {pascal_name}Native_fast_add(
    _vm_context: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {{
    if arg_count < 2 || args.is_null() {{
        return SolixCValue {{
            tag: 0,
            flags: 0,
            payload: 0,
        }};
    }}
    let first = unsafe {{ (*args.add(0)).payload as i32 }};
    let second = unsafe {{ (*args.add(1)).payload as i32 }};
    SolixCValue {{
        tag: 2,
        flags: 0,
        payload: ((first + second) as i64) as u64,
    }}
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_register_natives(
    collector_context: *mut std::ffi::c_void,
    register_callback: RegisterNativeCallback,
) {{
    let add_symbol = c"{pascal_name}Native::fast_add";
    unsafe {{
        register_callback(
            collector_context,
            add_symbol.as_ptr(),
            {pascal_name}Native_fast_add,
        );
    }}
}}
"""
        (bridge_src_dir / "lib.rs").write_text(bridge_lib_content, encoding="utf-8")

    # 10. Placeholders for resources and lib
    (resources_dir / ".gitkeep").touch()
    (lib_dir / ".gitkeep").touch()

    print(f"Initialized Solix project '{project_name}' at '{target_path}'")
    return target_path


def parse_arguments() -> argparse.Namespace:
    """Parses command-line arguments for project scaffolding."""
    parser = argparse.ArgumentParser(
        description="Scaffold a new, ready-to-build Solix project."
    )
    parser.add_argument("path", help="Target directory for the new project")
    parser.add_argument("--name", help="Project name (default: directory name)")
    parser.add_argument("--version", default="0.1.0", help="Initial project version (default: 0.1.0)")
    parser.add_argument("--description", default="A Solix project", help="Project description")
    parser.add_argument("--license", default="Apache-2.0", help="License identifier (default: Apache-2.0)")
    parser.add_argument("--author", action="append", default=[], help="Author name (can be repeated)")
    parser.add_argument("--main", default="Main::main", help="Main entrypoint symbol (default: Main::main)")
    parser.add_argument("--test", default="TestRunner::test", help="Test entrypoint symbol (default: TestRunner::test)")
    parser.add_argument("--lib", action="store_true", help="Generate a library project instead of an application")
    parser.add_argument("--no-native", action="store_true", help="Do not create a Rust native cdylib bridge")
    parser.add_argument(
        "-d",
        "--deps",
        action="append",
        default=[],
        help="Dependency specification in pkg@ver format (can be repeated)",
    )
    parser.add_argument("--force", action="store_true", help="Overwrite existing manifest.toml")
    return parser.parse_args()


def main() -> None:
    args = parse_arguments()
    try:
        generate_project(
            target_path=Path(args.path),
            name=args.name,
            version=args.version,
            description=args.description,
            license_id=args.license,
            authors=args.author,
            main_entry=args.main,
            test_entry=args.test,
            is_library=args.lib,
            no_native=args.no_native,
            dependencies=args.deps,
            force=args.force,
        )
    except Exception as exc:
        print(f"Error: {exc}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
