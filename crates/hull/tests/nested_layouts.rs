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
                assert!(parser::parse_diagnostics(&db, file).is_empty());
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
