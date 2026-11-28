# `solix export` — Package and Binary Export

The `export` subcommand exports packaged containers (`.slxp`) or compiled bytecode binaries (`.slxb`) to designated destination directories or package paths for distribution.

---

## Synopsis

```bash
solix export package [path] [options]
solix export binaries [path] [options]
```

---

## Subcommand 1: `solix export package`

Exports a built package artifact or project directory into a distributable archive container (`.slxp`).

### Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `[path]` | Path | Optional (default: `.`) | Path to a project root directory containing `manifest.yaml`. |

### Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--output <path>` | `-o` | Path | None | Destination file or directory path for the exported package. Defaults to `<name>.slxp` in project or current directory. |
| `--profile <mode>` | `-p` | String | `"release"` | Target build profile (`release`, `debug`, or `test`). |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for resolving dependencies. |

### Example

```bash
solix export package ./my_library --output ./dist/my_library.slxp
```

---

## Subcommand 2: `solix export binaries`

Exports a standalone compiled `.slxb` bytecode container from a Solix project directory.

### Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `[path]` | Path | Optional (default: `.`) | Path to a project root directory containing `manifest.yaml`. |

### Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--output <path>` | `-o` | Path | None | Destination file path or directory for the exported binary container (`<name>.slxb`). |
| `--profile <mode>` | `-p` | String | `"release"` | Target build profile (`release`, `debug`, or `test`). |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for resolving dependencies. |

### Example

```bash
solix export binaries ./my_library --output ./dist/app.slxb
```
