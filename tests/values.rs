use kitaqfc::{assembler::image, codegen, json::Value, lowerer, parser};
use std::{fs, path::PathBuf};
#[test]
fn scalar_storage_arguments_signed_values_and_control_flow_match_reference_roms() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/values");
    let manifest =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap();
    let Value::Array(cases) = manifest else {
        panic!("fixture manifest");
    };
    assert_eq!(cases.len(), 61);
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("case");
        };
        let get = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k.as_str() == key)
                .map(|(_, v)| v)
                .unwrap()
        };
        let Value::String(name) = get("case") else {
            panic!("case name");
        };
        let Value::String(mapper) = get("mapper") else {
            panic!("case mapper");
        };
        let parsed = parser::parse_files(
            &[fixtures.join(format!("{name}.c"))],
            &[root.join("lib")],
            false,
        );
        assert!(
            parsed
                .diagnostics
                .iter()
                .all(|d| d.severity != kitaqfc::tokenizer::Severity::Error),
            "{name}: {:?}",
            parsed.diagnostics
        );
        let lowered = lowerer::lower(&parsed.tree, &lowerer::Options::default());
        assert!(
            lowered
                .diagnostics
                .iter()
                .all(|d| d.severity != kitaqfc::tokenizer::Severity::Error),
            "{name}: {:?}",
            lowered.diagnostics
        );
        let mut options = codegen::Options::default();
        options.image.cartridge.set_mapper(mapper).unwrap();
        let instructions =
            codegen::compile_all(&lowered.tree, &options).unwrap_or_else(|e| panic!("{name}: {e}"));
        let output = image::assemble(&instructions, &options.image).unwrap();
        assert_eq!(
            output.rom,
            fs::read(fixtures.join(format!("{name}.nes"))).unwrap(),
            "{name}: all ROM bytes"
        );
    }
}
