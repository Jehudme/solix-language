# CLI Reference: Package Management Commands

---

## 1. `solix new`

Scaffolds a new standard Solix project with `manifest.toml`, `src/main.slx`, and test suites:

```bash
solix new <project_name>
```

---

## 2. `solix install`

Installs a compiled `.slxarc` archive, `.slxb` binary, or project folder into the local package store:

```bash
solix install <target> [--profile <profile>] [--force]
```

### Key Flags
* `-f, --force`: Overwrites an existing installed package version.
* `--profile <debug|release|test>`: Selects build profile when installing from a project directory.

---

## 3. `solix uninstall`

Removes a package version or an entire package from the local package store:

```bash
solix uninstall <package_name> [--version <version>]
```

---

## 4. `solix list`

Lists all installed packages currently available in the package store:

```bash
solix list [flags]
```

### Key Flags
* `--json`: Emits package inventory as machine-readable JSON.
* `--filter <query>`: Filters packages by name substring.
* `--profile <profile>`: Filters packages by build profile.
* `--sort <name|version|date|size|profile>`: Sorts package entries.
