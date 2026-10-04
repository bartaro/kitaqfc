use kitaqfc::{assembler::image, codegen, json::Value, lowerer, parser, tokenizer::Severity};
use std::{fs, path::PathBuf};
#[test]
fn every_public_intrinsic_compiles_to_the_reference_rom() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/intrinsics");
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("manifest");
    };
    assert_eq!(cases.len(), 158);
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("case");
        };
        let Value::String(name) = fields.get("case").unwrap() else {
            panic!("name");
        };
        let Value::String(mapper) = fields.get("mapper").unwrap() else {
            panic!("mapper");
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
                .all(|d| d.severity != Severity::Error),
            "{name}: {:?}",
            parsed.diagnostics
        );
        let lowered = lowerer::lower(&parsed.tree, &lowerer::Options::default());
        assert!(
            lowered
                .diagnostics
                .iter()
                .all(|d| d.severity != Severity::Error),
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
