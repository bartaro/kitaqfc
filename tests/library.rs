use kitaqfc::{assembler::image, codegen, json::Value, lowerer, parser};
use std::{fs, path::PathBuf};

#[test]
fn current_library_headers_and_entity_callbacks_match_reference_roms() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/library");
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("case manifest");
    };
    assert_eq!(cases.len(), 2);
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("case");
        };
        let Some(Value::String(name)) = fields.get("case") else {
            panic!("case name");
        };
        let Some(Value::String(mapper)) = fields.get("mapper") else {
            panic!("mapper");
        };
        let Some(Value::Array(linked)) = fields.get("linked") else {
            panic!("linked modules");
        };
        let mut sources = vec![fixtures.join(format!("{name}.c"))];
        sources.extend(linked.iter().map(|v| {
            let Value::String(name) = v else {
                panic!("linked source");
            };
            root.join("lib").join(name)
        }));
        let parsed = parser::parse_files(&sources, &[root.join("lib")], false);
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
        let assembly =
            codegen::compile_all(&lowered.tree, &options).unwrap_or_else(|e| panic!("{name}: {e}"));
        let image = image::assemble(&assembly, &options.image).unwrap();
        assert_eq!(
            image.rom,
            fs::read(fixtures.join(format!("{name}.nes"))).unwrap(),
            "{name}: full ROM"
        );
    }
}
