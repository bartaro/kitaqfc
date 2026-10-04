use kitaqfc::{assembler::image, codegen, json::Value, lowerer, parser};
use std::{fs, path::PathBuf};

#[test]
fn surom_interrupt_call_graph_and_mapper_writes_match_reference_constraints() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/surom");
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("case manifest");
    };
    assert_eq!(cases.len(), 6);
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("case");
        };
        let Some(Value::String(name)) = fields.get("case") else {
            panic!("name");
        };
        let parsed = parser::parse_files(
            &[fixtures.join(format!("{name}.c"))],
            &[root.join("lib")],
            false,
        );
        assert!(parsed.diagnostics.is_empty());
        let lowered = lowerer::lower(&parsed.tree, &lowerer::Options::default());
        assert!(
            lowered
                .diagnostics
                .iter()
                .all(|d| d.severity != kitaqfc::tokenizer::Severity::Error)
        );
        let mut options = codegen::Options::default();
        options.image.cartridge.set_mapper("surom512").unwrap();
        let result = codegen::compile_all(&lowered.tree, &options);
        if let Some(Value::String(code)) = fields.get("expected_error_code") {
            let error = result.unwrap_err();
            assert!(error.contains(code), "{name}: {error}");
        } else {
            let image = image::assemble(&result.unwrap(), &options.image).unwrap();
            assert_eq!(
                image.rom,
                fs::read(fixtures.join(format!("{name}.nes"))).unwrap(),
                "{name}: ROM"
            );
        }
    }
}
