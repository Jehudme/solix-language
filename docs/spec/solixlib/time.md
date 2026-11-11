# Formal Specification: Standard Library Time & Date (`solix.time`)

## Document Metadata

* **Specification ID:** SPEC-0022
* **Scope:** Time Intervals, Monotonic Clocks, Wall-Clock Dates & Stopwatch
* **Status:** Final Normative Standard
* **Source Path:** [`libraries/solixlib/src/time/`](file:///home/jehud/Projects/solix-language/libraries/solixlib/src/time/)

---

## 1. Time Intervals (`solix.time.Duration`)

`Duration` represents a signed time span with nanosecond precision:
* **Constructors:** `static Duration from_nanos(int64 nanos)`, `from_micros`, `from_millis`, `from_seconds`, `from_minutes`, `from_hours`.
* **Accessors:** `int64 as_nanos()`, `int64 as_micros()`, `int64 as_millis()`, `float64 as_seconds()`.
* **Arithmetic:** `Duration operator+(Duration other)`, `Duration operator-(Duration other)`.

---

## 2. Monotonic Clock (`solix.time.Instant`)

`Instant` measures an opaque monotonic time stamp guaranteed never to jump backwards due to system clock adjustments:
* `static Instant now()`: Captures current monotonic timestamp.
* `Duration elapsed()`: Returns duration elapsed since this instant was captured.
* `Duration duration_since(Instant earlier)`: Computes difference between two points in time.

---

## 3. Wall-Clock Dates (`solix.time.DateTime`)

`DateTime` represents human calendar dates and times:
* `static DateTime utc_now()`: Current UTC time.
* `static DateTime now()`: Current local system time.
* **Fields:** `int32 year()`, `int32 month()`, `int32 day()`, `int32 hour()`, `int32 minute()`, `int32 second()`, `int32 millisecond()`.
* `String to_iso8601()`: Formats date as ISO-8601 string (`"2026-11-10T21:30:00Z"`).

---

## 4. Benchmarking Timer (`solix.time.Stopwatch`)

`Stopwatch` facilitates high-resolution benchmarking and profiling:
* `static Stopwatch start_new()`: Creates and immediately begins running stopwatch.
* `void start()` / `void stop()` / `void reset()` / `void restart()`.
* `bool is_running()`
* `Duration elapsed()`
* `int64 elapsed_millis()`
* `int64 elapsed_micros()`
