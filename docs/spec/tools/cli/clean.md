# `solix clean` — Project Build Artifact & Cache Cleanup

The `clean` subcommand safely removes generated build artifacts and intermediate caches from a Solix project's `build/` directory, reporting the reclaimed disk space and number of deleted files.

---

## Synopsis

```bash
solix clean [path] [options]
```

---

## Positional Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `[path]` | Path | No | `.` | Root directory of the Solix project to clean. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--cache-only` | | Flag | `false` | Removes only intermediate cache artifacts in `build/.cache/`, preserving compiled package archives in `build/<profile>/`. |

---

## Behavior & Semantics

### 1. Full Build Cleanup (Default)

Running `solix clean` without flags removes the entire `build/` directory within the target project root, including all intermediate compilation outputs, native staging trees, and compiled package archives:

```bash
solix clean
```

Example output:
```text
Cleaned '/path/to/project/build': Reclaimed 14.2 MB across 48 file(s).
```

### 2. Cache-Only Cleanup (`--cache-only`)

Passing `--cache-only` removes strictly the `build/.cache/` directory while preserving final distribution packages (such as `build/release/<name>.slxp`):

```bash
solix clean --cache-only
```

Example output:
```text
Cleaned '/path/to/project/build/.cache': Reclaimed 12.8 MB across 42 file(s).
```

### 3. Idempotent Execution

If no `build/` directory or `build/.cache/` directory exists, `solix clean` completes cleanly with an exit code of `0` and informs the user:

```text
No build directory found at '/path/to/project/build'. Nothing to clean.
```
