#!/usr/bin/env python3
"""End-to-End Multi-Platform Command & Option Lifecycle Matrix Test Suite (GL-0009).

Executes an exhaustive, hermetic developer lifecycle pipeline across Linux, macOS,
and Windows runners:
1. Project scaffolding: `solix new package` and `solix new native` (Rust and C).
2. Source formatting: `solix fmt` and `solix fmt --check`.
3. Documentation generation: `solix doc` (HTML and Markdown).
4. Project compilation & building: `solix build` across profiles (debug, release, test)
   and environment checks, including building `libraries/solixlib` under `--profile test`.
5. Virtual machine execution: `solix run` across all execution options (`--safe`,
   `--metrics`, `--heap-size`, `--max-stack`, `--cycle-collector`, `--verbose`, etc.).
6. Direct compilation: `solix compile source` and `solix compile package`.
7. Container inspection: `solix info` (`--disasm`, `--json`, `--api`).
8. Artifact distribution export: `solix export package` and `solix export binaries`.
9. Package store installation: `solix install package` and `solix install binaries`.
10. Package store query: `solix list` (`--kind`, `--profile`, `--filter`, `--sort`, `--json`).
11. Package store removal: `solix uninstall package` and `solix uninstall binaries`.
12. Workspace hygiene: `solix clean` (`--cache-only` and full clean).
13. Offline documentation portal: `solix wiki` (`--no-open`, `--path`, `--output`, `--search`).
14. Protocol daemon validation: `solix lsp --help` and `solix dap --help`.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile
import unittest


def resolve_solix_binary(explicit_path_argument: str | None) -> Path | None:
    """Resolves the executable path to the Solix CLI binary."""
    if explicit_path_argument:
        resolved_candidate = Path(explicit_path_argument).resolve()
        if resolved_candidate.is_file():
            return resolved_candidate
        return None

    environment_binary = os.environ.get("SOLIX_BIN")
    if environment_binary:
        resolved_candidate = Path(environment_binary).resolve()
        if resolved_candidate.is_file():
            return resolved_candidate

    repository_root = Path(__file__).resolve().parent.parent.parent
    executable_extension = ".exe" if platform.system() == "Windows" else ""
    for build_profile in ("release", "debug"):
        candidate_path = (
            repository_root / "target" / build_profile / f"solix{executable_extension}"
        )
        if candidate_path.is_file():
            return candidate_path

    system_candidate = shutil.which("solix")
    if system_candidate:
        return Path(system_candidate).resolve()

    return None


def execute_step(
    command_arguments: list[str],
    working_directory: Path,
    step_description: str,
    environment_variables: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    """Executes a command step and verifies that the process exited with code zero."""
    print(f"\n=== [LIFECYCLE STEP] {step_description} ===")
    print("+", " ".join(command_arguments))
    process_result = subprocess.run(
        command_arguments,
        cwd=working_directory,
        env=environment_variables,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if process_result.returncode != 0:
        print("--- STDOUT ---")
        print(process_result.stdout)
        print("--- STDERR ---")
        print(process_result.stderr)
        raise RuntimeError(
            f"Step '{step_description}' failed with exit code {process_result.returncode}"
        )
    return process_result


def execute_step_expecting_failure(
    command_arguments: list[str],
    working_directory: Path,
    step_description: str,
    environment_variables: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    """Executes a command step and verifies that the process exited with non-zero exit code."""
    print(f"\n=== [LIFECYCLE STEP: EXPECT FAILURE] {step_description} ===")
    print("+", " ".join(command_arguments))
    process_result = subprocess.run(
        command_arguments,
        cwd=working_directory,
        env=environment_variables,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if process_result.returncode == 0:
        print("--- STDOUT ---")
        print(process_result.stdout)
        print("--- STDERR ---")
        print(process_result.stderr)
        raise RuntimeError(
            f"Step '{step_description}' unexpectedly succeeded with exit code 0"
        )
    return process_result


def run_full_lifecycle_matrix(
    solix_binary_path: Path, repository_root: Path
) -> None:
    """Executes the complete hermetic CLI lifecycle matrix across all toolchain commands."""
    solix_executable_string = str(solix_binary_path.resolve())

    with tempfile.TemporaryDirectory(prefix="solix_lifecycle_") as temporary_workspace_directory:
        workspace_path = Path(temporary_workspace_directory).resolve()
        package_store_directory = workspace_path / "ephemeral_package_store"
        package_store_directory.mkdir(parents=True, exist_ok=True)
        isolated_home_directory = workspace_path / "ephemeral_solix_home"
        isolated_home_directory.mkdir(parents=True, exist_ok=True)

        execution_environment = dict(os.environ)
        execution_environment["SOLIX_HOME"] = str(isolated_home_directory)
        execution_environment["SOLIX_FORCE_EMBEDDED_ASSETS"] = "1"

        package_store_argument = str(package_store_directory)
        application_directory = workspace_path / "lifecycle_app"

        # ---------------------------------------------------------------------
        # Phase 1: Project Scaffolding (`solix new package` and `solix new native`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "new",
                "package",
                str(application_directory),
                "--name",
                "lifecycle_app",
                "--version",
                "1.0.0",
                "--description",
                "End-to-End Command Lifecycle Application",
                "--license",
                "Apache-2.0",
                "--author",
                "Solix Quality Team",
                "--no-native",
                "--no-std",
                "--force",
            ],
            working_directory=workspace_path,
            step_description="Scaffold application package via 'solix new package'",
            environment_variables=execution_environment,
        )

        manifest_file_path = application_directory / "manifest.yaml"
        if not manifest_file_path.is_file():
            raise FileNotFoundError(
                f"Expected manifest.yaml at {manifest_file_path} after project initialization"
            )

        # Attach dual-backend native libraries: Rust and CMake C
        execute_step(
            [
                solix_executable_string,
                "new",
                "native",
                "native_rust",
                "--project",
                str(application_directory),
                "--lang",
                "rust",
                "--profile",
                "release,debug,test",
                "--platforms",
                "host",
            ],
            working_directory=workspace_path,
            step_description="Scaffold Rust native library via 'solix new native native_rust'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "new",
                "native",
                "native_c",
                "--project",
                str(application_directory),
                "--lang",
                "c",
                "--profile",
                "release,debug,test",
                "--platforms",
                "host",
            ],
            working_directory=workspace_path,
            step_description="Scaffold C native library via 'solix new native native_c'",
            environment_variables=execution_environment,
        )

        # Write application entrypoints exercising both native bridges
        application_source_path = application_directory / "src" / "main.slx"
        application_source_path.write_text(
            """package lifecycle_app;

public class NativeRustNative {
    native public static int32 add(int32 left_value, int32 right_value);
}

public class NativeCNative {
    native public static int32 add(int32 left_value, int32 right_value);
}

public class Main {
    public static int32 main() {
        int32 rust_computed = NativeRustNative::add(15, 20);
        int32 c_computed = NativeCNative::add(3, 4);
        if (rust_computed + c_computed == 42) {
            return 0;
        }
        return 1;
    }
}
""",
            encoding="utf-8",
        )

        test_source_path = application_directory / "tests" / "test_main.slx"
        test_source_path.write_text(
            """package lifecycle_app.tests;

import lifecycle_app.NativeRustNative;
import lifecycle_app.NativeCNative;

public class TestRunner {
    public static int32 test() {
        int32 rust_computed = NativeRustNative::add(10, 10);
        int32 c_computed = NativeCNative::add(11, 11);
        if (rust_computed + c_computed == 42) {
            return 0;
        }
        return 1;
    }
}
""",
            encoding="utf-8",
        )

        # Also test legacy/simple root scaffolding invocation
        auxiliary_project_directory = workspace_path / "auxiliary_scaffold"
        execute_step(
            [
                solix_executable_string,
                "new",
                str(auxiliary_project_directory),
                "--name",
                "auxiliary_scaffold",
                "--no-native",
                "--no-std",
            ],
            working_directory=workspace_path,
            step_description="Scaffold simple auxiliary project via 'solix new'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 2: Source Code Formatting (`solix fmt`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "fmt",
                str(application_directory / "src"),
                str(application_directory / "tests"),
            ],
            working_directory=application_directory,
            step_description="Format source tree via 'solix fmt'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "fmt",
                "--check",
                str(application_directory / "src"),
                str(application_directory / "tests"),
            ],
            working_directory=application_directory,
            step_description="Verify formatting cleanliness via 'solix fmt --check'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 3: Documentation Generation (`solix doc`)
        # ---------------------------------------------------------------------
        documentation_html_directory = application_directory / "docs_output_html"
        documentation_markdown_directory = application_directory / "docs_output_markdown"

        execute_step(
            [
                solix_executable_string,
                "doc",
                str(application_directory),
                "--format",
                "html",
                "--output",
                str(documentation_html_directory),
            ],
            working_directory=application_directory,
            step_description="Generate HTML API documentation via 'solix doc --format html'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "doc",
                str(application_directory),
                "--format",
                "markdown",
                "--output",
                str(documentation_markdown_directory),
            ],
            working_directory=application_directory,
            step_description="Generate Markdown API documentation via 'solix doc --format markdown'",
            environment_variables=execution_environment,
        )

        if not (documentation_html_directory / "index.html").is_file():
            raise FileNotFoundError("Expected index.html in generated HTML documentation directory")

        # ---------------------------------------------------------------------
        # Phase 4: Build Matrix Across Profiles (`solix build`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "build",
                str(application_directory),
                "--check-env",
            ],
            working_directory=application_directory,
            step_description="Pre-flight toolchain environment verification via 'solix build --check-env'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "build",
                str(application_directory),
                "--profile",
                "debug",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=application_directory,
            step_description="Build application under debug profile via 'solix build --profile debug'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "build",
                str(application_directory),
                "--profile",
                "release",
                "--target",
                "host",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=application_directory,
            step_description="Build application under release profile via 'solix build --profile release --target host'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "build",
                str(application_directory),
                "--profile",
                "test",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=application_directory,
            step_description="Build application under test profile via 'solix build --profile test'",
            environment_variables=execution_environment,
        )

        # Test building solixlib standard library under test profile as mandated by user
        solixlib_directory = repository_root / "libraries" / "solixlib"
        if solixlib_directory.is_dir():
            execute_step(
                [
                    solix_executable_string,
                    "build",
                    str(solixlib_directory),
                    "--profile",
                    "test",
                    "--pkg-dir",
                    package_store_argument,
                ],
                working_directory=solixlib_directory,
                step_description="Build solixlib under test profile via 'solix build libraries/solixlib --profile test'",
                environment_variables=execution_environment,
            )

        # ---------------------------------------------------------------------
        # Phase 5: Virtual Machine Execution Matrix (`solix run`)
        # ---------------------------------------------------------------------
        release_package_path = application_directory / "build" / "release" / "lifecycle_app.slxp"
        test_package_path = application_directory / "build" / "test" / "lifecycle_app.slxp"

        # Baseline execution of built release package
        execute_step(
            [
                solix_executable_string,
                "run",
                "package",
                str(release_package_path),
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=application_directory,
            step_description="Execute release package via 'solix run package'",
            environment_variables=execution_environment,
        )

        # Comprehensive runtime options verification
        execute_step(
            [
                solix_executable_string,
                "run",
                "package",
                str(release_package_path),
                "--pkg-dir",
                package_store_argument,
                "--safe",
                "--metrics",
                "text",
                "--heap-size",
                "16MB",
                "--max-stack",
                "512",
                "--cycle-collector",
                "on",
                "--cycle-threshold",
                "2048",
                "--cycle-budget-us",
                "10000",
                "--verbose",
                "--no-verify",
                "--",
                "sample_argument_first",
                "sample_argument_second",
            ],
            working_directory=application_directory,
            step_description="Execute release package with exhaustive CLI runtime flags and options",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "run",
                "package",
                str(release_package_path),
                "--pkg-dir",
                package_store_argument,
                "--metrics",
                "json",
                "--cycle-collector",
                "off",
            ],
            working_directory=application_directory,
            step_description="Execute release package with JSON metrics and cycle collector disabled",
            environment_variables=execution_environment,
        )

        # Execute test profile package with custom entrypoint symbol
        execute_step(
            [
                solix_executable_string,
                "run",
                "package",
                str(test_package_path),
                "--entry",
                "TestRunner::test",
                "--pkg-dir",
                package_store_argument,
                "--verbose",
            ],
            working_directory=application_directory,
            step_description="Execute test package with explicit --entry TestRunner::test",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 6: Compilation Subcommands (`solix compile` variations)
        # ---------------------------------------------------------------------
        standalone_source_file = workspace_path / "standalone_sample.slx"
        standalone_source_file.write_text(
            """package standalone_sample;

public class Main {
    public static int32 main() {
        int32 computed_accumulator = 0;
        int32 iteration_counter = 0;
        while (iteration_counter < 10) {
            computed_accumulator = computed_accumulator + iteration_counter;
            iteration_counter = iteration_counter + 1;
        }
        if (computed_accumulator == 45) {
            return 0;
        }
        return 1;
    }
}
""",
            encoding="utf-8",
        )

        compiled_standalone_binary = workspace_path / "standalone_sample.slxb"
        emitted_api_header_file = workspace_path / "emitted_api_interface.slx"

        execute_step(
            [
                solix_executable_string,
                "compile",
                "source",
                str(standalone_source_file),
                "--output",
                str(compiled_standalone_binary),
                "--time",
                "--metrics",
                "text",
                "--emit-api",
                str(emitted_api_header_file),
                "--inline-threshold",
                "10",
                "--link",
                "static",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="Compile standalone source via 'solix compile source' with metrics and API emission",
            environment_variables=execution_environment,
        )

        if not compiled_standalone_binary.is_file():
            raise FileNotFoundError(
                f"Compiled binary missing at {compiled_standalone_binary}"
            )
        if not emitted_api_header_file.is_file():
            raise FileNotFoundError(
                f"Emitted API interface missing at {emitted_api_header_file}"
            )

        # Syntax check only without emitting bytecode
        execute_step(
            [
                solix_executable_string,
                "compile",
                "source",
                str(standalone_source_file),
                "--check",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="Syntax and semantic validation via 'solix compile source --check'",
            environment_variables=execution_environment,
        )

        # Compile package sources directly into binary container
        compiled_package_binary = workspace_path / "compiled_package_binaries.slxb"
        execute_step(
            [
                solix_executable_string,
                "compile",
                "package",
                str(application_directory),
                "--output",
                str(compiled_package_binary),
                "--profile",
                "release",
                "--time",
                "--metrics",
                "json",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=application_directory,
            step_description="Compile package sources via 'solix compile package'",
            environment_variables=execution_environment,
        )

        if not compiled_package_binary.is_file():
            raise FileNotFoundError(
                f"Compiled package binary missing at {compiled_package_binary}"
            )

        # Syntax check package sources
        execute_step(
            [
                solix_executable_string,
                "compile",
                "package",
                str(application_directory),
                "--check",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=application_directory,
            step_description="Syntax and semantic validation via 'solix compile package --check'",
            environment_variables=execution_environment,
        )

        # Run compiled standalone binary
        execute_step(
            [
                solix_executable_string,
                "run",
                str(compiled_standalone_binary),
                "--pkg-dir",
                package_store_argument,
                "--verbose",
            ],
            working_directory=workspace_path,
            step_description="Execute compiled standalone binary via 'solix run <binary.slxb>'",
            environment_variables=execution_environment,
        )

        # Run source on-the-fly via `solix run source`
        execute_step(
            [
                solix_executable_string,
                "run",
                "source",
                str(standalone_source_file),
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="Compile and execute source on-the-fly via 'solix run source'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 7: Binary Container Inspection (`solix info`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "info",
                str(compiled_standalone_binary),
                "--disasm",
            ],
            working_directory=workspace_path,
            step_description="Disassemble binary container bytecode via 'solix info --disasm'",
            environment_variables=execution_environment,
        )

        info_json_result = execute_step(
            [
                solix_executable_string,
                "info",
                str(compiled_standalone_binary),
                "--json",
            ],
            working_directory=workspace_path,
            step_description="Inspect binary container metadata JSON via 'solix info --json'",
            environment_variables=execution_environment,
        )
        parsed_info_json = json.loads(info_json_result.stdout)
        if "functions_count" not in parsed_info_json:
            raise ValueError(
                f"Malformed JSON output from 'solix info --json': {info_json_result.stdout}"
            )

        execute_step(
            [
                solix_executable_string,
                "info",
                str(compiled_standalone_binary),
                "--api",
            ],
            working_directory=workspace_path,
            step_description="Synthesize API interface declarations via 'solix info --api'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "info",
                str(release_package_path),
                "--json",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="Inspect package archive container metadata via 'solix info <package.slxp> --json'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 8: Package & Binary Export (`solix export`)
        # ---------------------------------------------------------------------
        exported_package_archive = workspace_path / "exported_lifecycle_app.slxp"
        exported_standalone_binary = workspace_path / "exported_lifecycle_app.slxb"

        execute_step(
            [
                solix_executable_string,
                "export",
                "package",
                str(application_directory),
                "--output",
                str(exported_package_archive),
                "--profile",
                "debug",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=application_directory,
            step_description="Export packaged bundle via 'solix export package'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "export",
                "binaries",
                str(application_directory),
                "--output",
                str(exported_standalone_binary),
                "--profile",
                "debug",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=application_directory,
            step_description="Export compiled binary container via 'solix export binaries'",
            environment_variables=execution_environment,
        )

        if not exported_package_archive.is_file():
            raise FileNotFoundError(
                f"Exported package archive missing at {exported_package_archive}"
            )
        if not exported_standalone_binary.is_file():
            raise FileNotFoundError(
                f"Exported binary container missing at {exported_standalone_binary}"
            )

        # ---------------------------------------------------------------------
        # Phase 9: Package Store Installation (`solix install`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "install",
                "package",
                str(exported_package_archive),
                "--force",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="Install package archive into store via 'solix install package'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "install",
                "binaries",
                str(exported_standalone_binary),
                "--force",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="Install binary container into store via 'solix install binaries'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 10: Store Listing Queries (`solix list`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "list",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="List installed packages via 'solix list'",
            environment_variables=execution_environment,
        )

        list_json_result = execute_step(
            [
                solix_executable_string,
                "list",
                "--pkg-dir",
                package_store_argument,
                "--json",
            ],
            working_directory=workspace_path,
            step_description="Query installed store inventory via 'solix list --json'",
            environment_variables=execution_environment,
        )
        parsed_store_inventory = json.loads(list_json_result.stdout)
        if not isinstance(parsed_store_inventory, list) or len(parsed_store_inventory) == 0:
            raise ValueError(
                f"Expected non-empty list from 'solix list --json', got: {list_json_result.stdout}"
            )

        execute_step(
            [
                solix_executable_string,
                "list",
                "--pkg-dir",
                package_store_argument,
                "--kind",
                "package",
            ],
            working_directory=workspace_path,
            step_description="Filter store inventory by kind via 'solix list --kind package'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "list",
                "--pkg-dir",
                package_store_argument,
                "--kind",
                "binaries",
            ],
            working_directory=workspace_path,
            step_description="Filter store inventory by kind via 'solix list --kind binaries'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "list",
                "--pkg-dir",
                package_store_argument,
                "--filter",
                "lifecycle_app",
                "--sort",
                "name",
                "--reverse",
            ],
            working_directory=workspace_path,
            step_description="Filter and reverse-sort store inventory via 'solix list --filter ... --sort name --reverse'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 11: Store Removal (`solix uninstall`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "uninstall",
                "package",
                "lifecycle_app@1.0.0",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="Uninstall package from store via 'solix uninstall package'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "uninstall",
                "binaries",
                "lifecycle_app",
                "--pkg-dir",
                package_store_argument,
            ],
            working_directory=workspace_path,
            step_description="Uninstall binary from store via 'solix uninstall binaries'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 12: Offline Documentation Portal (`solix wiki`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "wiki",
                "--no-open",
                "--path",
            ],
            working_directory=workspace_path,
            step_description="Resolve offline documentation index entrypoint via 'solix wiki --no-open --path'",
            environment_variables=execution_environment,
        )

        wiki_export_directory = workspace_path / "exported_offline_wiki"
        execute_step(
            [
                solix_executable_string,
                "wiki",
                "--output",
                str(wiki_export_directory),
                "--no-open",
            ],
            working_directory=workspace_path,
            step_description="Export offline documentation portal via 'solix wiki --output <dir> --no-open'",
            environment_variables=execution_environment,
        )

        if not (wiki_export_directory / "index.html").is_file():
            raise FileNotFoundError(
                f"Expected index.html inside exported wiki directory {wiki_export_directory}"
            )

        execute_step(
            [
                solix_executable_string,
                "wiki",
                "--search",
                "installation",
                "--no-open",
            ],
            working_directory=workspace_path,
            step_description="Query offline documentation portal via 'solix wiki --search ... --no-open'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 13: Project Cache & Artifact Cleaning (`solix clean`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "clean",
                str(application_directory),
                "--cache-only",
            ],
            working_directory=application_directory,
            step_description="Clean intermediate build caches via 'solix clean --cache-only'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "clean",
                str(application_directory),
            ],
            working_directory=application_directory,
            step_description="Clean entire project build output via 'solix clean'",
            environment_variables=execution_environment,
        )

        # ---------------------------------------------------------------------
        # Phase 14: Daemon Protocols (`solix lsp` and `solix dap`)
        # ---------------------------------------------------------------------
        execute_step(
            [
                solix_executable_string,
                "lsp",
                "--help",
            ],
            working_directory=workspace_path,
            step_description="Verify LSP server command definition via 'solix lsp --help'",
            environment_variables=execution_environment,
        )

        execute_step(
            [
                solix_executable_string,
                "dap",
                "--help",
            ],
            working_directory=workspace_path,
            step_description="Verify DAP debugger command definition via 'solix dap --help'",
            environment_variables=execution_environment,
        )

    print("\n=======================================================")
    print("ALL 14 SOLIX CLI COMMAND LIFECYCLE PHASES PASSED CLEANLY")
    print("=======================================================")


class CliLifecycleMatrixTestCase(unittest.TestCase):
    """Unit test runner wrapper for the CLI lifecycle matrix."""

    def test_full_command_lifecycle_matrix(self) -> None:
        repository_root = Path(__file__).resolve().parent.parent.parent
        resolved_binary_path = resolve_solix_binary(explicit_path_argument=None)
        if resolved_binary_path is None:
            raise unittest.SkipTest(
                "Solix toolchain binary not found; skipping CLI command lifecycle matrix."
            )
        run_full_lifecycle_matrix(
            solix_binary_path=resolved_binary_path,
            repository_root=repository_root,
        )


def main() -> int:
    """CLI entrypoint for standalone test matrix execution."""
    parser = argparse.ArgumentParser(
        description="Solix CLI Command & Option Lifecycle Matrix Test Suite (GL-0009)"
    )
    parser.add_argument(
        "--solix-binary",
        dest="solix_binary",
        default=None,
        help="Path to the compiled solix toolchain executable binary",
    )
    argument_values = parser.parse_args()

    repository_root = Path(__file__).resolve().parent.parent.parent
    resolved_binary = resolve_solix_binary(argument_values.solix_binary)
    if resolved_binary is None:
        print(
            "ERROR: Solix binary could not be resolved. Please specify --solix-binary <path>.",
            file=sys.stderr,
        )
        return 1

    try:
        run_full_lifecycle_matrix(
            solix_binary_path=resolved_binary,
            repository_root=repository_root,
        )
        return 0
    except Exception as execution_error:
        print(f"\nFATAL: Lifecycle matrix failed: {execution_error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
