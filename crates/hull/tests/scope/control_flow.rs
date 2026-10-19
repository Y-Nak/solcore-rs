use super::*;

#[test]
fn declarations_in_non_scoping_controls_reach_ordinary_assembly_and_closure_uses() {
    pretty_src_hull_with_std(
        "control_flow_local_lifetimes",
        include_str!("../../../../tests/e2e/control-flow-local-lifetimes/main.sol"),
    );
}

#[test]
fn hoisted_declarations_are_owned_by_each_specialized_function() {
    let hull = pretty_src_hull_with_std(
        "specialized_control_flow_locals",
        r#"
import * from std;
function value<a>(input: a, flag: bool) returns (word) {
    if (flag) { let result: word = 7; } else {}
    assembly { result := add(result, 1) }
    return result;
}
function main(flag: bool) returns (word) {
    let input: word = 0;
    return primAddWord(value(input, flag), value(true, flag));
}
"#,
    );
    assert!(hull.matches("let $if_local").count() >= 2, "{hull}");
}

#[test]
fn assembly_functions_do_not_capture_hoisted_core_locals() {
    let (db, output) = specialize_src(
        "assembly_function_no_core_capture",
        r#"
function main(flag: bool) returns (word) {
    if (flag) { let x: word = 7; } else {}
    let result: word;
    assembly {
        function read() -> result { result := x }
        result := read()
    }
    return result;
}
"#,
    );
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let emitted = emit_module(db, &output.module, EmitOptions::default());
    let diagnostics = check_program_with_db(db, &emitted.program);
    assert!(
        diagnostics.iter().any(|diagnostic| matches!(
            &diagnostic.kind,
            CheckDiagnosticKind::UndefinedVariable { name } if name == "x"
        )),
        "{diagnostics:?}"
    );
}

#[test]
fn generated_closure_captures_with_shared_source_spans_keep_distinct_bindings() {
    pretty_src_hull_with_std(
        "control_flow_capture_identity",
        include_str!("../../../../tests/e2e/control-flow-capture-identity/main.sol"),
    );
}
