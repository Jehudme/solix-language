#!/usr/bin/env python3
"""Solix Project Generator Script (`solix_scaffold.py`).

Programmatically scaffolds modern Solix projects configured with manifest.toml,
declarative [build] configuration, Python build automation, Solix source files,
test harnesses, and Rust cdylib bridges with hermetic build/.cache/cargo isolation.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import sys


DEFAULT_BUNDLED_SOLIX_FALLBACK_VERSION = "2.1.1"


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


def parse_bundled_package_version(manifest_json: str, package_name: str) -> str | None:
    """Extracts a package version from bundled_manifest.json content, preferring release."""
    try:
        parsed_payload = json.loads(manifest_json)
    except (json.JSONDecodeError, TypeError, ValueError):
        return None
    if not isinstance(parsed_payload, dict):
        return None
    packages_list = parsed_payload.get("packages")
    if not isinstance(packages_list, list):
        return None

    fallback_version: str | None = None
    for package_entry in packages_list:
        if not isinstance(package_entry, dict) or package_entry.get("name") != package_name:
            continue
        raw_version = package_entry.get("version")
        if not isinstance(raw_version, str):
            continue
        trimmed_version = raw_version.strip()
        if not trimmed_version:
            continue
        if package_entry.get("profile") == "release":
            return trimmed_version
        if fallback_version is None:
            fallback_version = trimmed_version

    return fallback_version


def resolve_bundled_package_version_from_dir(
    bundled_directory: Path, package_name: str
) -> str | None:
    """Resolves a bundled package version from bundled_manifest.json or staged subfolders."""
    manifest_file = bundled_directory / "bundled_manifest.json"
    if manifest_file.is_file():
        resolved = parse_bundled_package_version(
            manifest_file.read_text(encoding="utf-8"), package_name
        )
        if resolved:
            return resolved

    if not bundled_directory.is_dir():
        return None

    folder_prefix = f"{package_name}-"
    fallback_version: str | None = None
    for entry_path in sorted(bundled_directory.iterdir()):
        if not entry_path.is_dir():
            continue
        folder_name = entry_path.name
        if not folder_name.startswith(folder_prefix) or "@" not in folder_name:
            continue
        profile_part, version_part = folder_name[len(folder_prefix) :].split("@", 1)
        trimmed_version = version_part.strip()
        if not trimmed_version:
            continue
        if profile_part == "release":
            return trimmed_version
        if fallback_version is None:
            fallback_version = trimmed_version

    return fallback_version


def resolve_default_bundled_solix_version() -> str:
    """Resolves the active bundled `solix` library version independently of CLI crate version."""
    custom_dir_raw = os.environ.get("SOLIX_BUNDLED_PACKAGES_DIR", "").strip()
    if custom_dir_raw:
        custom_dir = Path(custom_dir_raw)
        if custom_dir.exists():
            resolved = resolve_bundled_package_version_from_dir(custom_dir, "solix")
            if resolved:
                return resolved

    assets_packages_dir = Path(__file__).resolve().parent.parent / "packages"
    if assets_packages_dir.exists():
        resolved = resolve_bundled_package_version_from_dir(assets_packages_dir, "solix")
        if resolved:
            return resolved

    return DEFAULT_BUNDLED_SOLIX_FALLBACK_VERSION


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
    no_std: bool = False,
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

    manifest_file = target_path / "manifest.yaml"
    if manifest_file.exists() and not force:
        raise FileExistsError(
            f"Target directory '{target_path}' already contains a Solix project (manifest.yaml exists). Use --force to overwrite."
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

    if not no_std and "solix" not in parsed_dependencies:
        parsed_dependencies["solix"] = resolve_default_bundled_solix_version()

    # 3. Write manifest.yaml
    if authors:
        authors_entries = ["\n    - " + "\n    - ".join(f'"{author_item}"' for author_item in authors)]
        authors_formatted = "".join(authors_entries)
    else:
        authors_formatted = " []"

    dependencies_block = ""
    if parsed_dependencies:
        dependency_lines = ["\n    dependencies:"]
        for dependency_name, dependency_version in sorted(parsed_dependencies.items()):
            dependency_lines.append(f"      {dependency_name}:")
            dependency_lines.append(f'        version: "{dependency_version}"')
            dependency_lines.append("        profile: release")
        dependencies_block = "\n".join(dependency_lines)

    default_release_platform = "universal" if no_native else "all"

    manifest_content = f"""configuration:
  name: {project_name}
  version: "{version}"
  description: "{description}"
  license: "{license_id}"
  authors:{authors_formatted}

profiles:
  release:
    entrypoint: {main_entry}
    optimization_level: 3
    inline_threshold: 16
    debug_symbols: false
    static: true
    platforms:
      - {default_release_platform}
    sources:
      - src{dependencies_block}

  debug:
    entrypoint: {main_entry}
    optimization_level: 0
    inline_threshold: 8
    debug_symbols: true
    static: false
    platforms:
      - host
    sources:
      - src{dependencies_block}

  test:
    entrypoint: {test_entry}
    optimization_level: 0
    inline_threshold: 8
    debug_symbols: true
    static: false
    platforms:
      - host
    sources:
      - src
      - tests{dependencies_block}
"""
    manifest_file.write_text(manifest_content, encoding="utf-8")

    # 4. Create directory layout
    scripts_dir = target_path / "scripts"
    src_dir = target_path / "src"
    tests_dir = target_path / "tests"
    resources_dir = target_path / "resources"
    bin_dir = target_path / "bin"
    natives_dir = target_path / "natives"

    scripts_dir.mkdir(parents=True, exist_ok=True)
    src_dir.mkdir(parents=True, exist_ok=True)
    tests_dir.mkdir(parents=True, exist_ok=True)
    resources_dir.mkdir(parents=True, exist_ok=True)
    bin_dir.mkdir(parents=True, exist_ok=True)
    natives_dir.mkdir(parents=True, exist_ok=True)

    # Pre-populate all target platform subdirectories under bin/
    for target_triple in (
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
    ):
        target_platform_dir = bin_dir / target_triple
        target_platform_dir.mkdir(parents=True, exist_ok=True)
        (target_platform_dir / ".gitkeep").touch()

    # 5. Write scripts/build.py
    build_py_content = """#!/usr/bin/env python3
import os
import sys
import solix_build as tb


def verify(ctx: tb.BuildContext) -> None:
    \"\"\"Pre-flight verification hook run before native or Solix compilation.\"\"\"
    # Register additional project-specific checks below, e.g.:
    # tb.require_tool("cmake", min_version="3.20.0")
    _ = ctx


def main() -> None:
    profile = os.environ.get("SOLIX_PROFILE", "").strip() or "release"
    verify_only = os.environ.get("SOLIX_VERIFY_ONLY", "").strip().lower() in ("1", "true", "yes")
    arguments = sys.argv[1:]
    index = 0
    while index < len(arguments):
        argument_item = arguments[index]
        if argument_item in ("debug", "release", "test"):
            profile = argument_item
        elif argument_item.startswith("--profile="):
            profile = argument_item.split("=", 1)[1]
        elif argument_item == "--profile" and index + 1 < len(arguments):
            profile = arguments[index + 1]
            index += 1
        elif argument_item.startswith("--target="):
            raw_target = argument_item.split("=", 1)[1]
            os.environ["SOLIX_TARGET_TRIPLE"] = (
                tb.get_host_target_triple() if raw_target.lower() == "host" else raw_target
            )
        elif argument_item == "--target" and index + 1 < len(arguments):
            raw_target = arguments[index + 1]
            os.environ["SOLIX_TARGET_TRIPLE"] = (
                tb.get_host_target_triple() if raw_target.lower() == "host" else raw_target
            )
            index += 1
        elif argument_item in ("--check-env", "--verify-only"):
            verify_only = True
        elif argument_item == "--release":
            profile = "release"
        elif argument_item == "--test":
            profile = "test"
        elif argument_item == "--debug":
            profile = "debug"
        index += 1
    tb.auto_build(profile, verify_fn=verify, verify_only=verify_only)


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
        bridge_name = f"__native_{snake_name}"
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
name = "{bridge_name}"
crate-type = ["cdylib"]

[dependencies]
solix-ffi = ">=1.1.3, <2.0.0"
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

    # 10. Placeholders for resources
    (resources_dir / ".gitkeep").touch()

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
    parser.add_argument(
        "--no-std",
        action="store_true",
        help="Do not add default solix standard library dependency",
    )
    parser.add_argument("--force", action="store_true", help="Overwrite existing manifest.yaml")
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
            no_std=arguments.no_std,
            dependencies=arguments.deps,
            force=arguments.force,
        )
    except Exception as runtime_error:
        print(f"Error: {runtime_error}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
