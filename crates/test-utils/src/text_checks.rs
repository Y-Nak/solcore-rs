//! Declarative assertions for textual compiler output in directory fixtures.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TextChecks {
    pub contains: Vec<String>,
    pub not_contains: Vec<String>,
    pub count: BTreeMap<String, usize>,
    pub min_count: BTreeMap<String, usize>,
    pub in_order: Vec<String>,
    pub regions: Vec<TextRegion>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextRegion {
    pub after: Option<String>,
    pub before: Option<String>,
    pub checks: TextChecks,
}

impl TextChecks {
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
