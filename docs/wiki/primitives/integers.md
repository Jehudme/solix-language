# Primitive Integers Reference

Solix provides signed and unsigned fixed-width integer types with explicit two's complement arithmetic and bounds checking:

---

## 1. Signed Integers

| Type | Bit Width | Range | Default | Friendly Alias |
| :--- | :---: | :--- | :---: | :--- |
| `int8` | 8 bits | $-128$ to $127$ | `0` | `sbyte` |
| `int16` | 16 bits | $-32,768$ to $32,767$ | `0` | `short` |
| `int32` | 32 bits | $-2,147,483,648$ to $2,147,483,647$ | `0` | `int` |
| `int64` | 64 bits | $-2^{63}$ to $2^{63}-1$ | `0` | `long` |

```solix
int32 counter = 100;
int64 large_timestamp = 1762809000L;
```

---

## 2. Unsigned Integers

| Type | Bit Width | Range | Default | Friendly Alias |
| :--- | :---: | :--- | :---: | :--- |
| `uint8` | 8 bits | $0$ to $255$ | `0` | `byte` |
| `uint16` | 16 bits | $0$ to $65,535$ | `0` | `ushort` |
| `uint32` | 32 bits | $0$ to $4,294,967,295$ | `0` | `uint` |
| `uint64` | 64 bits | $0$ to $2^{64}-1$ | `0` | `ulong` |

```solix
uint8 byte_mask = 0xFF;
uint32 memory_offset = 4096U;
```
