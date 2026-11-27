# `solix add`

Resolves and adds a package dependency to the project's `manifest.yaml`.

## Usage

```bash
solix add <PACKAGE> [OPTIONS]
```

## Options

| Flag | Description | Default |
| :--- | :--- | :--- |
| `<PACKAGE>` | Package specification (`name` or `name@version`) | Required |
| `--path <PATH>` | Project root directory containing `manifest.yaml` | `.` |
| `--dep-profile <PROFILE>` | Dependency build profile to link against | `release` |
| `--pkg-dir <DIR>` | Override local package store directory | `~/.solix/packages` |
