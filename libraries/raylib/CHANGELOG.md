# Changelog - raylib

All notable changes to this Solix package are documented in this file.

## [6.1.0] - 2026-11-28

### Features

- *(raylib)* Emit typedef aliases and add rlgl/raymath API bindings (#324, #325) (#331)

## [6.0.1] - 2026-11-27

### Bug Fixes

- *(solixlib, raylib, compiler)* Harden stdlib net/json/collections, overload resolution, and Raylib OOP ergonomics (#314)
- *(ffi,stdlib)* Harden C-unwind FFI ABI, Raylib bindgen, and solixlib_native safety (#313)

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

