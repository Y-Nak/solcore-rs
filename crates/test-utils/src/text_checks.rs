//! Declarative assertions for textual compiler output in directory fixtures.

use std::collections::BTreeMap;

#[derive(Default)]
pub struct TextChecks {
    pub contains: Vec<String>,
    pub not_contains: Vec<String>,
    pub count: BTreeMap<String, usize>,
    pub min_count: BTreeMap<String, usize>,
    pub in_order: Vec<String>,
    pub regions: Vec<TextRegion>,
}

pub struct TextRegion {
    pub after: Option<String>,
    pub before: Option<String>,
    pub checks: TextChecks,
}

impl TextChecks {
    /// Read simple text assertions embedded next to their source fixture.
    /// Needles use quoted strings so newlines and backslashes are unambiguous.
    /// Suite-specific options are handled by the caller.
    pub fn from_directives(source: &str, namespace: &str) -> Self {
        let mut checks = Self::default();
        for (line, directive) in crate::source_directives(source, namespace) {
            let (command, argument) = directive.split_once(' ').unwrap_or((directive, ""));
            match command {
                "contains" | "not-contains" | "in-order" => {
                    let needle = directive_string(argument, line);
                    match command {
                        "contains" => checks.contains.push(needle),
                        "not-contains" => checks.not_contains.push(needle),
                        _ => checks.in_order.push(needle),
                    }
                }
                "count" | "min-count" => {
                    let (count, needle) = argument
                        .split_once(' ')
                        .unwrap_or_else(|| panic!("line {line}: expected count and quoted needle"));
                    let count = count
                        .parse()
                        .unwrap_or_else(|_| panic!("line {line}: invalid count"));
                    let counts = if command == "count" {
                        &mut checks.count
                    } else {
                        &mut checks.min_count
                    };
                    assert!(
                        counts
                            .insert(directive_string(needle, line), count)
                            .is_none(),
                        "line {line}: duplicate count expectation"
                    );
                }
                _ => {}
            }
        }
        checks
    }

    pub fn assert_matches(&self, text: &str) {
        for needle in &self.contains {
            assert!(text.contains(needle), "missing {needle:?}:\n{text}");
        }
        for needle in &self.not_contains {
            assert!(!text.contains(needle), "unexpected {needle:?}:\n{text}");
        }
        for (needle, expected) in &self.count {
            assert_eq!(
                text.matches(needle).count(),
                *expected,
                "count of {needle:?}:\n{text}"
            );
        }
        for (needle, minimum) in &self.min_count {
            assert!(
                text.matches(needle).count() >= *minimum,
                "expected at least {minimum} occurrences of {needle:?}:\n{text}"
            );
        }
        let mut remaining = text;
        for needle in &self.in_order {
            let (_, suffix) = remaining
                .split_once(needle)
                .unwrap_or_else(|| panic!("missing ordered {needle:?}:\n{text}"));
            remaining = suffix;
        }
        for region in &self.regions {
            let mut selected = text;
            if let Some(after) = &region.after {
                selected = selected
                    .split_once(after)
                    .unwrap_or_else(|| panic!("missing region start {after:?}:\n{text}"))
                    .1;
            }
            if let Some(before) = &region.before {
                selected = selected
                    .split_once(before)
                    .unwrap_or_else(|| panic!("missing region end {before:?}:\n{text}"))
                    .0;
            }
            region.checks.assert_matches(selected);
        }
    }
}

/// Decode a nonempty quoted string in a source expectation.
pub fn directive_string(argument: &str, line: usize) -> String {
    let needle: String = serde_json::from_str(argument)
        .unwrap_or_else(|error| panic!("line {line}: expected a quoted string: {error}"));
    assert!(!needle.is_empty(), "line {line}: empty text expectation");
    needle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_expectations_preserve_escaping_counts_and_order() {
        let source = r#"
//~ other: contains "ignored"
//~ hull: contains "a\n\"b\\c"
//~ hull: not-contains "missing"
//~ hull: count 2 "repeat"
//~ hull: min-count 1 "b\\c"
//~ hull: in-order "start"
//~ hull: in-order "end"
//~ hull: loader fixture
"#;
        TextChecks::from_directives(source, "hull")
            .assert_matches("start a\n\"b\\c repeat repeat end");
    }

    #[test]
    #[should_panic(expected = "duplicate count expectation")]
    fn source_expectations_reject_duplicate_counts() {
        TextChecks::from_directives("//~ hull: count 1 \"x\"\n//~ hull: count 2 \"x\"", "hull");
    }

    #[test]
    #[should_panic(expected = "expected a quoted string")]
    fn source_expectations_reject_malformed_strings() {
        TextChecks::from_directives("//~ hull: contains not-quoted", "hull");
    }
}
