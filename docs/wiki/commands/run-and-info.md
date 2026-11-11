# CLI Reference: `run` & `info`

---

## 1. `solix run`

Executes compiled `.slxb` binaries or `.slxarc` project archives:

```bash
solix run <target.slxb | target.slxarc> [args...]
```

### Key Flags
* `--test`: Runs unit test fixtures embedded within the archive.
* `--yolo, --ignore-checksum`: Skips runtime dependency and native shared library SHA-256 verification.
* `--heap-size <bytes>`: Configures initial VirtualHeap allocation capacity.

---

## 2. `solix info`

Inspects compiled container metadata, binary sections, and disassembly:

```bash
solix info <target_path | package_name[@version]> [flags]
```

### Key Flags
* `--disasm`: Disassembles all functions into human-readable 64-bit Register VM opcodes.
* `--api`: Emits synthesized public external API signatures (`manifest_to_source`).
* `--json`: Emits binary metadata, classes, vtables, and instructions as structured JSON.
* `--sections`: Lists all binary container sections and byte lengths.
