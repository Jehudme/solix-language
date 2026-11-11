# Guide 03: Control Flow & Functions

This guide details branching with `if`/`else` and `switch`, loop constructs, free functions, default parameter values, function pointers, and first-class lambdas.

---

## 1. Branching & Conditionals

### 1.1 `if` / `else`
Conditions must evaluate to a strict `bool` expression:

```solix
int32 score = 85;

if (score >= 90) {
    Console::println("Grade: A");
} else if (score >= 80) {
    Console::println("Grade: B");
} else {
    Console::println("Grade: C");
}
```

### 1.2 `switch` / `case`
Discrete multi-way branching over integer or enum values:

```solix
int32 command_code = 2;

switch (command_code) {
    case 1:
        start_engine();
        break;
    case 2:
        stop_engine();
        break;
    default:
        Console::println("Unknown command");
        break;
}
```

---

## 2. Iteration & Loops

```solix
// while loop
int32 counter = 0;
while (counter < 5) {
    counter = counter + 1;
}

// do / while loop
do {
    process_message();
} while (has_pending_messages());

// for loop
for (int32 index = 0; index < 10; index = index + 1) {
    if (index == 4) continue;
    if (index == 8) break;
}
```

---

## 3. Functions & Default Parameters

Free functions can be declared at module scope:

```solix
int32 multiply(int32 factor_a, int32 factor_b = 2) {
    return factor_a * factor_b;
}

public int32 main() {
    int32 result_one = multiply(5);     // Evaluates to 10
    int32 result_two = multiply(5, 3);  // Evaluates to 15
    return 0;
}
```

---

## 4. Function Pointers & Lambdas

Functions are first-class values in Solix:

```solix
// Function pointer type alias
alias BinaryOperation = int32(*)(int32, int32);

int32 add(int32 left, int32 right) { return left + right; }

public int32 main() {
    BinaryOperation op = add;
    int32 result = op(10, 20);

    // Lambda closure
    BinaryOperation multiply_lambda = (int32 left, int32 right) => left * right;
    int32 product = multiply_lambda(10, 20);
    return 0;
}
```

---

## 5. Cross-References

* **Specification:** [Control Flow Statements](file:///home/jehud/Projects/solix-language/docs/spec/statements/control-flow.md)
* **Wiki Reference:** [Control Flow Keywords](file:///home/jehud/Projects/solix-language/docs/wiki/keywords/control_flow.md)
* **Next Guide:** [Guide 04: Classes, OOP & Namespaces](file:///home/jehud/Projects/solix-language/docs/guide/04-classes-oop-and-namespaces.md)
