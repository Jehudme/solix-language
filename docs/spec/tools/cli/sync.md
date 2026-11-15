# `solix sync` — Synchronize & Rebuild Packages

The `sync` subcommand synchronizes and rebuilds installed packages directly from their recorded source directories, updating local artifacts, refreshing the extract cache, and recording new cryptographic checksums in `registry.json`.

---

## Synopsis

```bash
solix sync [package_name] [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `[package_name]` | String | No | Name of the installed package to rebuild from its recorded source directory. Omit when using `--all` or running inside a project directory. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--all` | | Flag | `false` | Synchronizes all registered packages that have recorded source directories. |
| `--profile <prof>` | | String | (recorded) | Build profile override (`debug`, `release`, `test`). Defaults to each package's recorded profile. |
| `--force` | `-f` | Flag | `false` | Force overwrite during re-installation. |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for the local package store. |

---

## Developer Workflow

### 1. High-Frequency Local Development

When developing a library alongside application consumers:

```bash
# 1. Install library directly from source directory once
solix install ../libraries/solixlib --release

# 2. Modify library source code in ../libraries/solixlib...

# 3. Rebuild and synchronize with one command from anywhere
solix sync solixlib

# Or synchronize all active local libraries:
solix sync --all
```

### 2. Automatic Build & Sync on Compilation

You can also pass `--install` (alias `--sync`) directly during build or compilation:

```bash
# In library project folder:
solix build --release --sync

# Or when compiling standalone library binary:
solix compile src/math.slx --name math_utils --version 1.0.0 --sync
```
