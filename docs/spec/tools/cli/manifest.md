# Solix Project Manifest (`manifest.toml`) Specification

The `manifest.toml` file is the declarative configuration descriptor for Solix projects, applications, and libraries. It defines package identity, author attribution, application and test entrypoints, and dependency requirements.

---

## Schema Overview

A complete `manifest.toml` document structure:

```toml
[info]
name = "my_project"
version = "0.1.0"
description = "High-performance Solix application with native bindings"
license = "Apache-2.0"
authors = ["Jane Developer <jane@example.com>", "Solix Core Team"]

[entrypoints]
main = "Main::main"
test = "TestRunner::test"

[dependencies]
solixlib = "2.0.0"
raylib = ">=1.0.0"
```

---

## Section & Field Specifications

### 1. `[info]` Table (Required)

Contains foundational metadata identifying the package and its authors.

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `name` | String | **Yes** | — | Unique package identifier. Must be a valid identifier (alphanumeric, underscores, hyphens). |
| `version` | String | **Yes** | `"0.1.0"` | Semantic version string adhering to SemVer 2.0 (`MAJOR.MINOR.PATCH`). |
| `description` | String | No | `""` | Human-readable summary of the package. |
| `license` | String | No | `"Apache-2.0"` | SPDX license identifier (e.g., `Apache-2.0`, `MIT`). |
| `authors` | Array of Strings | No | `[]` | List of author names or name/email pairs. |

```toml
[info]
name = "network_engine"
version = "1.2.0"
description = "Asynchronous networking primitives for Solix"
license = "MIT"
authors = ["Jane Developer"]
```

---

### 2. `[entrypoints]` Table (Optional for Libraries, Recommended for Apps)

Defines canonical execution entry points for running the program and its test harness.

| Field | Type | Default | Description |
|---|---|---|---|
| `main` | String | `"Main::main"` | Canonical fully-qualified method name invoked when executing the application (`solix run <target.slxarc>`). |
| `test` | String | `"TestRunner::test"` | Fully-qualified test runner method executed when invoking with `--test` (`solix run --test <target.slxarc>`). |

```toml
[entrypoints]
main = "App::main"
test = "SuiteRunner::run_tests"
```

> [!NOTE]
> Entry point signatures must return an integer (`int32` or `int64`) representing the OS process exit code, or return `void`.

---

### 3. `[dependencies]` Table (Optional)

Defines external package dependencies resolved against the local Solix package store (`~/.solix/packages`).

Each entry specifies the required package name and a SemVer version requirement string:

```toml
[dependencies]
solixlib = "2.0.0"
raylib = "^1.2.0"
math_matrix = "0.3.1"
```

#### Version Requirement Syntax

| Syntax | Type | Matches | Example |
|---|---|---|---|
| `"1.2.3"` | Exact / Compatible | `>= 1.2.3, < 2.0.0` (SemVer caret semantics) | `"1.2.3"` matches `1.2.4`, rejects `2.0.0` |
| `"^1.2.0"` | Caret Requirement | Compatible non-breaking updates | `"^1.2.0"` matches `1.3.0`, rejects `2.0.0` |
| `">=1.0.0"` | Comparison Range | Greater than or equal to version | `">=1.0.0"` matches any version `>= 1.0.0` |
| `"*"` | Wildcard | Any available installed version | `"*"` matches any installed version |

---

## Directory Conventions Around `manifest.toml`

A standard Solix project structured around `manifest.toml`:

```text
my_project/
├── manifest.toml             # Authoritative configuration descriptor
├── .gitignore                # Ignores /cache/, /build/, *.slxb, *.slxarc
├── src/
│   └── main.slx              # Application source code
├── tests/
│   └── test_main.slx         # Automated test suite
├── resources/                # Embedded static assets (images, audio, config)
├── lib/                      # Precompiled native shared libraries (.so / .dll / .dylib)
├── natives/                  # Rust cdylib bridge extension crates
│   └── my_project_bridge/
│       ├── Cargo.toml
│       └── src/lib.rs
└── scripts/
    ├── build.py              # Build automation entrypoint
    └── solix_toolbox.py      # Build toolbox helper module
```

---

## Role in Build & Distribution Lifecycle

1. **Scaffolding (`solix new`)**: Creates `manifest.toml` populated with CLI arguments (`--name`, `--version`, `--license`, `--author`, `--deps`).
2. **Building (`solix build`)**: `scripts/build.py` reads `manifest.toml`, extracts metadata and dependencies, invokes `solix compile`, and packages `manifest.toml` at the root of the resulting `.slxarc` archive.
3. **Execution (`solix run`)**: When running a `.slxarc` archive, the VM runtime reads `manifest.toml` from the archive container to determine entry points (`[entrypoints].main` or `[entrypoints].test`) and resolve bundled resources.
4. **Inspection (`solix info`)**: Reads and formats `manifest.toml` when displaying container metadata.
