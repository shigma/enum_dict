# enum_dict
 
[![Crates.io](https://img.shields.io/crates/v/enum_dict.svg)](https://crates.io/crates/enum_dict)
[![Documentation](https://docs.rs/enum_dict/badge.svg)](https://docs.rs/enum_dict)

A Rust library for efficient enum-indexed dictionaries.

## Installation
 
Add to your `Cargo.toml`:
 
```toml
[dependencies]
enum_dict = { version = "0.1", features = ["full"] }
```

## Quick Start

```rs
use enum_dict::{DictKey, RequiredDict, OptionalDict};

#[derive(DictKey)]
enum Color {
    Red,
    Green,
    Blue,
}

fn main() {
    // RequiredDict - all keys must have values
    let mut colors = RequiredDict::new(|color| match color {
        Color::Red => "#FF0000",
        Color::Green => "#00FF00",
        Color::Blue => "#0000FF",
    });

    // Direct indexing - no .get() needed!
    println!("Red hex: {}", colors[Color::Red]);

    // Mutable access
    colors[Color::Red] = "#FF0001";

    // OptionalDict - keys may or may not have values
    let mut favorite_colors = OptionalDict::<Color, String>::new();
    favorite_colors.set(Color::Blue, "Sky Blue".to_string());

    // Returns Option<&String>
    if let Some(favorite) = favorite_colors[Color::Blue] {
        println!("Favorite blue: {}", favorite);
    }
}
```

## Serde Support

With the serde feature enabled, `RequiredDict` and `OptionalDict` can be serialized and deserialized using [serde](https://serde.rs/):

```rs
use serde::{Serialize, Deserialize};

#[derive(DictKey)]
enum Locale {
    En,
    Fr,
    Jp,
    Zh,
}

#[derive(Serialize, Deserialize)]
struct I18nMessage {
    translations: RequiredDict<Locale, String>,
}
```

Extra keys in the serialized data are ignored during deserialization.

## Why `enum_dict`?

Compared to traditional `HashMap` approach, `enum_dict` uses `Vec` under the hood, allowing for:

- **Direct Indexing**: Access values with `dict[key]` instead of `dict.get(&key)`.
- **Performance**: Faster access times due to contiguous memory layout.
- **Type Safety**: Compile-time checks ensure all enum variants are handled.
- **Simplicity**: Less boilerplate code for common use cases.
