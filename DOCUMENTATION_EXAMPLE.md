# Generated Documentation Example

This file shows an example of the RustDoc documentation that the `#[actor]` macro automatically generates.

## Sample Actor Code

```rust
#[derive(Debug, Clone)]
pub struct DocumentedActor {
    pub name: String,
    pub count: i32,
}

#[actor]
impl DocumentedActor {
    /// Add two numbers together
    /// This method performs basic arithmetic addition
    pub async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    
    /// Greet someone with a personalized message
    pub async fn greet(&self, name: String) -> String {
        format!("Hello {}, I'm {}!", name, self.name)
    }
    
    /// A method with no parameters
    pub async fn ping(&self) -> String {
        "pong".to_string()
    }
}
```

## Generated Documentation

The `#[actor]` macro generates documentation that looks like this:

---

**Actor implementation for `DocumentedActor`.**

This implementation provides JSON-based method dispatch for the following methods:

| Method | Parameters | Return Type |
|--------|------------|-------------|
| `add` | `a`: `i32`, `b`: `i32` | `i32` |
| `greet` | `name`: `String` | `String` |
| `ping` | None | `String` |

## Method Details

### `add`

Add two numbers together
This method performs basic arithmetic addition

**Parameters:**
- `a`: `i32`
- `b`: `i32`

**Returns:** `i32`

**JSON Payload:**
```json
{
  "a": 42,
  "b": 42
}
```

**Usage Example:**
```rust
let result = actor.dispatch("add", r#"{"a": 42, "b": 42}"#);
```

### `greet`

Greet someone with a personalized message

**Parameters:**
- `name`: `String`

**Returns:** `String`

**JSON Payload:**
```json
{
  "name": "example"
}
```

**Usage Example:**
```rust
let result = actor.dispatch("greet", r#"{"name": "example"}"#);
```

### `ping`

A method with no parameters

**Parameters:** None

**Returns:** `String`

**JSON Payload:**
```json
{}
```

**Usage Example:**
```rust
let result = actor.dispatch("ping", "{}");
```

## Usage Notes

- All methods are called via the `dispatch(method_name, json_params)` function
- Parameters must be provided as a JSON string
- Return values are serialized as JSON strings
- Unknown method names will return an error message
- Invalid JSON parameters will return a parse error message

---

## Benefits

This auto-generated documentation provides:

1. **Complete API Reference**: Every available method is documented with its signature
2. **JSON Payload Examples**: Shows exactly what JSON to send for each method
3. **Usage Examples**: Copy-paste ready code examples
4. **Type Information**: Clear parameter and return types
5. **Method Documentation**: Preserves your original doc comments
6. **Error Handling Info**: Explains what happens with invalid inputs

To view this documentation in your browser, run:
```bash
cargo doc --open
```
