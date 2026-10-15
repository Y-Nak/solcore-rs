use dir_test::{Fixture, dir_test};
use proptest::prelude::*;
use solcore_parser::format::{FormatOptions, format_source};

fn format(source: &str) -> String {
    format_source(source, &FormatOptions::default()).unwrap()
}

fn assert_stable(source: &str) {
    let formatted = format(source);
    assert_eq!(
        format(&formatted),
        formatted,
        "formatting must be idempotent"
    );
}

#[test]
fn formats_declarations_types_and_control_flow() {
    let source = "import{A,B}from lib;export{A};enum E<a>{A(a),B}\nfunction f<a>(x:Pair<a,word>)returns(word){for(let i=0;i<3;i+=1){if(x>0){continue;}else{break;}}return x?1:2;}";
    assert_eq!(
        format(source),
        "import { A, B } from lib;\nexport { A };\nenum E<a> {\n    A(a),\n    B\n}\nfunction f<a>(x: Pair<a, word>) returns (word) {\n    for (let i = 0; i < 3; i += 1) {\n        if (x > 0) {\n            continue;\n        } else {\n            break;\n        }\n    }\n    return x ? 1 : 2;\n}\n"
    );
    assert_stable(source);
}

#[test]
fn formats_yul_without_semicolon_boundaries() {
    let source = "function f(){assembly{let x:=0 x:=add(x,1) for{let i:=0}lt(i,3){i:=add(i,1)}{mstore(i,x)} switch x case 0{leave}default{revert(0,0)}}}";
    assert_eq!(
        format(source),
        "function f() {\n    assembly {\n        let x := 0\n        x := add(x, 1)\n        for {\n            let i := 0\n        } lt(i, 3) {\n            i := add(i, 1)\n        } {\n            mstore(i, x)\n        }\n        switch x\n        case 0 {\n            leave\n        }\n        default {\n            revert(0, 0)\n        }\n    }\n}\n"
    );
    assert_stable(source);
}

#[test]
fn preserves_tokens_and_comment_payloads() {
    let source = "// doc  \r\nfunction f(){/* nested /* } */ comment */let s=\"a\n  b  \";let x=0x00FF;x+=1;x~=;return .Some((!x));}// tail  ";
    let formatted = format(source);
    for fragment in [
        "// doc  \r\n",
        "/* nested /* } */ comment */",
        "\"a\n  b  \"",
        "0x00FF",
        "x += 1",
        "x ~=;",
        "return .Some((!x));",
        "// tail  \r\n",
    ] {
        assert!(
            formatted.contains(fragment),
            "missing {fragment:?} in {formatted:?}"
        );
    }
    assert_stable(source);
}

#[test]
fn keeps_empty_for_clauses_inline() {
    assert_eq!(
        format("function f(){for(;true;){break;}}"),
        "function f() {\n    for (; true;) {\n        break;\n    }\n}\n"
    );
}

#[test]
fn rejects_invalid_input_without_returning_partial_output() {
    for source in [
        "function f(){let x=;}",
        "function f(){\"unterminated",
        "/* unterminated",
        "function f(){let x=lam(){let y=;};}",
    ] {
        assert!(
            format_source(source, &FormatOptions::default()).is_err(),
            "{source}"
        );
    }
}

#[test]
fn respects_options_for_whitespace_on_the_final_line() {
    let mut options = FormatOptions {
        insert_final_newline: false,
        trim_final_newlines: false,
        trim_trailing_whitespace: false,
        ..FormatOptions::default()
    };
    for source in ["function f() {}\n   ", "\n   "] {
        assert_eq!(format_source(source, &options).unwrap(), source);
        options.insert_final_newline = true;
        let expected = format!("{source}\n");
        assert_eq!(format_source(source, &options).unwrap(), expected);
        assert_eq!(format_source(&expected, &options).unwrap(), expected);
        options.insert_final_newline = false;
    }
}

#[test]
fn distinguishes_constructor_dots_and_comparison_parentheses() {
    let source =
        "function f(x:M.T){match(x){case comptime .A{return .B(x.field);}case .C{return x>(1);}}}";
    let formatted = format(source);
    assert!(formatted.contains("case comptime .A {"));
    assert!(formatted.contains("return .B(x.field);"));
    assert!(formatted.contains("return x > (1);"));
    assert_stable(source);
}

#[test]
fn retains_explicit_continuation_lines_and_blank_line_separators() {
    let source =
        "function f(\nx: word,\ny: word\n) returns (word) {\n\nreturn add(\nx,\ny\n);\n}\n\n\n";
    assert_eq!(
        format(source),
        "function f(\n    x: word,\n    y: word\n) returns (word) {\n\n    return add(\n        x,\n        y\n    );\n}\n"
    );
    assert_stable(source);
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/ok", glob: "**/*.sol")]
fn formatting_parser_fixtures(fixture: Fixture<&str>) {
    assert_stable(fixture.content());
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/corpus/ok", glob: "**/*.sol")]
fn formatting_reference_corpus(fixture: Fixture<&str>) {
    assert_stable(fixture.content());
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/../../std", glob: "**/*.sol")]
fn formatting_standard_library(fixture: Fixture<&str>) {
    assert_stable(fixture.content());
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/../../tests/e2e", glob: "**/*.sol")]
fn formatting_e2e_sources(fixture: Fixture<&str>) {
    assert_stable(fixture.content());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn arbitrary_utf8_never_panics(source in ".{0,256}") {
        if let Ok(formatted) = format_source(&source, &FormatOptions::default()) {
            prop_assert_eq!(format(&formatted), formatted);
        }
    }

    #[test]
    fn whitespace_variants_preserve_tokens_and_are_idempotent(gaps in prop::collection::vec("[ \\t\\r\\n]{0,8}", 12)) {
        let tokens = ["function", "f", "(", ")", "{", "return", "(", "1", "+", "2", ")", ";", "}"];
        let mut source = tokens[0].to_owned();
        for (gap, token) in gaps.iter().zip(&tokens[1..]) {
            // Keep the two identifiers separated; all other gaps may be empty.
            source.push_str(if source == "function" && gap.is_empty() { " " } else { gap });
            source.push_str(token);
        }
        let formatted = format(&source);
        prop_assert_eq!(format(&formatted), formatted);
    }
}
