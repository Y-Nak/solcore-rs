use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use dir_test::{Fixture, dir_test};
use solcore_test_utils::run_in_large_stack;

#[path = "support/frontend.rs"]
mod frontend;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Phase {
    Frontend,
    Typeck,
}

#[derive(Debug)]
struct KnownDiagnostic {
    phase: Phase,
    prefix: String,
    reason: String,
}

struct Manifest {
    verdicts: BTreeMap<String, String>,
    accepted_divergences: BTreeMap<String, Vec<KnownDiagnostic>>,
    rejected_divergences: BTreeMap<String, String>,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/../parser/tests/fixtures/corpus/ok", glob: "**/*.sol")]
fn reference_accepted_corpus(fixture: Fixture<&str>) {
    let path = PathBuf::from(fixture.path())
        .canonicalize()
        .expect("corpus source path");
    run_in_large_stack(move || {
        let manifest = manifest();
        let root = corpus_root();
        let relative = path.strip_prefix(root.join("ok")).unwrap();
        let report_path = accepted_report_path(relative);
        let (main_root, external_roots) = roots_for_entry(&root.join("ok"), relative);
        let outcome = frontend::run_frontend_with_roots(
            &path,
            &main_root,
            &root.join("ok/std"),
            external_roots,
        );
        let actual = outcome
            .frontend_error_diagnostics
            .iter()
            .map(|diagnostic| (Phase::Frontend, diagnostic))
            .chain(
                outcome
                    .typeck_error_diagnostics
                    .iter()
                    .map(|diagnostic| (Phase::Typeck, diagnostic)),
            )
            .collect::<Vec<_>>();
        let known = manifest
            .accepted_divergences
            .get(&report_path)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let unrecorded = actual
            .iter()
            .filter(|(phase, diagnostic)| {
                !known
                    .iter()
                    .any(|known| known.phase == *phase && diagnostic.starts_with(&known.prefix))
            })
            .collect::<Vec<_>>();
        let stale = known
            .iter()
            .filter(|known| {
                !actual.iter().any(|(phase, diagnostic)| {
                    *phase == known.phase && diagnostic.starts_with(&known.prefix)
                })
            })
            .map(|known| (&known.phase, &known.prefix, &known.reason))
            .collect::<Vec<_>>();
        assert!(
            unrecorded.is_empty() && stale.is_empty(),
            "{report_path}: reference-accepted full-frontend parity changed\n  unrecorded errors: {unrecorded:#?}\n  stale known divergences: {stale:#?}"
        );
    });
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/../parser/tests/fixtures/corpus/fail/test/examples", glob: "**/*.sol")]
fn reference_rejected_corpus(fixture: Fixture<&str>) {
    let path = PathBuf::from(fixture.path())
        .canonicalize()
        .expect("corpus source path");
    run_in_large_stack(move || {
        let manifest = manifest();
        let root = corpus_root();
        let relative = path.strip_prefix(root.join("fail/test/examples")).unwrap();
        let report_path = path_key(relative);
        if manifest.verdicts[&report_path] == "timeout" {
            // Timeouts belong to the rejected corpus, but have no frontend
            // verdict to compare. The manifest still checks their presence.
            return;
        }
        let relative = Path::new("test/examples").join(relative);
        let (main_root, external_roots) = roots_for_entry(&root.join("fail"), &relative);
        let outcome = frontend::run_frontend_with_roots(
            &path,
            &main_root,
            &root.join("ok/std"),
            external_roots,
        );
        let accepted = !outcome.frontend_has_errors && !outcome.typeck_has_errors;
        let known = manifest.rejected_divergences.get(&report_path);
        assert_eq!(
            accepted,
            known.is_some(),
            "{report_path}: reference-rejected corpus parity changed\n  known divergence: {known:?}\n  frontend errors: {:#?}\n  typeck errors: {:#?}",
            outcome.frontend_error_diagnostics,
            outcome.typeck_error_diagnostics,
        );
    });
}

fn manifest() -> &'static Manifest {
    static MANIFEST: OnceLock<Manifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        let root = corpus_root();
        let mut verdicts = BTreeMap::new();
        for fields in manifest_rows(
            &root.join("reference-frontend.tsv"),
            "path\tstatus\tcode",
            3,
        ) {
            assert!(
                matches!(fields[1].as_str(), "pass" | "fail" | "timeout"),
                "invalid reference status for {}: {}",
                fields[0],
                fields[1]
            );
            assert!(
                verdicts
                    .insert(fields[0].clone(), fields[1].clone())
                    .is_none(),
                "duplicate reference verdict for {}",
                fields[0]
            );
        }
        let accepted = verdicts
            .iter()
            .filter(|(_, status)| status.as_str() == "pass")
            .map(|(path, _)| path.clone())
            .collect::<BTreeSet<_>>();
        let rejected = verdicts
            .iter()
            .filter(|(_, status)| status.as_str() != "pass")
            .map(|(path, _)| path.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            accepted,
            relative_sources(&root.join("ok/test/examples")),
            "reference pass manifest and accepted corpus differ"
        );
        assert_eq!(
            rejected,
            relative_sources(&root.join("fail/test/examples")),
            "reference reject/timeout manifest and fail corpus differ"
        );

        let tested_paths = accepted
            .iter()
            .map(|path| format!("test/examples/{path}"))
            .chain(
                relative_sources(&root.join("ok/test/imports"))
                    .into_iter()
                    .map(|path| format!("test/imports/{path}")),
            )
            .chain(
                relative_sources(&root.join("ok/std"))
                    .into_iter()
                    .map(|path| format!("std/{path}")),
            )
            .collect::<BTreeSet<_>>();
        assert_eq!(
            tested_paths,
            relative_sources(&root.join("ok")),
            "full-frontend gate and accepted corpus tree differ"
        );
        let report_paths = tested_paths
            .iter()
            .map(|path| accepted_report_path(Path::new(path)))
            .collect::<BTreeSet<_>>();

        let mut accepted_divergences = BTreeMap::<String, Vec<KnownDiagnostic>>::new();
        let mut divergence_keys = BTreeSet::new();
        for fields in manifest_rows(
            &root.join("rust-rejected-reference-passes.tsv"),
            "# path<TAB>phase<TAB>diagnostic-prefix<TAB>reason",
            4,
        ) {
            let phase = match fields[1].as_str() {
                "frontend" => Phase::Frontend,
                "typeck" => Phase::Typeck,
                other => panic!("invalid accepted-corpus phase: {other}"),
            };
            assert!(
                report_paths.contains(&fields[0]),
                "accepted-corpus divergence manifest contains non-pass file: {}",
                fields[0]
            );
            assert!(
                divergence_keys.insert((fields[0].clone(), phase, fields[2].clone())),
                "duplicate accepted-corpus divergence for {}",
                fields[0]
            );
            accepted_divergences
                .entry(fields[0].clone())
                .or_default()
                .push(KnownDiagnostic {
                    phase,
                    prefix: fields[2].clone(),
                    reason: fields[3].clone(),
                });
        }

        let mut rejected_divergences = BTreeMap::new();
        for fields in manifest_rows(
            &root.join("rust-accepted-reference-failures.tsv"),
            "# path<TAB>reason",
            2,
        ) {
            assert_eq!(
                verdicts.get(&fields[0]).map(String::as_str),
                Some("fail"),
                "stale reference-rejected divergence for {}",
                fields[0]
            );
            assert!(
                rejected_divergences
                    .insert(fields[0].clone(), fields[1].clone())
                    .is_none(),
                "duplicate known divergence for {}",
                fields[0]
            );
        }
        Manifest {
            verdicts,
            accepted_divergences,
            rejected_divergences,
        }
    })
}

fn manifest_rows(path: &Path, header: &str, columns: usize) -> Vec<Vec<String>> {
    let content = fs::read_to_string(path).expect("corpus manifest");
    let mut lines = content.lines();
    assert_eq!(
        lines.next(),
        Some(header),
        "invalid manifest header: {}",
        path.display()
    );
    lines
        .enumerate()
        .filter_map(|(index, line)| {
            if line.is_empty() && header.starts_with('#') {
                return None;
            }
            let fields = line.split('\t').map(str::to_owned).collect::<Vec<_>>();
            assert_eq!(
                fields.len(),
                columns,
                "invalid manifest row {} in {}",
                index + 2,
                path.display()
            );
            for field in &fields {
                assert!(
                    !field.is_empty() && field.trim() == field,
                    "invalid manifest field on row {} in {}",
                    index + 2,
                    path.display()
                );
            }
            Some(fields)
        })
        .collect()
}

fn accepted_report_path(relative: &Path) -> String {
    path_key(
        relative
            .strip_prefix("test/examples")
            .or_else(|_| relative.strip_prefix("test"))
            .unwrap_or(relative),
    )
}

fn path_key(relative: &Path) -> String {
    relative
        .iter()
        .map(|component| component.to_str().expect("UTF-8 corpus path"))
        .collect::<Vec<_>>()
        .join("/")
}

fn roots_for_entry(root: &Path, relative: &Path) -> (PathBuf, BTreeMap<String, PathBuf>) {
    let mut external = BTreeMap::new();
    let main_root = if relative.starts_with("std") {
        root.join("std")
    } else if relative.starts_with("test/imports/extlib") {
        external.insert("extlib".to_owned(), root.join("test/imports/extlib"));
        root.join("test/imports/extlib")
    } else if relative.starts_with("test/imports") {
        external.insert("extlib".to_owned(), root.join("test/imports/extlib"));
        root.join("test/imports")
    } else {
        // Example suites use their category directory as the import root.
        let path = relative
            .strip_prefix("test/examples")
            .expect("known corpus area");
        let category = path.components().next().unwrap().as_os_str();
        if [
            "cases",
            "comptime",
            "dispatch",
            "invokable",
            "opcodes",
            "pragmas",
            "spec",
        ]
        .iter()
        .any(|name| category == *name)
        {
            root.join("test/examples").join(category)
        } else {
            root.join("test/examples")
        }
    };
    (main_root, external)
}

fn relative_sources(root: &Path) -> BTreeSet<String> {
    fn walk(root: &Path, directory: &Path, paths: &mut BTreeSet<String>) {
        for entry in fs::read_dir(directory).expect("corpus directory") {
            let path = entry.expect("corpus entry").path();
            if path.is_dir() {
                walk(root, &path, paths);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("sol") {
                assert!(paths.insert(path_key(path.strip_prefix(root).unwrap())));
            }
        }
    }
    let mut paths = BTreeSet::new();
    walk(root, root, &mut paths);
    paths
}

fn corpus_root() -> PathBuf {
    frontend::repo_root()
        .join("crates/parser/tests/fixtures/corpus")
        .canonicalize()
        .expect("corpus root")
}
