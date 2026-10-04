use kitaqfc::{io, ir_json, json::Value, lowerer, parser, tokenizer::Severity};
use std::{fs, path::PathBuf};

fn sources() -> (PathBuf, Vec<PathBuf>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cases = fs::read_dir(root.join("tests/frontend"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "c"))
        .collect::<Vec<_>>();
    cases.sort();
    assert_eq!(cases.len(), 29);
    (root, cases)
}

fn known_rejection(name: &str) -> Option<Vec<(u16, usize, usize)>> {
    let line = match name {
        "kitaqfc_phase20_intrinsics_smoke.c" => 29,
        "kitaqfc_phase22_fds_metadata_smoke.c" => 13,
        _ => return None,
    };
    Some(vec![
        (1000, line - 1, 10),
        (1000, line - 1, 11),
        (1000, line - 1, 12),
        (1002, line + 2, 1),
    ])
}

fn verify(lower: bool) {
    let (root, cases) = sources();
    for source in cases {
        let name = source.file_name().unwrap().to_str().unwrap();
        let parsed =
            parser::parse_files(&[source.clone()], &[root.join("lib"), root.clone()], false);
        let errors = parsed
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect::<Vec<_>>();
        if let Some(expected) = known_rejection(name) {
            let actual = errors
                .iter()
                .map(|d| (d.code, d.position.line, d.position.column))
                .collect::<Vec<_>>();
            assert_eq!(
                actual, expected,
                "{name}: reference dialect rejects the same input"
            );
            continue;
        }
        assert!(errors.is_empty(), "{name}: {errors:?}");
        let tree = if lower {
            let output = lowerer::lower(&parsed.tree, &lowerer::Options::default());
            assert!(
                !output
                    .diagnostics
                    .iter()
                    .any(|d| d.severity == Severity::Error),
                "{name}: {:?}",
                output.diagnostics
            );
            output.tree
        } else {
            parsed.tree
        };
        let expected = Value::parse(
            &io::read_utf8(source.with_file_name(format!(
                "{name}.{}.json",
                if lower { "lower" } else { "parse" }
            )))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            ir_json::expr(&tree),
            expected,
            "{name}: complete types, nodes, values and source coordinates"
        );
    }
}

#[test]
fn parser_matches_reference_fc_dialect() {
    verify(false);
}

#[test]
fn lowering_matches_reference_fc_temporaries_and_control_flow() {
    verify(true);
}
