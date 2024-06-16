# jsonparser

A hand written JSON parser in Rust in under 500 lines of code.

This parser reads JSON source from a string and construct a custom JSONValue object in memory.

```rust
enum JSONValue {
    Null,
    True,
    False,
    Number(f64),
    String(String),
    Array(Vec<JSONValue>),
    Object(HashMap<String, JSONValue>),
}
```

All the source code is in the `src/main.rs` file, including:

- Enum definitions for JSONValue and JSONParseError
- The Parser
- Unit tests
- Quick benchmarks

There is an accompanying blog post for this on my website at [https://krishkrish.com/blog/json-parser-in-rust](https://krishkrish.com/blog/json-parser-in-rust)

## Usage

To use this parser, you can call the `parse` function with a JSON source string.

```rust
let json = r#"
{
    "name": "John Doe",
    "age": 30,
    "is_student": false,
    "marks": [90, 80, 85],
    "address": {
        "street": "123 Main St",
        "city": "New York"
    }
}

"#;

let json_value = jsonparser::parse(json).unwrap();
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [JSON.org](https://www.json.org/json-en.html)
- [serde-rs's json-benchmark](https://github.com/serde-rs/json-benchmark/tree/master/data)
