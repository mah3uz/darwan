use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: String,
    pub family: Option<String>,
    pub author: String,
    pub background: Background,
    #[serde(default, rename = "font")]
    pub fonts: Vec<FontRequirement>,
    #[serde(default)]
    pub supports: Supports,
    #[serde(default, rename = "option")]
    pub options: Vec<ThemeOption>,
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    pub fn option(&self, key: &str) -> Option<&ThemeOption> {
        self.options.iter().find(|o| o.key == key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Background {
    Color,
    Image,
    Video,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontRequirement {
    pub file: String,
    pub family: String,
    pub license: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Supports {
    #[serde(default)]
    pub clock_format: bool,
    #[serde(default)]
    pub date_format: bool,
    #[serde(default)]
    pub background_override: bool,
}

// Flat, not a tagged enum: serde can't combine `flatten` with `deny_unknown_fields`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeOption {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: OptionKind,
    #[serde(default)]
    pub choices: Vec<Choice>,
    pub min: Option<i64>,
    pub max: Option<i64>,
    #[serde(default)]
    pub filters: Vec<String>,
    #[serde(default)]
    pub enabled_when: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptionKind {
    Bool,
    Enum,
    Int,
    Color,
    File,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

impl ThemeOption {
    pub fn check(&self, value: &str) -> Result<(), String> {
        match self.kind {
            OptionKind::Bool => match value {
                "true" | "false" => Ok(()),
                _ => Err(format!("expected true or false, got {value:?}")),
            },
            OptionKind::Enum => {
                if self.choices.iter().any(|c| c.value == value) {
                    Ok(())
                } else {
                    let allowed: Vec<&str> =
                        self.choices.iter().map(|c| c.value.as_str()).collect();
                    Err(format!(
                        "expected one of {}, got {value:?}",
                        allowed.join(", ")
                    ))
                }
            }
            OptionKind::Int => {
                let n: i64 = value
                    .parse()
                    .map_err(|_| format!("expected an integer, got {value:?}"))?;
                let (min, max) = (self.min.unwrap_or(i64::MIN), self.max.unwrap_or(i64::MAX));
                if (min..=max).contains(&n) {
                    Ok(())
                } else {
                    Err(format!("expected {min}..={max}, got {n}"))
                }
            }
            OptionKind::Color => {
                let hex = value.strip_prefix('#').unwrap_or("");
                if matches!(hex.len(), 3 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                    Ok(())
                } else {
                    Err(format!(
                        "expected #RGB, #RRGGBB or #AARRGGBB, got {value:?}"
                    ))
                }
            }
            OptionKind::File => {
                let lower = value.to_ascii_lowercase();
                if value.is_empty() {
                    Err("expected a file path".into())
                } else if self.filters.is_empty()
                    || self
                        .filters
                        .iter()
                        .any(|f| lower.ends_with(&f.to_ascii_lowercase()))
                {
                    Ok(())
                } else {
                    Err(format!(
                        "expected a file ending in {}, got {value:?}",
                        self.filters.join(", ")
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn option(extra: &str) -> ThemeOption {
        toml::from_str(&format!("key = \"k\"\nlabel = \"K\"\n{extra}")).unwrap()
    }

    #[test]
    fn unknown_manifest_field_is_rejected_so_typos_fail_loudly() {
        let err = Manifest::parse(
            "name = \"x\"\nauthor = \"a\"\nbackground = \"color\"\nnmae = \"typo\"\n",
        );
        assert!(err.is_err());
        let err = toml::from_str::<ThemeOption>(
            "key = \"k\"\nlabel = \"K\"\ntype = \"bool\"\nenabled_whn = {}\n",
        );
        assert!(err.is_err());
    }

    #[test]
    fn enum_accepts_only_declared_choices() {
        let o = option("type = \"enum\"\nchoices = [{ value = \"dark\", label = \"Dark\" }]");
        assert!(o.check("dark").is_ok());
        assert!(o.check("Dark").is_err());
    }

    #[test]
    fn bool_is_the_lowercase_string_themes_compare_against() {
        let o = option("type = \"bool\"");
        assert!(o.check("true").is_ok());
        assert!(
            o.check("True").is_err(),
            "orbital compares config.enableWindup !== \"false\""
        );
    }

    #[test]
    fn int_enforces_bounds() {
        let o = option("type = \"int\"\nmin = 1\nmax = 5");
        assert!(o.check("5").is_ok());
        assert!(o.check("6").is_err());
        assert!(o.check("x").is_err());
    }

    #[test]
    fn color_and_file_formats() {
        assert!(option("type = \"color\"").check("#1a1a4a").is_ok());
        assert!(option("type = \"color\"").check("red").is_err());
        let f = option("type = \"file\"\nfilters = [\".png\", \".jpg\"]");
        assert!(f.check("/home/u/Wall.PNG").is_ok());
        assert!(f.check("/home/u/wall.mp4").is_err());
    }
}
