# Formal Specification: Standard Library I/O & File System (`solix.io`)

## Document Metadata

* **Specification ID:** SPEC-0020
* **Scope:** Standard Console, File System, Streams & Buffering
* **Status:** Final Normative Standard
* **Source Path:** [`libraries/solixlib/src/io/`](file:///home/jehud/Projects/solix-language/libraries/solixlib/src/io/)

---

## 1. Console I/O (`solix.io.Console`)

The `Console` class provides static methods for terminal interaction:
* `static void print(string text)` / `static void print(String text)`
* `static void println(string text)` / `static void println(String text)`
* `static String read_line()`: Reads a line from standard input up to newline.
* `static void clear()`: Clears the terminal screen.
* `static void set_color(int32 color_code)`: Sets ANSI terminal color attributes.

---

## 2. Stream Hierarchy & Enums

### 2.1 Stream Enums
* **`FileMode`**: `Open`, `Create`, `Append`, `OpenOrCreate`, `Truncate`.
* **`SeekOrigin`**: `Begin`, `Current`, `End`.

### 2.2 `IStream` Interface & Implementations
```text
IStream
  ├── FileStream      # OS file descriptor stream with seek/read/write
  └── MemoryStream    # Dynamic resizable byte buffer stream
```

* `int32 read(int8[] buffer, int32 offset, int32 count)`
* `void write(int8[] buffer, int32 offset, int32 count)`
* `int64 seek(int64 offset, SeekOrigin origin)`
* `void flush()`
* `void close()`

---

## 3. High-Level File System Operations

### 3.1 `Path` Utilities
* `static String combine(String path_a, String path_b)`: Joins paths using platform separator.
* `static String get_directory_name(String full_path)`
* `static String get_file_name(String full_path)`
* `static String get_extension(String full_path)`

### 3.2 `File` Class
* `static bool exists(String path_string)`
* `static String read_all_text(String path_string)`
* `static void write_all_text(String path_string, String content_text)`
* `static int8[] read_all_bytes(String path_string)`
* `static void write_all_bytes(String path_string, int8[] data_bytes)`
* `static void delete(String path_string)`

### 3.3 `Directory` Class
* `static bool exists(String directory_path)`
* `static void create_directory(String directory_path)`
* `static String[] list_files(String directory_path)`
* `static String[] list_directories(String directory_path)`
* `static void delete(String directory_path, bool recursive)`
