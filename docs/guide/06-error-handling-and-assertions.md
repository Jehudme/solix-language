# Guide 06: Error Handling & Assertions

This guide details structured exception handling with `try`/`catch`/`finally`, custom exception types, assertions, and monadic `Result<T, E>`.

---

## 1. Structured `try` / `catch` / `finally`

Exceptions represent exceptional runtime failures that unwind call stacks until caught:

```solix
extern import solix.io.Console;
extern import solix.core.IOException;

public void parse_configuration_file(string path_string) {
    try {
        read_file_data(path_string);
    } catch (IOException io_error) {
        Console::println("I/O error occurred: " + io_error.message());
    } catch (Exception general_error) {
        Console::println("General error: " + general_error.message());
    } finally {
        Console::println("Cleanup executed deterministically.");
    }
}
```

---

## 2. Defining Custom Exceptions

Custom exceptions inherit from `solix.core.Exception`:

```solix
public class ValidationException extends Exception {
    public ValidationException(string failure_reason) {
        super(failure_reason);
    }
}

public void validate_age(int32 candidate_age) {
    if (candidate_age < 0 || candidate_age > 150) {
        throw new ValidationException("Invalid age value provided");
    }
}
```

---

## 3. Invariant Assertions (`assert`)

Use `assert(condition, message)` to enforce internal program invariants during development:

```solix
public void process_transaction(int32 account_balance, int32 withdrawal_amount) {
    assert(withdrawal_amount > 0, "Withdrawal amount must be strictly positive");
    assert(account_balance >= withdrawal_amount, "Insufficient account balance");
}
```

If an assertion condition evaluates to `false`, the VM immediately prints the source coordinate failure and halts execution.

---

## 4. Cross-References

* **Specification:** [Zero-Cost Exception Handling](file:///home/jehud/Projects/solix-language/docs/spec/exception-unwinding.md)
* **Wiki Reference:** [Runtime Errors (`R0001`–`R0023`)](file:///home/jehud/Projects/solix-language/docs/wiki/errors/runtime-errors.md)
* **Next Guide:** [Guide 07: Generics & Variadic Templates](file:///home/jehud/Projects/solix-language/docs/guide/07-generics-and-variadic-templates.md)
