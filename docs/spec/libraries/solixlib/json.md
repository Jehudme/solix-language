# Solix Standard Library: JSON (`solix.json`)

The `solix.json` package provides high-performance JSON manipulation and serialization backed 1-to-1 by Rust's `serde_json` engine via native bridge bindings.

## 1. Class Hierarchy & Types

* **`JsonValue`**: Core JSON node representing null, boolean, integer, float, string, array, or object values, with RFC 6901 pointer querying and builder helpers.
* **`JsonObject`**: Fluent builder for JSON objects (`set_null`, `set_bool`, `set_int`, `set_float`, `set_string`, `set_value`, `build`).
* **`JsonArray`**: Fluent builder for JSON arrays (`add_null`, `add_bool`, `add_int`, `add_float`, `add_string`, `add_value`, `build`).
* **`JsonException`**: Thrown when parsing malformed JSON, with line, column, and diagnostic code `[E0332]`.

## 2. API Reference

```solix
package solix.json;

public class JsonValue {
    public static JsonValue parse(string json_text);
    public static JsonValue parse(String json_text);
    public static string escape_pointer_segment(string segment);
    public static String escape_pointer_segment(String segment);

    public string stringify();
    public string stringify(bool pretty);
    public String query(string json_pointer);
    public String query(String json_pointer);
    public bool has(string json_pointer);
    public bool has(String json_pointer);
    public JsonValue get(string json_pointer);
    public JsonValue get(String json_pointer);
    public JsonValue at(int32 index);
    public int32 size();
    public string[] keys();

    public bool is_null();
    public bool is_bool();
    public bool is_boolean();
    public bool is_number();
    public bool is_string();
    public bool is_array();
    public bool is_object();

    public bool as_bool();
    public bool as_boolean();
    public int64 as_int64();
    public float64 as_float64();
    public String as_string();
    public String get_string_or(string json_pointer, string fallback);
    public int64 get_int64_or(string json_pointer, int64 fallback);
    public float64 get_float64_or(string json_pointer, float64 fallback);
    public bool get_bool_or(string json_pointer, bool fallback);
}
```

> [!NOTE]
> `JsonValue::query` distinguishes missing JSON pointers (`null` return) from present JSON `null` (`"null"`) and empty strings (`"\"\""`), and `as_string()` unescapes JSON strings cleanly via `serde_json`. `escape_pointer_segment` encodes `~` as `~0` and `/` as `~1` per RFC 6901.

## 3. Usage Example

```solix
import solix.json.JsonValue;
import solix.json.JsonObject;
import solix.io.Console;

int32 main() {
    string payload = "{\"name\": \"Solix\", \"version\": 3, \"active\": true}";
    JsonValue root = JsonValue::parse(payload);

    JsonValue name_node = root.get("/name");
    Console::println("Application: " + name_node.as_string().data);

    JsonValue built = new JsonObject()
        .set_string("language", "Solix")
        .set_int("major", 3)
        .build();
    Console::println(built.stringify(true));
    return 0;
}
```
