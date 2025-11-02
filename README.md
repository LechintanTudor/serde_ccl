# Serde CCL

[![Crates.io](https://img.shields.io/crates/v/serde_ccl)](https://crates.io/crates/serde_ccl)
[![Documentation](https://docs.rs/serde_ccl/badge.svg)](https://docs.rs/serde_ccl)

[Serde](https://crates.io/crates/serde)-based crate for deserializing
[CCL Documents](https://chshersh.com/blog/2025-01-06-the-most-elegant-configuration-language.html).

## Example

CCL document named `example.ccl`.

```text
imports =
    = ~/.config/terminal/theme.ccl
    = ~/.config/terminal/font.ccl

dynamic_title = false
font_size = 12
shell = tmux new-session -A -s main
```

Code to deserialize the CCL document.

```rust
use serde::Deserialize;

const DOCUMENT = include_str!("example.ccl");

#[derive(Debug, Deserialize)]
struct Config {
    imports: Vec<String>,  
    dynamic_title: bool,
    font_size: f64,
    shell: String,
}

fn main() {
    let config = serde_ccl::from_str::<Config>(DOCUMENT).unwrap();
    println!("{config:?}");
}
```

## Other Examples

### Deserializing Arrays

Arrays are deserialized as key-value pairs where the key is empty. Non-empty
keys are ignored.

```rust
use serde::Deserialize;

const CCL: &str = r"
values =
    = 0
    ignored = 1
    = 1
";

#[derive(Deserialize)]
struct Data {
    values: Vec<i32>,
}

fn main() {
    let data = serde_ccl::from_str::<Data>(CCL).unwrap();
    assert_eq!(data.values, &[0, 1]);
}
```

### Deserializing Enums

Enums are deserialized as key-value pairs where the key is the variant name and
the value is the payload.

```rust
use serde::Deserialize;

const CCL: &str = r"
none =
    None =
rgb =
    Rgb =
        = 10
        = 20
        = 30
";

#[derive(Deserialize)]
struct Data {
    none: Color,
    rgb: Color,
}

#[derive(Deserialize)]
enum Color {
    None,
    Rgb(u8, u8, u8),
}

fn main() {
    let data = serde_ccl::from_str::<Data>(CCL).unwrap();
    assert!(matches!(data.none, Color::None));
    assert!(matches!(data.rgb, Color::Rgb(10, 20, 30)));
}
```

### Deserializing Unit Enums

For enums containing only unit variants it's more convenient to deserialize them
from strings instead of key-value pairs. This can be achieved using a simple
macro.

```rust
use serde::Deserialize;

macro_rules! define_enum {
    ($Name:ident { $($Variant:ident => $repr:literal,)* }) => {
        #[derive(Deserialize)]
        #[serde(try_from = "&str")]
        pub enum $Name {
            $($Variant,)*
        }

        impl TryFrom<&str> for $Name {
            type Error = &'static str;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                Ok(match s {
                    $($repr => Self::$Variant,)*
                    _ => return Err("invalid variant"),
                })
            }
        }
    };
}

const CCL: &str = r"
theme = light
";

define_enum!(Theme {
    Light => "light",
    Dark => "dark",
});

#[derive(Deserialize)]
struct Data {
    theme: Theme,
}

#[test]
fn test_enum_from_str() {
    let data = serde_ccl::from_str::<Data>(CCL).unwrap();
    assert!(matches!(data.theme, Theme::Light));
```

## License

serde_ccl is dual-licensed under either

- MIT License ([LICENSE-MIT](LICENSE-MIT) or
  [https://opensource.org/license/mit/](https://opensource.org/license/mit/))

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))

at your option.

<br />

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above without any additional terms or conditions.
