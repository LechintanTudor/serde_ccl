# Serde CCL

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
  let config = serde_ccl::from_str::<Config>(DOCUMENT)
    .expect("Failed to parse document");

  println!("{config:?}");
}
```

## License

Sparsey is dual-licensed under either

- MIT License ([LICENSE-MIT](LICENSE-MIT) or
  [https://opensource.org/license/mit/](https://opensource.org/license/mit/))

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))

at your option.

<br />

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above without any additional terms or conditions.
