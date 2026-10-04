use kitaqfc::{assembler::image, codegen, json::Value, lowerer, parser};
use std::{fs, path::PathBuf};

#[test]
fn accepted_original_smoke_sources_match_complete_reference_roms() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/source_roms");
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("case manifest");
    };
    let mut accepted = 0;
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("case");
        };
        if !matches!(fields.get("reference_accepted"), Some(Value::Bool(true))) {
            continue;
        }
        let Some(Value::String(name)) = fields.get("source") else {
            panic!("source");
        };
        let parsed = parser::parse_files(
            &[root.join("tests/frontend").join(name)],
            &[root.join("lib"), root.clone()],
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
        options.image.cartridge.set_mapper("mmc3").unwrap();
        let assembly =
            codegen::compile_all(&lowered.tree, &options).unwrap_or_else(|e| panic!("{name}: {e}"));
        let image = image::assemble(&assembly, &options.image).unwrap();
        let golden = fixtures.join(format!("{}.nes", name.strip_suffix(".c").unwrap()));
        assert_eq!(image.rom, fs::read(golden).unwrap(), "{name}: full ROM");
        accepted += 1;
    }
    assert_eq!(accepted, 15);
}
