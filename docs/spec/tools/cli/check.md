# `solix check`

Type-checks a Solix project using `CompilationPipeline::check()` without emitting bytecode or native artifacts.

## Usage

```bash
solix check [PATH] [OPTIONS]
```

## Options

| Flag | Description | Default |
| :--- | :--- | :--- |
| `[PATH]` | Project root directory containing `manifest.yaml` | `.` |
| `-p, --profile <PROFILE>` | Target build profile (`debug`, `release`, `test`, or custom) | `debug` |
| `--pkg-dir <DIR>` | Override local package store directory | `~/.solix/packages` |
| `--no-verify` | Bypass cryptographic SHA-256 verification on dependencies | `false` |
