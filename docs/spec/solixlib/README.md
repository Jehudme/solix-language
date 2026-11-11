# Solix Standard Library (`solixlib`) v2.0 Formal Specifications

The official Solix standard library (`solixlib`) provides high-performance, deterministic standard types, data structures, and system primitives:

* [**Core (`core.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/core.md): Standard interfaces (`IStringable`, `IEquatable<T>`, `IComparable<T>`, `ICloneable<T>`), primitive utilities (`Primitives`), heap strings (`String`), mutable string builder (`StringBuilder`), monadic wrappers (`Optional<T>`, `Result<T, E>`), and exception classes.
* [**Collections (`collections.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/collections.md): Linear containers (`List`, `LinkedList`, `Deque`, `Queue`, `Stack`, `CircularBuffer`), associative maps (`HashMap`, `TreeMap`), sets (`HashSet`, `TreeSet`, `BitSet`), and sorting algorithms (`Algorithms`).
* [**I/O & File System (`io.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/io.md): `Console`, streams (`IStream`, `MemoryStream`, `FileStream`), stream readers/writers, `Path`, `File`, and `Directory`.
* [**Mathematics & Randomness (`math.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/math.md): Floating-point and integer transcendentals (`Math`), and deterministic `xoshiro256**` pseudo-random number generator (`Random`).
* [**Time & Date (`time.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/time.md): High-precision `Duration`, monotonic `Instant`, wall-clock `DateTime`, and benchmarking `Stopwatch`.
* [**System & Process (`system.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/system.md): `Environment` inspection, process command execution (`Process`), and process results (`ProcessResult`).
* [**Cryptography & Encoding (`crypto.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/crypto.md): SHA-256, SHA-1, MD5 hashing (`Hash`), URL-safe Base64 (`Base64`), and hexadecimal serialization (`Hex`).
