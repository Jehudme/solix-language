# Guide 04: Classes, OOP & Namespaces

This guide covers object-oriented programming in Solix, single inheritance, virtual method dispatch, interface contracts, operator overloading, and qualified namespaces.

---

## 1. Class Declarations & Constructors

```solix
public class Point2D {
    public float64 coord_x;
    public float64 coord_y;

    public Point2D(float64 initial_x, float64 initial_y) {
        this.coord_x = initial_x;
        this.coord_y = initial_y;
    }

    public float64 magnitude_squared() {
        return (this.coord_x * this.coord_x) + (this.coord_y * this.coord_y);
    }
}
```

---

## 2. Inheritance & Virtual Dispatch

Solix supports single class inheritance. Overriding methods must be explicitly annotated with `virtual` in the base class and `override` in the derived class:

```solix
public class Shape {
    public virtual float64 compute_area() {
        return 0.0;
    }
}

public class Circle extends Shape {
    private float64 radius;

    public Circle(float64 radius_value) {
        this.radius = radius_value;
    }

    public override float64 compute_area() {
        return 3.1415926535 * this.radius * this.radius;
    }
}
```

---

## 3. Interfaces & Multiple Contracts

```solix
public interface IDrawable {
    public void draw();
}

public class Button implements IDrawable {
    public void draw() {
        Console::println("[Button Rendered]");
    }
}
```

---

## 4. Static Members & Scope Resolution (`::`)

All static fields, static methods, and enum variants must be accessed using the scope resolution operator `::`:

```solix
public class MathUtility {
    public static const float64 EPSILON = 0.00001;

    public static bool nearly_equal(float64 first, float64 second) {
        float64 difference = first - second;
        if (difference < 0.0) difference = -difference;
        return difference < MathUtility::EPSILON;
    }
}

public int32 main() {
    bool is_close = MathUtility::nearly_equal(1.0, 1.000001);
    return 0;
}
```

---

## 5. Cross-References

* **Specification:** [Declarations & OOP Members](file:///home/jehud/Projects/solix-language/docs/spec/statements/declarations.md)
* **Wiki Reference:** [Modifier Keywords](file:///home/jehud/Projects/solix-language/docs/wiki/keywords/modifiers.md)
* **Next Guide:** [Guide 05: Memory Management, ARC & RAII](file:///home/jehud/Projects/solix-language/docs/guide/05-memory-management-arc-and-raii.md)
