# `solix wiki` — Offline Versioned Documentation Portal

The `wiki` subcommand launches the offline, static HTML documentation portal embedded inside the `solix` toolchain binary. Because every release binary bundles its matching language guides, wiki articles, formal specifications, and offline JavaScript search index (`search_index.js`), developers pinned to any version (such as `v1.4.0` or `v1.5.0`) always browse exact documentation matching their installed toolchain without requiring internet access or spawning a local HTTP server.

---

## Synopsis

```bash
solix wiki [topic] [options]
```

---

## Positional Arguments

| Argument | Type | Default | Description |
|---|---|---|---|
| `[topic]` | String | `None` | Optional search query or topic keyword used to pre-populate the portal search box (`index.html?q=<topic>`). |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--search <query>` | | String | `None` | Explicit search query or `--topic` alias to filter documentation results immediately upon launch. |
| `--no-open` | | Flag | `false` | Materializes and prints the resolved `index.html` path (or launch URI) without opening a web browser. |
| `--output <dir>` | `-o` | Path | `None` | Exports a copy of the offline HTML documentation site into `<dir>` before opening or printing. |
| `--path` | | Flag | `false` | Prints only the resolved filesystem path to `index.html` for scripting or editor integrations. |

---

## Environment Resolution Order

1. `SOLIX_WIKI_DIR`: Optional directory override containing a rendered `index.html` static portal.
2. Sibling `assets/docs/` next to the running `solix` executable (unless `SOLIX_FORCE_EMBEDDED_ASSETS=1` is set).
3. Workspace fallback `<cwd>/crates/solix-toolchain/assets/docs/`.
4. Compile-time embedded HTML portal materialized automatically into `$SOLIX_HOME/cache/wiki/v<version>/` (or `~/.solix/cache/wiki/v<version>/`).

---

## Practical Examples

```bash
# Open the bundled offline Solix documentation in your default browser
solix wiki

# Pre-filter offline docs for Weak References or ARC guides
solix wiki "weak reference"
solix wiki --search E0226

# Materialize embedded docs into isolated cache and print only the path
solix wiki --no-open --path

# Export static HTML docs for offline archive or CI artifacts
solix wiki --no-open --output ./exported-docs
```
