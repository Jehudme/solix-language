# `solix init`

Initializes a `manifest.yaml` using `vars:` YAML anchors inside an existing project directory without overwriting existing source files.

## Usage

```bash
solix init [PATH] [OPTIONS]
```

## Options

| Flag | Description | Default |
| :--- | :--- | :--- |
| `[PATH]` | Target directory to initialize | `.` |
| `--name <NAME>` | Package name | Directory name |
| `--force` | Overwrite existing `manifest.yaml` if present | `false` |
