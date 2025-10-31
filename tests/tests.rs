const CCL: &str = r"bool = true
char = a
i32 = -1
u32 = 1
f32 = 1.0
str = str

none =
some = some

enum_unit =
    Unit =
enum_newtype =
    Newtype = true
enum_tuple =
    Tuple =
        = 0
        = 1
enum_struct =
    Struct =
        a = 0
        b = 1

array =
    = 0
    ignored =
    = 1
map =
    0 = 0
    1 = 1
";

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Struct {
    bool: bool,
    char: char,
    i32: i32,
    u32: u32,
    f32: f32,
    str: String,
    none: Option<String>,
    some: Option<String>,
    enum_unit: Enum,
    enum_newtype: Enum,
    enum_tuple: Enum,
    enum_struct: Enum,
    array: Vec<u32>,
    map: BTreeMap<u32, u32>,
}

#[derive(Deserialize)]
enum Enum {
    Unit,
    Newtype(bool),
    Tuple(u32, u32),
    Struct { a: u32, b: u32 },
}

#[test]
fn test_all() {
    let data = serde_ccl::from_str::<Struct>(CCL).unwrap();

    // Primitives
    assert_eq!(data.bool, true);
    assert_eq!(data.char, 'a');
    assert_eq!(data.i32, -1);
    assert_eq!(data.u32, 1);
    assert!(0.99 < data.f32 && data.f32 < 1.1);
    assert_eq!(data.str, "str");

    // Options
    assert!(data.none.is_none());
    assert!(data.some.is_some_and(|data| data == "some"));

    // Enums
    assert!(matches!(data.enum_unit, Enum::Unit));
    assert!(matches!(data.enum_newtype, Enum::Newtype(true)));
    assert!(matches!(data.enum_tuple, Enum::Tuple(0, 1)));
    assert!(matches!(data.enum_struct, Enum::Struct { a: 0, b: 1 }));

    // Aggregate
    assert_eq!(data.array, &[0, 1]);
    assert_eq!(data.map, BTreeMap::from_iter([(0, 0), (1, 1)]))
}
