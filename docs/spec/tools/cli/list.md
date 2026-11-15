# `solix list` — Local Package Inventory

The `list` subcommand scans and reports all packages and versions installed in the local Solix package store (`$SOLIX_PKG_DIR/registry.json`), with rich filtering, multi-key sorting, and machine-readable output.

---

## Synopsis

```bash
solix list [options]
```

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--filter <query>` | `-f` | String | | Filters packages by matching substring in name or description. |
| `--profile <prof>` | | String | | Filters packages by build profile (`debug`, `release`, `test`). |
| `--kind <kind>` | `-k` | String | | Filters packages by container format (`slxb`, `slxarc`). |
| `--sort <key>` | | String | `"name"` | Sort field: `name`, `version`, `date`, `size`, or `profile`. |
| `--reverse` | `-r` | Flag | `false` | Reverses sort order. |
| `--json` | | Flag | `false` | Output package inventory formatted as structured JSON. |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for the local package store. |

---

## Output Formats

### 1. Human-Readable Table (Default)

```text
Installed Solix Packages (/home/user/.solix/packages):
NAME             VERSION    KIND    PROFILE  SIZE       INSTALLED AT         SHA256          SOURCE
---------------- ---------- ------- -------- ---------- -------------------- --------------- --------------------
solix            2.0.0      slxarc  release  681.8 KB   2026-11-11T05:44:54Z 46747ea15ac6... /home/user/solixlib
raylib           1.2.0      slxarc  release  1.2 MB     2026-11-11T02:15:30Z 9d1da08c4ede... -
```

### 2. JSON Output (`--json`)

```bash
solix list --json
```

```json
[
  {
    "name": "solix",
    "version": "2.0.0",
    "kind": "slxarc",
    "build_profile": "release",
    "sha256": "46747ea15ac6f2f15d462438a63fb9ef71581361ab5facca844c97a488d24812",
    "size_bytes": 698144,
    "installed_at_unix_secs": 1762839894,
    "installed_at_iso8601": "2026-11-11T05:44:54Z",
    "source": "/home/user/solixlib",
    "artifact_path": "/home/user/.solix/packages/solix-release@2.0.0/solix.slxarc",
    "native_libraries": {
      "lib/libsolixlib_native.so": "af5f19f16fac4342322dc64262fc55e49b4119122e690957773c78aa25405a44"
    },
    "description": "Solix Standard Library",
    "license": "Apache-2.0",
    "authors": ["Solix Team"]
  }
]
```
