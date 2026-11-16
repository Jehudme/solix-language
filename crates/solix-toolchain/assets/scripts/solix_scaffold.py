#!/usr/bin/env python3
"""Solix Project Generator Script (`solix_scaffold.py`).

Programmatically scaffolds modern Solix projects configured with manifest.toml,
declarative [build] configuration, Python build automation, Solix source files,
test harnesses, and Rust cdylib bridges with hermetic build/.cache/cargo isolation.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import shutil
import sys


def to_pascal_case(raw_name: str) -> str:
    """Converts a snake_case, kebab-case, or space-separated name to PascalCase."""
    words = re.split(r"[^a-zA-Z0-9]+", raw_name)
    cleaned = "".join(word_item.capitalize() for word_item in words if word_item)
    return cleaned if cleaned else "App"


def to_snake_case(raw_name: str) -> str:
    """Converts a name to a safe snake_case identifier."""
    words = re.split(r"[^a-zA-Z0-9]+", raw_name)
    cleaned = "_".join(word_item.lower() for word_item in words if word_item)
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
        "/build/",
        "*.slxb",
        "*.slxarc",
        "",
    ])
    (target_path / ".gitignore").write_text(gitignore_content, encoding="utf-8")

    # 2. Parse dependencies into key-value pairs
    parsed_dependencies: dict[str, str] = {}
    for dependency_item in dependencies:
        if "@" in dependency_item:
            dependency_name, dependency_version = dependency_item.split("@", 1)
        elif ":" in dependency_item:
            dependency_name, dependency_version = dependency_item.split(":", 1)
        elif "=" in dependency_item:
            dependency_name, dependency_version = dependency_item.split("=", 1)
        else:
            dependency_name, dependency_version = dependency_item, "*"
        parsed_dependencies[dependency_name.strip()] = dependency_version.strip()

    # 3. Write manifest.toml
    authors_formatted = ", ".join(f'"{author_item}"' for author_item in authors)
    deps_lines = []
    for dependency_name, dependency_version in sorted(parsed_dependencies.items()):
        deps_lines.append(f'{dependency_name} = {{ version = "{dependency_version}", profile = "release" }}')
    
    release_deps_block = ""
    if deps_lines:
        release_deps_block = "\n[profiles.release.dependencies]\n" + "\n".join(deps_lines)

    manifest_content = f"""[configuration]
name = "{project_name}"
version = "{version}"
description = "{description}"
license = "{license_id}"
authors = [{authors_formatted}]

[profiles.release]
entrypoint = "{main_entry}"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]{release_deps_block}

[profiles.debug]
inherits = "release"
optimization_level = 0
inline_threshold = 8
debug_symbols = true
static = false

[profiles.test]
inherits = "debug"
entrypoint = "{test_entry}"
sources = ["src", "tests"]
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
import solix_build as tb

def main():
    profile = "debug"
    for argument_item in sys.argv[1:]:
        if argument_item in ("debug", "release", "test"):
            profile = argument_item
        elif argument_item.startswith("--profile="):
            profile = argument_item.split("=", 1)[1]
        elif argument_item == "--release":
            profile = "release"
        elif argument_item == "--test":
            profile = "test"
        elif argument_item == "--debug":
            profile = "debug"
    tb.auto_build(profile)

if __name__ == "__main__":
    tb.run_cli(main)
"""
    build_py_path = scripts_dir / "build.py"
    build_py_path.write_text(build_py_content, encoding="utf-8")
    build_py_path.chmod(0o755)

    # 6. Copy or write scripts/solix_build.py
    current_toolbox = Path(__file__).resolve().parent / "solix_build.py"
    if current_toolbox.exists():
        shutil.copy2(current_toolbox, scripts_dir / "solix_build.py")
    else:
        # Fallback to templates/untitled-rust/scripts/solix_build.py if available
        template_toolbox = Path(__file__).resolve().parent.parent / "templates" / "untitled-rust" / "scripts" / "solix_build.py"
        if template_toolbox.exists():
            shutil.copy2(template_toolbox, scripts_dir / "solix_build.py")

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

    # 9. Write natives/ (Rust cdylib with .cargo/config.toml or empty keep)
    if no_native:
        (natives_dir / ".gitkeep").touch()
    else:
        bridge_name = f"{snake_name}_bridge"
        bridge_dir = natives_dir / bridge_name
        bridge_src_dir = bridge_dir / "src"
        bridge_src_dir.mkdir(parents=True, exist_ok=True)

        cargo_config_dir = bridge_dir / ".cargo"
        cargo_config_dir.mkdir(parents=True, exist_ok=True)
        cargo_config_content = """[build]
target-dir = "../../build/.cache/cargo"
"""
        (cargo_config_dir / "config.toml").write_text(cargo_config_content, encoding="utf-8")

        cargo_toml_content = f"""[package]
name = "{bridge_name}"
version = "0.1.0"
edition = "2024"

[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
solix-ffi = ">=1.1.3, <=1.2.0"
"""
        (bridge_dir / "Cargo.toml").write_text(cargo_toml_content, encoding="utf-8")

        bridge_lib_content = f"""//! Rust native bridge extension crate for Solix package '{project_name}'.

use solix_ffi::{{NativeRegistry, SolixCValue, solix_export}};

pub unsafe extern "C" fn {pascal_name}Native_fast_add(
    _vm_context: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {{
    if arg_count < 2 || args.is_null() {{
        return SolixCValue::null();
    }}
    let first = unsafe {{ (*args.add(0)).payload as i32 }};
    let second = unsafe {{ (*args.add(1)).payload as i32 }};
    SolixCValue::from_i32(first + second)
}}

solix_export!(
    "{pascal_name}Native::fast_add" => {pascal_name}Native_fast_add
);
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
    parser.add_argument("--native", action="store_true", help="Explicitly scaffold a Rust native cdylib bridge")
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
    arguments = parse_arguments()
    try:
        generate_project(
            target_path=Path(arguments.path),
            name=arguments.name,
            version=arguments.version,
            description=arguments.description,
            license_id=arguments.license,
            authors=arguments.author,
            main_entry=arguments.main,
            test_entry=arguments.test,
            is_library=arguments.lib,
            no_native=arguments.no_native,
            dependencies=arguments.deps,
            force=arguments.force,
        )
    except Exception as runtime_error:
        print(f"Error: {runtime_error}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
