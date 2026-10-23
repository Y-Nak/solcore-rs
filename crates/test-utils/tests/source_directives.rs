use solcore_test_utils::source_directives;

#[test]
fn selects_the_suite_and_preserves_line_numbers_and_arguments() {
    let source = "// unrelated\n  //~ hull: contains \"return 7\"\n//~ sonatina: compile O0\n//~ hull : valid\n";
    assert_eq!(
        source_directives(source, "hull").collect::<Vec<_>>(),
        [(2, "contains \"return 7\""), (4, "valid")]
    );
}

#[test]
#[should_panic(expected = "line 2: expected //~ hull: expectation")]
fn rejects_a_missing_namespace_separator() {
    source_directives("// unrelated\n//~ hull contains \"return 7\"", "hull").count();
}
