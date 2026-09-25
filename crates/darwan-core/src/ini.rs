use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IniError {
    #[error("invalid key {0:?}: use letters, digits and _")]
    Key(String),
    #[error("value for {0:?} contains a line break or NUL")]
    Value(String),
}

// Mirrors QSettings: keys before any section header belong to General.
pub fn parse_general(text: &str) -> BTreeMap<String, String> {
    let mut section = "General".to_string();
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name.trim().to_string();
            continue;
        }
        if section != "General" {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            out.insert(k.trim().to_string(), unquote(v.trim()));
        }
    }
    out
}

pub fn write_general(values: &BTreeMap<String, String>) -> Result<String, IniError> {
    let mut out = String::from("[General]\n");
    for (k, v) in values {
        if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(IniError::Key(k.clone()));
        }
        if v.contains(['\n', '\r', '\0']) {
            return Err(IniError::Value(k.clone()));
        }
        out.push_str(k);
        out.push('=');
        out.push_str(&quote(v));
        out.push('\n');
    }
    Ok(out)
}

fn quote(v: &str) -> String {
    // QSettings reads a leading `@` as a type marker such as `@Variant(...)`.
    let v = if v.starts_with('@') {
        format!("@{v}")
    } else {
        v.to_string()
    };
    // Unquoted, a `,` makes QSettings return a list, which SDDM passes to QML as "".
    let plain =
        !v.is_empty() && v.trim() == v && !v.contains([',', ';', '"', '\\', '=', '#', '[', ']']);
    if plain {
        return v;
    }
    let escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn unquote(v: &str) -> String {
    let v = unescape(v);
    match v.strip_prefix("@@") {
        Some(rest) => format!("@{rest}"),
        None => v,
    }
}

fn unescape(v: &str) -> String {
    let Some(inner) = v.strip_prefix('"').and_then(|v| v.strip_suffix('"')) else {
        return v.to_string();
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn reads_only_general_and_skips_comments() {
        let text = "# c\n[General]\ngameMode=game\n; c\n[Other]\ngameMode=menu\n";
        assert_eq!(parse_general(text), map(&[("gameMode", "game")]));
    }

    #[test]
    fn keys_before_a_section_belong_to_general() {
        assert_eq!(parse_general("a=1\n"), map(&[("a", "1")]));
    }

    #[test]
    fn newline_in_a_value_is_rejected_because_it_would_inject_keys_into_a_root_written_file() {
        let err = write_general(&map(&[(
            "dateFormat",
            "x\n[General]\nbackground=/etc/shadow",
        )]));
        assert_eq!(err, Err(IniError::Value("dateFormat".into())));
    }

    #[test]
    fn key_with_bracket_or_equals_is_rejected() {
        assert!(write_general(&map(&[("a]b", "1")])).is_err());
        assert!(write_general(&map(&[("a=b", "1")])).is_err());
    }

    #[test]
    fn commas_are_quoted_so_qsettings_keeps_a_string_not_a_list() {
        let out = write_general(&map(&[("dateFormat", "ddd, MMM d")])).unwrap();
        assert_eq!(out, "[General]\ndateFormat=\"ddd, MMM d\"\n");
    }

    #[test]
    fn leading_at_is_doubled_so_sddm_never_deserialises_a_user_value_as_a_qvariant() {
        let out = write_general(&map(&[("dateFormat", "@Variant(AAAA)")])).unwrap();
        assert_eq!(out, "[General]\ndateFormat=@@Variant(AAAA)\n");
    }

    #[test]
    fn written_values_read_back_unchanged() {
        let values = map(&[
            ("a", "plain"),
            ("b", "with, comma"),
            ("c", "q\"uote\\"),
            ("d", " pad "),
            ("e", "@x"),
        ]);
        assert_eq!(parse_general(&write_general(&values).unwrap()), values);
    }
}
