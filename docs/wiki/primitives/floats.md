# Primitive Floating-Point Types Reference

Solix provides IEEE-754 floating-point types for scientific, graphical, and high-performance numeric computing:

---

| Type | Bit Width | Precision | Range | Default | Friendly Alias |
| :--- | :---: | :---: | :--- | :---: | :--- |
| `float32` | 32 bits | ~7 decimal digits | $\pm 1.18 \times 10^{-38}$ to $\pm 3.4 \times 10^{38}$ | `0.0f` | `float` |
| `float64` | 64 bits | ~15-17 decimal digits | $\pm 2.23 \times 10^{-308}$ to $\pm 1.8 \times 10^{308}$ | `0.0` | `double` |

```solix
float32 single_coordinate = 3.14f;
float64 double_precision = 2.718281828459045;
```

* **Literals:** Floating literals default to `float64`. Append `f` or `F` for `float32`.
* **Bit-Exact Constant Storage:** In compiled `.slxb` containers, floating constants are serialized by raw bit representations to prevent cross-platform floating-point drift.
