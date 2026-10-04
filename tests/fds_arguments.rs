use kitaqfc::{assembler::image, codegen, fds::disk, json::Value, lowerer, parser};
use std::{fs, path::PathBuf};

#[test]
fn fds_overlay_argument_preservation_fastcall_and_guard_match_reference_images() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/fds_arguments");
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("case manifest");
    };
    assert_eq!(cases.len(), 4);
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("case");
        };
        let Some(Value::String(name)) = fields.get("case") else {
            panic!("name");
        };
        let Some(Value::Array(flags)) = fields.get("flags") else {
            panic!("flags");
        };
        let flags = flags
            .iter()
            .map(|v| {
                let Value::String(s) = v else {
                    panic!("flag");
                };
                s.as_str()
            })
            .collect::<Vec<_>>();
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
        options.image.cartridge.set_mapper("fds").unwrap();
        options.image.fds.trim_fill = true;
        options.fast_call = flags.contains(&"--fastcall-v2");
        options.fds_guard = !flags.contains(&"--fds-no-overlay-guard");
        let assembly = codegen::compile_all(&lowered.tree, &options).unwrap();
        let image = image::assemble(&assembly, &options.image).unwrap();
        assert_eq!(
            image.rom,
            fs::read(fixtures.join(format!("{name}.nes"))).unwrap(),
            "{name}: ROM"
        );
        options.image.fds.disk.native_layout = true;
        let disk = disk::build(
            &image.prg,
            &image.chr,
            &options.image.fds.disk,
            &image.fds_overlays,
        )
        .unwrap();
        assert_eq!(
            disk.image,
            fs::read(fixtures.join(format!("{name}.fds"))).unwrap(),
            "{name}: disk"
        );
    }
}
