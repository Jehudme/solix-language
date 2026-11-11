# Guide 10: Standard Library Tour (`solixlib`)

This guide provides practical recipes covering the official Solix standard library (`solixlib` v2.0): collections, file I/O, streams, cryptography, date/time, and process execution.

---

## 1. Linear Collections & Algorithms

```solix
extern import solix.collections.List;
extern import solix.collections.Algorithms;
extern import solix.io.Console;

public int32 main() {
    List<int32> numbers = new List<int32>();
    numbers.add(42);
    numbers.add(10);
    numbers.add(99);

    int32[] buffer = numbers.to_array();
    Algorithms::sort<int32>(buffer);

    for (int32 index = 0; index < buffer.length; index = index + 1) {
        Console::println("Item: " + buffer[index]);
    }
    return 0;
}
```

---

## 2. File I/O & Text Operations

```solix
extern import solix.io.File;
extern import solix.core.String;
extern import solix.io.Console;

public void read_sample_file() {
    String file_path = new String("data/config.json");

    if (File::exists(file_path)) {
        String content_text = File::read_all_text(file_path);
        Console::println("Loaded: " + content_text);
    }
}
```

---

## 3. Cryptographic Hashing

```solix
extern import solix.crypto.Hash;
extern import solix.core.String;
extern import solix.io.Console;

public void compute_digests() {
    String secret_payload = new String("secure_password_123");
    String sha256_digest = Hash::sha256_hex(secret_payload);

    Console::println("SHA-256: " + sha256_digest);
}
```

---

## 4. Date, Time & Benchmarking

```solix
extern import solix.time.Stopwatch;
extern import solix.time.DateTime;
extern import solix.io.Console;

public void run_benchmark() {
    DateTime start_time = DateTime::now();
    Console::println("Benchmark started at: " + start_time.to_iso8601());

    Stopwatch timer = Stopwatch::start_new();
    perform_intensive_computation();
    timer.stop();

    Console::println("Elapsed: " + timer.elapsed_millis() + " ms");
}
```

---

## 5. Cross-References

* **Specifications:** [Standard Library Formal Specifications](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/README.md)
* **Wiki Reference:** [Language Keywords](file:///home/jehud/Projects/solix-language/docs/wiki/keywords/README.md)
