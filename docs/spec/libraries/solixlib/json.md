# Solix Standard Library: JSON (`solix.json`)

The `solix.json` package provides high-performance JSON manipulation and serialization backed 1-to-1 by Rust's `serde_json` engine via native bridge bindings.

## 1. Class Hierarchy & Types

* **`JsonValue`**: Core JSON node representing null, boolean, integer, float, string, array, or object values.
* **`JsonException`**: Thrown when parsing malformed JSON, with line, column, and diagnostic code `[E0332]`.

## 2. API Reference

```solix
namespace solix.json;

public class JsonValue {
    public static JsonValue parse(string json_text);
    public string stringify(bool pretty);
    public string query(string json_pointer);

    public bool is_null();
    public bool is_boolean();
    public bool is_number();
    public bool is_string();
    public bool is_array();
    public bool is_object();

    public bool as_boolean();
    public int64 as_int64();
    public float64 as_float64();
    public string as_string();
}
```

## 3. Usage Example

```solix
import solix.json.JsonValue;
import solix.io.Console;

int32 main() {
    string payload = "{\"name\": \"Solix\", \"version\": 2, \"active\": true}";
    JsonValue root = JsonValue::parse(payload);

    string app_name = root.query("/name");
    Console::print_line("Application: " + app_name);

    string formatted = root.stringify(true);
    Console::print_line(formatted);
    return 0;
}
```
