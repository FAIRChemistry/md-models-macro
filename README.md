# MD-Models Macro

![Build Status](https://github.com/JR-1991/mdmodels-macro/actions/workflows/test.yml/badge.svg)

This macro can be used to convert a markdown model using [MD-Models]() into Rust structs and enums. The resulting Rust code can be used to serialize and deserialize the model and integrate it into your Rust project.

## Installation

```bash
cargo install mdmodels-macro
```

## Example

Suppose you have a markdown file `model.md` with the following content:

````markdown
# Test

### Object

- string_value
  - Type: string
- enum_value
  - Type: SomeEnum

### SomeEnum

```python
VALUE = value
ANOTHER = another
```
````

You can convert this markdown file into Rust code using the following command:

```rust
use mdmodels_macro::parse_mdmodel;

// if you want to use the macro in the current module
parse_mdmodel!("tests/data/model.md");

// or if you want to use the macro in a module (e.g. in lib.rs)
mod my_module {
  parse_mdmodel!("tests/data/model.md");
}
```

At this point, the macro will generate the corresponding structs and enums in Rust code, which will be available as a module. The module name is derived from the title (`# Test`) as snake case, if present. Otherwise the module name will be `model`.

You can then use the module in your code:

### Non-builder pattern

```rust
fn main () {
    let obj = Object {
        string_value: "Hello, World!".to_string(),
        enum_value: model::SomeEnum::VALUE,
    };

    // Serialize the object
    let serialized = serde_json::to_string(&obj).unwrap();

    println!("Serialized: \n{}\n", serialized);

    // Deserialize the object
    let deserialized: Object = serde_json::from_str(&serialized).unwrap();

    println!("Deserialized: \n{:#?}\n", deserialized);
}
```

### Builder pattern

This macro also supports the builder pattern. To use the builder pattern, you need to use the `Builder` struct of the object in the markdown file:

```rust
fn main () -> Result<(), Box<dyn std::error::Error>> {
    let obj = ObjectBuilder::new()
        .string_value("Hello, World!")
        .enum_value(model::SomeEnum::VALUE)
        .build()?;

    // Serialize the object
    let serialized = serde_json::to_string(&obj).unwrap();

    println!("Serialized: \n{}\n", serialized);

    // Deserialize the object
    let deserialized: Object = serde_json::from_str(&serialized).unwrap();

    println!("Deserialized: \n{:#?}\n", deserialized);

    Ok(())
}
```
