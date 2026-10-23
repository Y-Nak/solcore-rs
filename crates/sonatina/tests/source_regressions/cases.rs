// Expected behavior is expressed in Rust so fixture discovery only depends on source files.
use super::*;

pub(super) fn expectations(path: &str) -> Case {
    match path {
        "crates/sonatina/tests/fixtures/regressions/contract_memory/main.sol" => Case {
            checks: TextChecks {
                contains: vec![
                    "object @MemoryContractDeploy".into(),
                    "embed .runtime as &MemoryContract".into(),
                    "sym_addr &MemoryContract".into(),
                    "sym_size &MemoryContract".into(),
                    "evm_mstore ".into(),
                    "evm_mload ".into(),
                ],
                ..TextChecks::default()
            },
            compile: vec![Optimization::O0],
            ..Case::default()
        },
        "crates/sonatina/tests/fixtures/regressions/decoded_assembly_word/main.sol" => Case {
            checks: TextChecks {
                contains: vec![
                    "4540866244600635114649842549360310111892940575123159374096375843447573711370"
                        .into(),
                ],
                ..TextChecks::default()
            },
            ..Case::default()
        },
        "crates/sonatina/tests/fixtures/regressions/dynamic_memory_bounded/main.sol"
        | "crates/sonatina/tests/fixtures/regressions/dynamic_memory_unbounded/main.sol" => Case {
            compile: vec![Optimization::O0, Optimization::O2],
            compile_error: Some("dynamic length".into()),
            ..Case::default()
        },
        "crates/sonatina/tests/fixtures/regressions/entry_identity/main.sol" => Case {
            checks: TextChecks {
                regions: vec![solcore_test_utils::text_checks::TextRegion {
                    after: Some("func public %solcore_entry_12_root_2eruntime".into()),
                    before: None,
                    checks: TextChecks {
                        contains: vec!["main_5fmain_5fd".into()],
                        not_contains: vec!["main_5fmain_5fa_5fd".into()],
                        ..TextChecks::default()
                    },
                }],
                ..TextChecks::default()
            },
            deploy_output: Some(
                "0x000000000000000000000000000000000000000000000000000000000000002a".into(),
            ),
            ..Case::default()
        },
        "crates/sonatina/tests/fixtures/regressions/folded_cr/main.sol" => Case {
            checks: TextChecks {
                contains: vec!["return 4.i256".into()],
                ..TextChecks::default()
            },
            file_url_imports: true,
            ..Case::default()
        },
        "crates/sonatina/tests/fixtures/regressions/folded_crlf/main.sol" => Case {
            checks: TextChecks {
                contains: vec!["return 19.i256".into()],
                ..TextChecks::default()
            },
            file_url_imports: true,
            ..Case::default()
        },
        "tests/e2e/incremental-reachability/main.sol" => Case {
            compile: vec![Optimization::O0, Optimization::O2],
            file_url_imports: true,
            ..Case::default()
        },
        "tests/e2e/loop-post-reachability/main.sol" => Case {
            compile: vec![Optimization::O0],
            file_url_imports: true,
            ..Case::default()
        },
        "crates/sonatina/tests/fixtures/regressions/loop_terminator_invalid/main.sol"
        | "crates/sonatina/tests/fixtures/regressions/loop_terminator_return/main.sol"
        | "crates/sonatina/tests/fixtures/regressions/loop_terminator_revert/main.sol"
        | "crates/sonatina/tests/fixtures/regressions/loop_terminator_stop/main.sol" => {
            Case { ..Case::default() }
        }
        _ => panic!("missing expectations for source fixture {path}"),
    }
}
