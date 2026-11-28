# `solix run` — Virtual Machine Execution

The `run` subcommand executes Solix bytecode binary containers (`*.slxb`), application packages (`*.slxp`), or compiles and runs raw source files (`*.slx`) on-the-fly in the 64-bit Register Virtual Machine.

---

## Synopsis

```bash
solix run package [target] [options] [-- [guest_args...]]
solix run source <sources...> [options] [-- [guest_args...]]
solix run <binaries...> [options] [-- [guest_args...]]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<target>` | Path | **Yes** (for `package`) | Path to a Solix project root directory or `.slxp` package archive. |
| `<sources...>` | Paths | **Yes** (for `source`) | One or more `.slx` source files to compile in-memory and execute directly. |
| `<binaries...>` | Paths | **Yes** (for direct invocation) | One or more `.slxb` binary containers, `.slxp` packages, or `.slx` source files to execute. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--lib <path>` | `-l` | Path (Repeated) | `[]` | Additional library files or native shared libraries to load (`*.slxb`, `*.slxp`, `*.so`, `*.dll`, `*.dylib`). |
| `--entry <symbol>` | `-e` | String | None | Overrides default entry point symbol (e.g. `Main::main`). If the specified entrypoint symbol does not exist in the loaded program or archive container, execution halts immediately with diagnostic code `[E0301]`. |
| `--profile <profile>` | `-p` | String | None | Target build profile to execute (e.g. `release`, `debug`, `test`). |
| `--inline-threshold <N>` | | Integer | `8` | Maximum statement threshold for AST inlining when compiling `.slx` sources. |
| `--heap-size <size>` | | String | `"32MB"` | Virtual heap allocation capacity limit (e.g., `"16MB"`, `"64MB"`, `"1GB"`). |
| `--max-stack <depth>` | | Integer | `1024` | Maximum call stack frame depth limit. |
| `--cycle-collector <mode>` | | String | `"on"` | Controls the Bacon & Rajan cycle collector (`on`/`true`/`1` or `off`/`false`/`0`). |
| `--cycle-threshold <count>` | | Integer | `1024` | Potential-cycle candidate buffer threshold triggering background collection (must be `>= 1`, otherwise halts with `[E0341]`). |
| `--cycle-budget-us <us>` | | Integer | None | Optional per-pause microsecond time budget for incremental cycle collection steps. |
| `--verbose` | `-v` | Flag | `false` | Prints VM execution statistics and memory report upon process completion. |
| `--safe` | | Flag | `false` | Executes in safe mode with defensive invariant validation and strict bounds checking. |
| `--jit <mode>` | | String | `"tiered"` | Controls the Cranelift JIT compiler (`tiered`, `eager`, or `off`). Target-gated at compile time via `solix-runtime`'s `jit` feature (`default = ["jit"]`), automatically disabled on `wasm32` and non-`x86_64`/`aarch64` targets. |
| `--metrics [<format>]` | | Optional String | None (`text` if flag only) | Prints runtime execution telemetry and memory statistics formatted as `text` or `json` (enables per-instruction telemetry tracking via `RuntimeOptions::with_metrics_tracking(true)`, includes `jit_compiled_functions` and `jit_compile_time_ns`, and reports guest-only `active_guest_objects()` leaks excluding immortal constant-pool strings). |
| `--no-verify` | | Flag | `false` | Bypasses dependency and native library cryptographic SHA-256 checksum verification (visible alias `--ignore-checksum`). |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for resolving package dependencies. |
| `-- [guest_args...]` | | Strings | `[]` | Arguments passed directly to the guest application entry point. |

> [!TIP]
> Direct invocations such as `solix run path/to/file.slx` or `file.slxb` use a zero-overhead CLI fast path that bypasses subcommand augmentation and defers bundled package store bootstrapping and ancestor `manifest.yaml` probes until imports or package references require them.

---

## Execution Modes

### 1. Direct Source Execution (`*.slx`)

When given `.slx` source files, `solix run` compiles them on-the-fly in memory without writing `.slxb` to disk, resolving dependencies and executing immediately:

```bash
solix run src/main.slx
```

### 2. Standalone Binary Container (`*.slxb`)

When given a compiled `.slxb` container, the VM decodes the binary, initializes the register execution context, and invokes the entry point (`Main::main` by default, or overridden by `-e`):

```bash
solix run bin/app.slxb
```

### 3. Package Container (`*.slxp`)

When given a package container, the runtime:
1. Inspects embedded `manifest.yaml` (or legacy `manifest.toml`).
2. Automatically stages and loads bundled companion native shared libraries matching `__native_*` (or `lib__native_*`) in `bin/<target_triple>/*` (with fallback to `lib/<target_triple>/*`).
3. Sets `SOLIX_RESOURCE_ROOT` to the package resources directory.
4. Selects the target entry point using strict precedence:
   - Explicit `--entry <symbol>` override on the CLI (validating symbol existence, throwing `[E0301]` on missing entrypoint).
   - Explicit `--profile <profile>` (or package spec profile such as `pkg:test`), resolving `profiles.<profile>.entrypoint` from the bundled manifest.
   - The compiled entry point recorded in the root `binaries.slxb` container (`entry_point_id` / `entry_point`), so packages built with `--test` or custom profiles execute their built entrypoint without extra CLI flags.
   - Fallback to `profiles.<build_profile>.entrypoint` or `profiles.release.entrypoint` when no entry point is embedded in the binary.
5. Invokes the entry point in the VM.

```bash
# Run application
solix run build/release/my_app.slxp

# Run test profile container
solix run build/test/my_app.slxp
```

### 4. Installed Package Execution

When given a package name or specification instead of a file path, `solix run` resolves and executes the package from the local package store (`~/.solix/packages`):

```bash
# Run latest installed version (default profile: release)
solix run my-app

# Run specific version
solix run my-app@1.2.0

# Run specific profile using colon notation
solix run my-app:debug@1.2.0
solix run my-app:release

# Run with explicit --profile flag
solix run my-app --profile debug
```

---

## Passing Guest Arguments

All arguments placed after `--` are passed directly into the guest Solix program:

```bash
solix run build/release/my_app.slxp -- --port 8080 --host 0.0.0.0
```

---

## Exit Codes & Process Propagation

The VM propagates the return code of the Solix entry point method directly to the operating system shell:

```solix
class Main {
    public static int32 main() {
        return 42; // Process exits with code 42
    }
}
```

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Program executed cleanly and entry point returned 0. |
| `N` (`N > 0`) | Guest Exit | Non-zero exit code explicitly returned by the program entry point. |
| `1` | VM / Runtime Fault | Unhandled exception, memory limit exceeded, or container loading error. |
