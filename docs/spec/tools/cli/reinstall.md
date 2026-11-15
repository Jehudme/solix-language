# `solix reinstall` — Reinstall & Rebuild Packages

The `reinstall` subcommand (visible alias `sync`) rebuilds and reinstalls installed packages directly from their recorded source directories, validating directory and manifest integrity, updating local artifacts, refreshing the extract cache, and recording new cryptographic checksums in `registry.json`.

---

## Synopsis

```bash
solix reinstall [spec] [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `[spec]` | String | No | Package specifier (e.g. `raylib`, `raylib-debug`, `raylib@5.5.0`, or `raylib-debug@5.5.0`) to rebuild from its recorded source directory. Omit when using `--all` or running inside a project directory. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--all` | | Flag | `false` | Reinstalls all registered packages that have recorded source directories. |
| `--profile <prof>` | | String | (recorded) | Build profile override (`debug`, `release`, `test`). Defaults to each package's recorded profile. |
| `--force` | `-f` | Flag | `false` | Force overwrite during re-installation. |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for the local package store. |

---

## Developer Workflow

### 1. High-Frequency Local Development

When developing a library alongside application consumers:

```bash
# 1. Install library directly from source directory once
solix install ../libraries/solixlib --profile release

# 2. Modify library source code in ../libraries/solixlib...

# 3. Rebuild and reinstall with one command from anywhere
solix reinstall solixlib

# Or reinstall all active local libraries:
solix reinstall --all
```
