# `solix new` — Project Scaffolding

The `new` subcommand creates and initializes a complete Solix project workspace configured with `manifest.toml`, Python build automation scripts, starter Solix source files, an automated test harness, and an optional Rust native bridge (`cdylib`) extension crate.

---

## Synopsis

```bash
solix new <path> [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<path>` | Path | **Yes** | Destination directory path for the new project. Created if it does not exist. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--name <name>` | | String | Target directory name | Project and package identifier name. |
| `--version <version>` | | String | `"0.1.0"` | Initial semantic version for the package. |
| `--description <desc>` | | String | `"A Solix project"` | Human-readable package description. |
| `--license <id>` | | String | `"Apache-2.0"` | SPDX license identifier. |
| `--author <name>` | | String (Repeated) | `[]` | Author attribution name (can be specified multiple times). |
| `--main <symbol>` | | String | `"Main::main"` | Canonical application entry point symbol. |
| `--test <symbol>` | | String | `"TestRunner::test"` | Canonical test runner entry point symbol. |
| `--lib` | | Flag | `false` | Scaffolds a reusable library project instead of an executable application. |
| `--native` | | Flag | `false` | Explicitly enables and scaffolds the Rust native `cdylib` extension bridge in `natives/`. |
| `--no-native` | | Flag | `false` | Omits the Rust native `cdylib` extension crate bridge in `natives/`. |
| `--deps <pkg@ver>` | `-d` | String (Repeated) | `[]` | Declares an external package dependency (e.g. `-d raylib@1.0.0`). |
| `--force` | | Flag | `false` | Overwrites existing `manifest.toml` if the directory already contains a project. |

---

## Generated Project Layout

### 1. Application Project (Default)

Executing `solix new my_app`:

```text
my_app/
├── manifest.toml             # Configuration, metadata, and dependencies
├── .gitignore                # Ignores /build/, /target/, *.slxb, *.slxarc
├── src/
│   └── main.slx              # Starter application source
├── tests/
│   └── test_main.slx         # Automated test entrypoint
├── resources/                # Application assets directory
├── lib/                      # Staging directory for native shared libraries
├── natives/
│   └── my_app_bridge/        # Rust cdylib bridge extension
│       ├── Cargo.toml
│       └── src/lib.rs        # SolixCValue FFI and native callback exports
└── scripts/
    ├── build.py              # Build automation script
    └── solix_build.py        # Turnkey build orchestration primitives
```

#### Generated `manifest.toml`
```toml
[configuration]
name = "my_app"
version = "0.1.0"
description = "A Solix project"
license = "Apache-2.0"
authors = []

[profiles.release]
entrypoint = "Main::main"
optimization_level = 3
inline_threshold = 16
debug_symbols = false
static = true
sources = ["src"]

[profiles.debug]
inherits = "release"
optimization_level = 0
inline_threshold = 8
debug_symbols = true
static = false

[profiles.test]
inherits = "debug"
entrypoint = "TestRunner::test"
sources = ["src", "tests"]
```

#### Generated `src/main.slx`
```solix
class Main {
    public static int32 main() {
        return 0;
    }
}
```

#### Generated `tests/test_main.slx`
```solix
class TestRunner {
    public static int32 test() {
        return 0;
    }
}
```

### 2. Library Project (`--lib`)

Executing `solix new my_lib --lib`:

Generates `src/lib.slx` with package declaration instead of `src/main.slx`:

```solix
package my_lib;

public class MyLib {
    public static int32 version() {
        return 1;
    }
}
```

### 3. Pure Solix Project (`--no-native`)

Executing `solix new my_tool --no-native`:
Omits the Rust `Cargo.toml` and native bridge crate inside `natives/`, creating an empty `natives/.gitkeep`.

---

## Collision Protection Rules

1. If the target directory already contains a `manifest.toml`, `solix new` halts with an error unless `--force` is supplied.
2. When `--force` is specified, existing files are preserved unless explicitly rewritten by the scaffolding engine.

---

## Exit Codes

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Project workspace successfully initialized. |
| `1` | Error | Target directory conflict (without `--force`) or filesystem permission failure. |

---

## Examples

### Scaffolding an Application with Dependencies
```bash
solix new game_app \
  --description "2D Arcade Game in Solix" \
  --author "Alice Dev <alice@example.com>" \
  --deps "raylib@1.0.0" \
  --deps "math_matrix@0.2.0"
```

### Scaffolding a Pure Library
```bash
solix new json_parser --lib --no-native
```
