# Guide 08: Packages, Dependencies & Linking

This guide explains how Solix manages dependencies, resolves semantic versions, and supports both dynamic modular linking and static hermetic linking.

---

## 1. Declaring Dependencies in `manifest.toml`

Every Solix package specifies external package requirements in `manifest.toml`:

```toml
[info]
name = "game_engine"
version = "1.0.0"

[dependencies]
solix = "2.0.0"
raylib = "5.0.0"
```

---

## 2. Importing External Packages (`extern import`)

Use `extern import` to bind symbols from installed dependencies:

```solix
extern import solix.io.Console;
extern import raylib.Window;

public int32 main() {
    Console::println("Initializing game engine...");
    return 0;
}
```

---

## 3. Dynamic vs Static Linking

### 3.1 Dynamic Linking (Default)
* Builds modular `.slxb` containers.
* Dependencies remain external archives in `~/.solix/packages/`.
* When booting the application, the VM resolves external function calls and quickens them in-place for fast execution.

### 3.2 Static Linking (`--static-link`)
* Bundles all dependency bytecode into a single standalone `.slxb` container.
* Runs **Dead-Code Elimination (DCE)**: unreferenced classes, methods, and functions in libraries are pruned, minimizing final binary size.

```bash
solix compile src/main.slx -o app.slxb --static-link
```

---

## 4. Cross-References

* **Specification:** [Package Solver & Linking](file:///home/jehud/Projects/solix-language/docs/spec/package-solver-and-linking.md)
* **Next Guide:** [Guide 09: Native Rust Extensions](file:///home/jehud/Projects/solix-language/docs/guide/09-native-rust-extensions.md)
