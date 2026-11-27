# `solix test`

Compiles a Solix project using the `test` profile (or `--profile`) and executes all discovered zero-argument `test_*` functions/methods.

## Usage

```bash
solix test [PATH] [OPTIONS]
```

## Options

| Flag | Description | Default |
| :--- | :--- | :--- |
| `[PATH]` | Project root directory containing `manifest.yaml` | `.` |
| `-p, --profile <PROFILE>` | Target build profile | `test` |
| `--filter <PATTERN>` | Run only tests whose symbol name contains `<PATTERN>` | None |
| `--pkg-dir <DIR>` | Override local package store directory | `~/.solix/packages` |
| `--no-verify` | Bypass cryptographic SHA-256 verification on dependencies | `false` |
