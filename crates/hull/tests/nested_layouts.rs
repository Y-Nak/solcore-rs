use nameres::{Db as _, module_id_from_key};
use parser::parse_file_to_hir;
use solcore_hull::{EmitOptions, check_program_with_db, emit_module};
use solcore_test_utils::{define_frontend_test_db, load_main_source, run_in_large_stack};
use specialize::{SpecializeOptions, specialize_module};

define_frontend_test_db!(TestDb, hir_ty);

fn nested_source(depth: usize, mixed: bool, inline: bool) -> String {
    let mut source = String::from(
        "enum Box<a> { Box(a) }\n\
         enum Choice<a> { Empty, Choice(a) }\n\
         enum Product<a> { Product(word, a) }\n\
         struct Record<a> { value: a; }\n",
    );
    let mut ty = "word".to_owned();
    let mut expression = "input".to_owned();
    let mut pattern = "payload".to_owned();
    let mut statements = String::new();
    for index in 1..=depth {
        let ctor = if mixed {
            match index % 4 {
                0 => "Box",
                1 => "Choice",
                2 => "Product",
                _ => "Record",
            }
        } else {
            "Box"
        };
        ty = format!("{ctor}<{ty}>");
        let fields = if ctor == "Product" { "0, " } else { "" };
        expression = format!("{ctor}({fields}{expression})");
        let fields = if ctor == "Product" { "_, " } else { "" };
        pattern = format!("{ctor}({fields}{pattern})");
        if !inline {
            statements.push_str(&format!("let value{index}: {ty} = {expression};\n"));
            expression = format!("value{index}");
        }
    }
    source.push_str(&format!(
        "function unwrap(value: {ty}) returns (word) {{
            match (value) {{ case {pattern} {{ return payload; }} default {{ return 0; }} }}
         }}
         function main(input: word) returns (word) {{
            {statements}
            return unwrap({expression});
         }}"
    ));
    source
}

#[test]
fn deeply_nested_constructor_and_match_layouts_emit_and_check() {
    run_in_large_stack(|| {
        for mixed in [false, true] {
            for inline in [false, true] {
                let mut db = TestDb::default();
                let source = nested_source(24, mixed, inline);
                let key = load_main_source(&mut db, &source);
                let entry = module_id_from_key(&db, &key);
                let file = db.module_file(entry).expect("main source");
                let module = parse_file_to_hir(&db, file).module(&db);
                assert!(
                    parser::parse_diagnostics(&db, file).is_empty(),
                    "{source}\n{:?}",
                    parser::parse_diagnostics(&db, file)
                );
                let diagnostics = hir_ty::collect_frontend_diagnostics(&db, entry);
                assert!(
                    diagnostics
                        .iter()
                        .all(|diagnostic| diagnostic.level != hir::diag::DiagnosticLevel::Error),
                    "{diagnostics:?}"
                );
                let specialized = specialize_module(&db, module, SpecializeOptions::default());
                assert!(
                    specialized.diagnostics.is_empty(),
                    "{:?}",
                    specialized.diagnostics
                );
                let emitted = emit_module(&db, &specialized.module, EmitOptions::default());
                assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
                assert_eq!(check_program_with_db(&db, &emitted.program), Vec::new());
                assert!(!emitted.program.functions.is_empty());
            }
        }
    });
}

fn assert_layout_finiteness(source: &str, finite: bool) {
    let source = source.to_owned();
    run_in_large_stack(move || {
        let mut db = TestDb::default();
        let key = load_main_source(&mut db, &source);
        let entry = module_id_from_key(&db, &key);
        let file = db.module_file(entry).expect("main source");
        let module = parse_file_to_hir(&db, file).module(&db);
        assert!(
            parser::parse_diagnostics(&db, file).is_empty(),
            "{source}\n{:?}",
            parser::parse_diagnostics(&db, file)
        );
        let diagnostics = hir_ty::collect_frontend_diagnostics(&db, entry);
        assert!(
            diagnostics
                .iter()
                .all(|d| d.level != hir::diag::DiagnosticLevel::Error),
            "{source}\n{diagnostics:?}"
        );
        let specialized = specialize_module(&db, module, SpecializeOptions::default());
        assert!(
            specialized.diagnostics.is_empty(),
            "{source}\n{:?}",
            specialized.diagnostics
        );
        let emitted = emit_module(&db, &specialized.module, EmitOptions::default());
        if finite {
            assert!(
                emitted.diagnostics.is_empty(),
                "{source}\n{:?}",
                emitted.diagnostics
            );
            assert!(check_program_with_db(&db, &emitted.program).is_empty());
        } else {
            assert!(
                emitted.diagnostics.iter().any(|d| matches!(
                    d.kind,
                    solcore_hull::EmitDiagnosticKind::NonRegularRecursiveType { .. }
                )),
                "{source}\n{:?}",
                emitted.diagnostics
            );
        }
    });
}

#[test]
fn growing_recursive_instantiations_are_diagnosed() {
    for definitions in [
        "enum Nest<a> { Nil, Cons(a, Nest<Box<a>>) }",
        "type Wrapped(a) = Box<a>; enum Nest<a> { Nil, Cons(a, Nest<Wrapped<a>>) }",
        "enum Nest<a> { Nil, Cons(Box<Box<Nest<Box<a>>>>) }",
        "enum Nest<a> { Nil, Cons(a, (Nest<(a, a)>, word)) }",
        "enum Nest<a> { Nil, Next(Nest<comptime<a>>) }",
        "enum Nest<a> { Nil, Next(Nest<comptime<comptime<a>>>) }",
        "type Qualified(a) = comptime<a>; enum Nest<a> { Nil, Next(Nest<Qualified<a>>) }",
        "enum Nest<a> { Nil, Next(Box<Nest<comptime<a>>>) }",
    ] {
        assert_layout_finiteness(
            &format!(
                "enum Box<a> {{ Box(a) }} {definitions} function main(x: Nest<word>) returns (word) {{ return 0; }}"
            ),
            false,
        );
    }
}

#[test]
fn growing_cycles_can_cross_alternative_constructors() {
    for wrapper in ["Box<b>", "comptime<b>"] {
        assert_layout_finiteness(
            &format!(
                "enum Box<a> {{ Box(a) }}
                 enum Nest<a,b> {{ Nil, Left(Nest<{wrapper},word>), Right(Nest<word,a>) }}
                 function main(x: Nest<word,bool>) returns (word) {{ return 0; }}"
            ),
            false,
        );
    }
}

#[test]
fn regular_permuted_and_reset_recursion_remains_finite() {
    for definitions in [
        "enum Nest<a,b> { Nil, Next(a, Nest<a,b>) }",
        "enum Nest<a,b> { Nil, Next(a, Nest<b,a>) }",
        "enum Nest<a,b> { Nil, Next(a, Nest<Box<b>,word>) }",
        "enum Nest<a,b> { Nil, Next(a, Nest<comptime<b>,word>) }",
        "enum Nest<a,b> { Nil, Next(a, Nest<comptime<word>,b>) }",
        "enum Nest<a,b> { Nil, Next(Box<Box<Nest<a,b>>>) }",
        "enum Ignore<a> { Ignore } enum Nest<a,b> { Next(Ignore<Nest<Box<a>,b>>) }",
        "enum Ignore<a> { Ignore } enum Nest<a,b> { Next(Ignore<Nest<comptime<a>,b>>) }",
    ] {
        assert_layout_finiteness(
            &format!(
                "enum Box<a> {{ Box(a) }} {definitions} function main(x: Nest<word,bool>) returns (word) {{ return 0; }}"
            ),
            true,
        );
    }
}

#[test]
fn fixed_recursive_arguments_are_checked_only_when_the_representation_uses_them() {
    let definitions = "enum Box<a> { Box(a) } enum Ignore<a> { Ignore }
        enum Grow<a> { Stop, Next(Grow<Box<a>>) }";
    for (field, finite) in [("Box<Grow<word>>", false), ("Ignore<Grow<word>>", true)] {
        assert_layout_finiteness(
            &format!(
                "{definitions} enum Root {{ Root({field}) }}
            function main(x: Root) returns (word) {{ return 0; }}"
            ),
            finite,
        );
    }
}
