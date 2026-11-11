# Formal Specification: Standard Library System & Process (`solix.system`)

## Document Metadata

* **Specification ID:** SPEC-0023
* **Scope:** Environment Introspection & Subprocess Spawning
* **Status:** Final Normative Standard
* **Source Path:** [`libraries/solixlib/src/system/`](file:///home/jehud/Projects/solix-language/libraries/solixlib/src/system/)

---

## 1. Environment Introspection (`solix.system.Environment`)

The `Environment` class provides static inspection of the host operating system:
* `static Optional<String> get_variable(String variable_name)`: Reads environment variable.
* `static void set_variable(String variable_name, String variable_value)`: Sets variable in process.
* `static String[] get_arguments()`: Returns CLI command line arguments passed to `main`.
* `static String current_directory()`: Returns process current working directory.
* `static String operating_system()`: Returns `"linux"`, `"windows"`, or `"macos"`.
* `static String processor_architecture()`: Returns `"x86_64"`, `"aarch64"`, etc.
* `static int32 processor_count()`: Returns number of available logical CPU cores.
* `static void exit(int32 exit_code)`: Terminates process immediately with status code.

---

## 2. Process Execution (`solix.system.Process`)

The `Process` class spawns and awaits external child processes:

```solix
public class Process {
    public static ProcessResult execute(String program_path, String[] command_arguments) { ... }
    public static ProcessResult execute_shell(String shell_command) { ... }
}
```

### `ProcessResult` Container
* `public int32 exit_code;`
* `public String standard_output;`
* `public String standard_error;`
* `public bool is_success()`: Returns `exit_code == 0`.
