use solcore_test_utils::fixture_path_key;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn fixture_checkout(root: &Path) -> (PathBuf, PathBuf) {
    let manifest = root.join("crates/test-crate");
    let source = root.join("tests/fixtures/case/main.sol");
    fs::create_dir_all(&manifest).unwrap();
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "contract C {}\n").unwrap();
    (manifest, source)
}

#[test]
fn fixture_keys_use_repository_relative_slashes() {
    let temp = tempfile::tempdir().unwrap();
    let (manifest, source) = fixture_checkout(temp.path());
    assert_eq!(
        fixture_path_key(manifest, source),
        "tests/fixtures/case/main.sol"
    );
}

#[cfg(unix)]
#[test]
fn fixture_keys_resolve_a_symlinked_checkout() {
    let temp = tempfile::tempdir().unwrap();
    let (manifest, source) = fixture_checkout(&temp.path().join("real"));
    let alias = temp.path().join("alias");
    std::os::unix::fs::symlink(temp.path().join("real"), &alias).unwrap();
    let alias_manifest = alias.join("crates/test-crate");
    assert_eq!(
        fixture_path_key(alias_manifest, source),
        "tests/fixtures/case/main.sol"
    );
    assert_eq!(
        fixture_path_key(manifest, alias.join("tests/fixtures/case/main.sol")),
        "tests/fixtures/case/main.sol"
    );
}
