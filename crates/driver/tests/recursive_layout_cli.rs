use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn recursive_runtime_layouts_are_rejected_before_backend_emission() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "solcore-recursive-layout-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    let input = dir.join("main.sol");

    for source in [
        include_str!("../../hull/tests/fixtures/layouts/recursive_construction/main.sol"),
        include_str!("../../hull/tests/fixtures/layouts/recursive_field_match/main.sol"),
    ] {
        fs::write(&input, source).expect("write source");
        for backend in ["--emit-yul", "--emit-sonatina"] {
            let output = Command::new(env!("CARGO_BIN_EXE_solcore-driver"))
                .arg(&input)
                .arg(backend)
                .arg("--color=never")
                .output()
                .expect("run driver");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.code(), Some(1), "{backend}: {stderr}");
            assert!(stderr.contains("error[SC0420]"), "{backend}: {stderr}");
            assert!(
                stderr.contains("cannot lower recursive type `Chain`"),
                "{backend}: {stderr}"
            );
            assert!(
                stderr.contains("recursive runtime values are not supported"),
                "{backend}: {stderr}"
            );
            assert!(
                !stderr.contains("translation failed"),
                "{backend}: {stderr}"
            );
            assert!(
                output.stdout.is_empty(),
                "{backend}: emitted backend output"
            );
        }
    }

    fs::remove_dir_all(&dir).expect("remove temp dir");
}
