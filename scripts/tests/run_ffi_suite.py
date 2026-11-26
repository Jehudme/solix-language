#!/usr/bin/env python3
"""End-to-end FFI and dual-backend build/runtime verification runner for Solix.

Executes across Linux, macOS, and Windows:
1. Standalone C11 & C++17 CTest suite in `ffi/c`.
2. Rust `solix-ffi` unit tests in `ffi/rust`.
3. Standalone native unit tests in `examples/ffi_dual_backend/natives/{cmake_backend,rust_backend}`.
4. Dual-backend Solix build and VM execution on `examples/ffi_dual_backend`
   (`--profile cmake_backend`, `--profile rust_backend`, and `--profile release`).
5. Fresh `solix new` + `solix new native` (C, C++, Rust) standalone + combined Solix smoke tests.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def run_step(command: list[str], cwd: Path, label: str, env: dict[str, str] | None = None) -> None:
    """Executes a verification step and exits immediately with diagnostics on failure."""
    print(f"\n=== [{label}] ===")
    print("+", " ".join(command))
    proc = subprocess.run(command, cwd=cwd, env=env, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"FAIL [{label}] exited with code {proc.returncode}")


def resolve_solix_binary(repo_root: Path, explicit: str | None) -> list[str]:
    """Resolves the `solix` CLI binary command."""
    if explicit:
        return [explicit]
    if os.environ.get("SOLIX_BIN"):
        return [os.environ["SOLIX_BIN"]]

    exe_suffix = ".exe" if os.name == "nt" else ""
    for profile in ("debug", "release"):
        candidate = repo_root / "target" / profile / f"solix{exe_suffix}"
        if candidate.is_file():
            return [str(candidate)]
    return ["cargo", "run", "--bin", "solix", "--"]


def run_cmake_ctest(source_dir: Path, build_dir: Path, label: str) -> None:
    """Configures, builds, and runs CTest on a CMake project in standalone mode."""
    run_step(
        [
            "cmake",
            "-S",
            str(source_dir),
            "-B",
            str(build_dir),
            "-DBUILD_TESTING=ON",
            "-DSOLIX_NATIVE_BUILD=OFF",
        ],
        cwd=source_dir,
        label=f"{label}: cmake configure",
    )
    run_step(
        ["cmake", "--build", str(build_dir), "--config", "Release", "--parallel"],
        cwd=source_dir,
        label=f"{label}: cmake build",
    )
    run_step(
        ["ctest", "--test-dir", str(build_dir), "-C", "Release", "--output-on-failure"],
        cwd=source_dir,
        label=f"{label}: ctest",
    )


def verify_new_native_smoke(
    repo_root: Path,
    solix_cmd: list[str],
    scratch_root: Path,
    standalone_only: bool = False,
) -> None:
    """Scaffolds a fresh project with C, C++, and Rust native modules and runs verification."""
    app_dir = scratch_root / "smoke_polyglot_app"
    run_step(
        [*solix_cmd, "new", str(app_dir), "--no-native"],
        cwd=repo_root,
        label="smoke: solix new --no-native",
    )

    for module_name, backend_lang in [("mod_c", "c"), ("mod_cpp", "cpp"), ("mod_rust", "rust")]:
        run_step(
            [
                *solix_cmd,
                "new",
                "native",
                module_name,
                "--project",
                str(app_dir),
                "--lang",
                backend_lang,
                "--platforms",
                "host",
            ],
            cwd=repo_root,
            label=f"smoke: solix new native {module_name} ({backend_lang})",
        )

    # Verify standalone builds/tests without Solix plugin mode
    for c_mod in ("mod_c", "mod_cpp"):
        run_cmake_ctest(
            app_dir / "natives" / c_mod,
            scratch_root / f"standalone_{c_mod}",
            f"smoke standalone {c_mod}",
        )
    run_step(
        [
            "cargo",
            "test",
            "--manifest-path",
            str(app_dir / "natives" / "mod_rust" / "Cargo.toml"),
        ],
        cwd=repo_root,
        label="smoke standalone mod_rust: cargo test",
    )

    if standalone_only:
        return

    # Wire Solix main.slx calling all 3 generated native modules simultaneously
    (app_dir / "src" / "main.slx").write_text(
        """package smoke_polyglot_app;

public class ModCNative {
    native public static int32 add(int32 left_val, int32 right_val);
}

public class ModCppNative {
    native public static int32 add(int32 left_val, int32 right_val);
}

public class ModRustNative {
    native public static int32 add(int32 left_val, int32 right_val);
}

public class Main {
    public static int32 main() {
        int32 total = ModCNative::add(10, 4)
            + ModCppNative::add(12, 2)
            + ModRustNative::add(9, 5);
        if (total != 42) {
            return 1;
        }
        return 0;
    }
}
""",
        encoding="utf-8",
    )

    run_step(
        [*solix_cmd, "build", str(app_dir), "--release"],
        cwd=repo_root,
        label="smoke: solix build combined C/C++/Rust",
    )
    run_step(
        [*solix_cmd, "run", str(app_dir / "build" / "release" / "smoke_polyglot_app.slxarc")],
        cwd=repo_root,
        label="smoke: solix run combined C/C++/Rust",
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Run full Solix C/C++/Rust FFI verification suite.")
    parser.add_argument(
        "--solix-binary",
        type=str,
        default=None,
        help="Path to the solix CLI binary to invoke.",
    )
    parser.add_argument(
        "--standalone-only",
        action="store_true",
        help="Run host standalone C/C++/Rust FFI and scaffold tests without Docker plugin builds.",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent.parent
    solix_cmd = resolve_solix_binary(repo_root, args.solix_binary)

    with tempfile.TemporaryDirectory(prefix="solix-ffi-suite-", ignore_cleanup_errors=True) as tmp_dir:
        scratch = Path(tmp_dir)

        # 1. C/C++ CMake SDK CTest
        run_cmake_ctest(repo_root / "ffi" / "c", scratch / "ffi_c_build", "ffi/c")

        # 2. Rust solix-ffi crate tests
        run_step(
            ["cargo", "test", "--manifest-path", str(repo_root / "ffi" / "rust" / "Cargo.toml")],
            cwd=repo_root,
            label="ffi/rust: cargo test",
        )

        # 3. Dual-backend example standalone native unit tests
        dual_example_dir = repo_root / "examples" / "ffi_dual_backend"
        run_cmake_ctest(
            dual_example_dir / "natives" / "cmake_backend",
            scratch / "example_cmake_standalone",
            "example cmake_backend standalone",
        )
        run_step(
            [
                "cargo",
                "test",
                "--manifest-path",
                str(dual_example_dir / "natives" / "rust_backend" / "Cargo.toml"),
            ],
            cwd=repo_root,
            label="example rust_backend standalone: cargo test",
        )

        # 4. Dual-backend example Solix build & execution for all profiles (Docker hosts)
        if not args.standalone_only:
            for profile in ("cmake_backend", "rust_backend", "release"):
                run_step(
                    [
                        *solix_cmd,
                        "build",
                        str(dual_example_dir),
                        "--profile",
                        profile,
                        "--target",
                        "host",
                    ],
                    cwd=repo_root,
                    label=f"examples/ffi_dual_backend: build [{profile}]",
                )
                archive_path = dual_example_dir / "build" / profile / "ffi_dual_backend.slxarc"
                run_step(
                    [*solix_cmd, "run", str(archive_path)],
                    cwd=repo_root,
                    label=f"examples/ffi_dual_backend: run [{profile}]",
                )

        # 5. `solix new` + `solix new native` cross-language dual-mode smoke test
        verify_new_native_smoke(repo_root, solix_cmd, scratch, standalone_only=args.standalone_only)

    print("\nALL FFI C/C++/Rust AND DUAL-BACKEND VERIFICATION CHECKS PASSED.")


if __name__ == "__main__":
    main()
