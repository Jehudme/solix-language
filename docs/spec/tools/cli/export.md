# `solix export` — Package and Binary Export

The `export` subcommand exports packaged containers (`.slxp`) or compiled bytecode binaries (`.slxb`) to designated destination directories or package paths for distribution.

---

## Synopsis

```bash
solix export package <target> [options]
solix export binaries <binary> [options]
```

---

## Subcommand 1: `solix export package`

Exports a built package artifact or project directory into a distributable archive container (`.slxp`).

### Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<target>` | Path | **Yes** | Path to a project root directory containing `manifest.yaml` or an existing package archive. |

### Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--output <path>` | `-o` | Path | None | Destination file or directory path for the exported package. Defaults to `build/<profile>/<name>.slxp`. |
| `--profile <mode>` | | String | `"release"` | Target build profile (`release`, `debug`, or `test`). |
| `--debug` | | Flag | `false` | Shorthand for `--profile debug`. |
| `--release` | | Flag | `false` | Shorthand for `--profile release`. |
| `--test` | | Flag | `false` | Shorthand for `--profile test`. |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for resolving dependencies. |

### Example

```bash
solix export package ./my_library --output ./dist/my_library.slxp
```

---

## Subcommand 2: `solix export binaries`

Exports a compiled bytecode container (`.slxb`) to a destination path or distribution folder.

### Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<binary>` | Path | **Yes** | Path to the compiled `.slxb` bytecode binary container. |

### Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--output <path>` | `-o` | Path | None | Destination file path or directory for the exported binary container. |

### Example

```bash
solix export binaries build/release/app.slxb --output ./dist/app.slxb
```
