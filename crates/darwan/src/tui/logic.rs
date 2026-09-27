use darwan_core::form::{Field, FieldKind};

// The value one step along a choice or boolean, or None for kinds edited as text.
pub fn step(field: &Field, forward: bool) -> Option<String> {
    match &field.kind {
        FieldKind::Bool => Some(
            if field.value == "true" {
                "false"
            } else {
                "true"
            }
            .to_string(),
        ),
        FieldKind::Choice(choices) if !choices.is_empty() => {
            let i = choices
                .iter()
                .position(|(v, _)| *v == field.value)
                .unwrap_or(0);
            let n = choices.len();
            let next = if forward {
                (i + 1) % n
            } else {
                (i + n - 1) % n
            };
            Some(choices[next].0.clone())
        }
        FieldKind::Int { min, max } => {
            let n: i64 = field.value.parse().unwrap_or(*min);
            let next = if forward {
                n.saturating_add(1)
            } else {
                n.saturating_sub(1)
            };
            Some(next.clamp(*min, *max).to_string())
        }
        FieldKind::Range { min, max, step } => {
            let n: f64 = field.value.parse().unwrap_or(*min);
            let next = if forward { n + step } else { n - step };
            // Snap to the step grid so repeated steps never drift into 0.30000000000000004.
            let snapped = min + ((next - min) / step).round() * step;
            Some(format_number(snapped.clamp(*min, *max)))
        }
        _ => None,
    }
}

fn format_number(n: f64) -> String {
    let s = format!("{n:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn display_value(field: &Field) -> String {
    match &field.kind {
        FieldKind::Choice(choices) => choices
            .iter()
            .find(|(v, _)| *v == field.value)
            .map_or(field.value.clone(), |(_, l)| l.clone()),
        FieldKind::Bool => if field.value == "true" { "on" } else { "off" }.into(),
        FieldKind::Text | FieldKind::Media(_) | FieldKind::Font | FieldKind::Color { .. }
            if field.value.is_empty() =>
        {
            "theme default".into()
        }
        FieldKind::Media(_) if field.value == "desktop" => "desktop wallpaper".into(),
        FieldKind::Color { .. } if field.value == "generate" => "generated".into(),
        _ => field.value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darwan_core::settings::Key;

    fn field(kind: FieldKind, value: &str) -> Field {
        Field {
            key: Key::ClockFormat,
            label: "x".into(),
            kind,
            value: value.into(),
            is_set: false,
            disabled: None,
            group: None,
        }
    }

    #[test]
    fn choices_wrap_both_ways() {
        let kind = FieldKind::Choice(vec![
            ("a".into(), "A".into()),
            ("b".into(), "B".into()),
            ("c".into(), "C".into()),
        ]);
        assert_eq!(step(&field(kind.clone(), "c"), true).as_deref(), Some("a"));
        assert_eq!(step(&field(kind, "a"), false).as_deref(), Some("c"));
    }

    #[test]
    fn ints_stay_inside_the_manifest_bounds() {
        let kind = FieldKind::Int { min: 1, max: 5 };
        assert_eq!(step(&field(kind.clone(), "5"), true).as_deref(), Some("5"));
        assert_eq!(step(&field(kind, "1"), false).as_deref(), Some("1"));
    }

    #[test]
    fn free_text_kinds_are_not_stepped() {
        assert_eq!(step(&field(FieldKind::Text, ""), true), None);
        assert_eq!(
            step(&field(FieldKind::Color { generate: false }, "#fff"), true),
            None
        );
    }

    #[test]
    fn values_are_shown_by_their_labels() {
        let kind = FieldKind::Choice(vec![("static".into(), "Fixed".into())]);
        assert_eq!(display_value(&field(kind, "static")), "Fixed");
        assert_eq!(display_value(&field(FieldKind::Bool, "true")), "on");
        assert_eq!(display_value(&field(FieldKind::Text, "")), "theme default");
    }

    #[test]
    fn ranges_step_on_their_grid_and_stay_in_bounds() {
        let speed = FieldKind::Range {
            min: 0.25,
            max: 3.0,
            step: 0.25,
        };
        assert_eq!(
            step(&field(speed.clone(), "1"), true).as_deref(),
            Some("1.25")
        );
        assert_eq!(
            step(&field(speed.clone(), "0.25"), false).as_deref(),
            Some("0.25")
        );
        assert_eq!(step(&field(speed, "3"), true).as_deref(), Some("3"));
        let contrast = FieldKind::Range {
            min: -1.0,
            max: 1.0,
            step: 0.25,
        };
        assert_eq!(step(&field(contrast, "0"), false).as_deref(), Some("-0.25"));
    }
}
