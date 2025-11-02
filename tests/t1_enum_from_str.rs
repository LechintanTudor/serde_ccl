use serde::Deserialize;

const CCL: &str = r"
theme = light
";

const CCL_ERR: &str = r"
theme = invalid
";

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

define_enum!(Theme {
    Light => "light",
    Dark => "dark",
});

#[derive(Deserialize)]
struct Config {
    theme: Theme,
}

#[test]
fn test_enum_from_str() {
    let config = serde_ccl::from_str::<Config>(CCL).unwrap();
    assert!(matches!(config.theme, Theme::Light));

    let config_err = serde_ccl::from_str::<Config>(CCL_ERR);
    assert!(config_err.is_err());
}
