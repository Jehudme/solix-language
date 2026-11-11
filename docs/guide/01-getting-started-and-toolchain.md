# Guide 01: Getting Started & The Solix Toolchain

Welcome to Solix! This guide walks you through setting up the Solix toolchain, creating your first project, compiling standalone scripts, and executing bytecode binaries.

---

## 1. Prerequisites & The Unified CLI

The Solix toolchain is distributed as a single high-performance binary: `solix`.
To verify your installation:

```bash
solix --version
```

---

## 2. Your First Standalone Script

Create a new file named `hello.slx`:

```solix
extern import solix.io.Console;

public int32 main() {
    Console::println("Hello, Solix 2.0!");
    return 0;
}
```

> [!NOTE]
> Notice the scope resolution operator `::` in `Console::println`. In Solix v2.0, all static methods and enum variants **must** be accessed using `::`.

### Compiling and Running
Compile the source file into a bytecode binary (`.slxb`):

```bash
solix compile hello.slx -o hello.slxb
```

Execute the binary with the Solix Virtual Machine:

```bash
solix run hello.slxb
```

Output:
```text
Hello, Solix 2.0!
```

---

## 3. Creating a Structured Project (`solix new`)

For multi-file applications and libraries, scaffold a project with `solix new`:

```bash
solix new my_application
cd my_application
```

This creates a standard project structure:

```text
my_application/
├── manifest.toml              # Project metadata & dependency declarations
├── src/
│   └── main.slx               # Application entrypoint
├── tests/                     # Integration tests
└── build/                     # Compiled .slxarc archives
```

### The `manifest.toml` Configuration

```toml
[info]
name = "my_application"
version = "0.1.0"
description = "A modern Solix application"
authors = ["Developer Name"]
license = "MIT"

[entrypoints]
main = "main"

[dependencies]
solix = "2.0.0"
```

### Building and Testing

Build the project in release mode:

```bash
solix build --release
```

This produces a self-contained archive at `build/release/my_application.slxarc`. Run it directly:

```bash
solix run build/release/my_application.slxarc
```

---

## 4. Cross-References & Deep Dives

* **Specification:** [Compilation Pipeline & Binary Containers](file:///home/jehud/Projects/solix-language/docs/spec/compilation-pipeline.md)
* **Wiki Reference:** [CLI Commands Cheat Sheet](file:///home/jehud/Projects/solix-language/docs/wiki/commands/README.md)
* **Next Guide:** [Guide 02: Variables, Primitives & Strings](file:///home/jehud/Projects/solix-language/docs/guide/02-variables-primitives-and-strings.md)
