# Changelog - solix

All notable changes to this Solix package are documented in this file.

## [2.2.0] - 2026-11-26

### Bug Fixes

- *(release)* Split macOS Xcode and Docker Linux/Windows bundled builds

### Features

- *(toolchain)* Containerize apple darwin cross-compilation everywhere (#276) (#277)
- *(toolchain)* Mandate docker cross-compilation for native library builds (#253, #251) (#271)

## [2.1.1] - 2026-11-25

### Bug Fixes

- *(compiler)* Resolve eleventh-sweep binder, LSP, DAP, VM, fmt, and stdlib edge cases (#251-#260) (#261)

## [2.1.0] - 2026-11-24

### Bug Fixes

- *(compiler)* Resolve tenth-sweep binder, LSP, DAP, VM, and stdlib edge cases (#229-#234) (#235)
- *(compiler)* Resolve ninth-sweep binder, LSP, VM, and stdlib edge cases (#223-#227) (#228)
- *(compiler)* Resolve eighth-sweep verifier, binder, DAP, and stdlib edge cases (#217-#221) (#222)
- *(compiler)* Resolve seventh-sweep verifier, binder, toolchain, DAP, and stdlib edge cases (#210-#215) (#216)
- *(compiler)* Resolve sixth-sweep verifier, binder, FFI, and stdlib edge cases (#203-#207) (#209)
- *(compiler)* Resolve fifth-sweep OOP, verifier, cycle collector, and stdlib edge cases (#194-#201) (#202)
- *(compiler)* Resolve fourth-sweep memory safety, arithmetic, and parser edge cases (#183-#192) (#193)
- *(compiler)* Resolve third-sweep compiler, runtime, and stdlib edge cases (#171-#180) (#181)
- *(compiler)* Resolve second-sweep architectural and safety edge cases (#170)

### Features

- *(toolchain)* Unify native platforms, eliminate lib/ directory, and add pre-compiled library verification (#151)

