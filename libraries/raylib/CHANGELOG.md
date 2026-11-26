# Changelog - raylib

All notable changes to this Solix package are documented in this file.

## [6.0.0] - 2026-11-26

### Bug Fixes

- *(release)* Split macOS Xcode and Docker Linux/Windows bundled builds

### Features

- *(toolchain)* Containerize apple darwin cross-compilation everywhere (#276) (#277)
- *(toolchain)* Mandate docker cross-compilation for native library builds (#253, #251) (#271)

### Refactoring

- *(raylib)* Emit root pascalcase functions and use solix-ffi (#272) (#273)

## [5.6.1] - 2026-11-25

### Bug Fixes

- *(compiler)* Resolve eleventh-sweep binder, LSP, DAP, VM, fmt, and stdlib edge cases (#251-#260) (#261)

## [5.6.0] - 2026-11-24

### Bug Fixes

- *(compiler)* Resolve fourth-sweep memory safety, arithmetic, and parser edge cases (#183-#192) (#193)

### Features

- *(toolchain)* Unify native platforms, eliminate lib/ directory, and add pre-compiled library verification (#151)

