# CLI Reference: `clean`, `doc` & `wiki` Utility Commands

This section documents project cleanup (`solix clean`), static API doc extraction (`solix doc`), and the embedded offline documentation portal (`solix wiki`).

---

## 1. `solix clean`

Removes build artifacts and intermediate native caches inside a project's `build/` directory:

```bash
solix clean [path] [options]
```

### Key Options & Flags
* `[path]`: Target project directory [default: `.`].
* `--cache-only`: Deletes only `build/.cache/` intermediate CMake/Cargo incremental caches while keeping compiled `.slxp` archives in `build/<profile>/`.

---

## 2. `solix doc`

Generates static HTML or Markdown API reference documentation from `///` and `/** ... */` doc comments:

```bash
solix doc [path] [options]
```

### Key Options & Flags
* `[path]`: Project root or `.slx` source directory [default: `.`].
* `-o, --output <dir>`: Output directory path [default: `docs/api`].
* `--format <html|markdown>`: Documentation output format (`html` [default] or `markdown`).
* `--open`: Launches `index.html` in the system browser when complete.
* `--profile <mode>`: Target build profile used for dependency/source resolution [default: `debug`].

---

## 3. `solix wiki`

Launches or exports the bundled offline HTML documentation portal matched to the installed `solix` binary version:

```bash
solix wiki [topic] [options]
```

### Key Options & Flags
* `[topic]`: Optional search filter query pre-populated in the documentation portal.
* `--search <query>`: Explicit search query alias for filtering documentation results.
* `--no-open`: Materializes the offline HTML portal without spawning a web browser.
* `-o, --output <dir>`: Copies the rendered offline site into `<dir>`.
* `--path`: Prints only the resolved filesystem path to `index.html`.
