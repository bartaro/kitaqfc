use kitaqfc::{assembly_json, codegen};
use std::path::PathBuf;
#[test]
fn all_native_helpers_and_mapper_sequences_match_reference() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/helpers");
    for (name, mapper, zp, guard) in [
        ("nrom", "nrom", false, true),
        ("uxrom", "uxrom", false, true),
        ("cnrom", "cnrom", false, true),
        ("axrom", "axrom", false, true),
        ("mmc1", "mmc1", false, true),
        ("mmc3", "mmc3", false, true),
        ("mmc5", "mmc5", false, true),
        ("vrc6", "vrc6", false, true),
        ("vrc7", "vrc7", false, true),
        ("fme7", "fme7", false, true),
        ("fds", "fds", false, true),
        ("surom512", "surom512", false, true),
        ("nrom-zp", "nrom", true, true),
        ("fds-zp-no-guard", "fds", true, false),
    ] {
        let mut options = codegen::Options::default();
        options.image.cartridge.set_mapper(mapper).unwrap();
        options.zero_page = zp;
        options.fds_guard = guard;
        for method in ["EmitIntrinsicHelpers", "EmitMapperSwitchSequence"] {
            let fixture = if method == "EmitMapperSwitchSequence" || zp {
                name
            } else if mapper == "fds" {
                "fds"
            } else {
                "nrom"
            };
            let expected = assembly_json::parse(
                &kitaqfc::io::read_utf8(root.join(format!("{fixture}.{method}.json"))).unwrap(),
            )
            .unwrap();
            let actual = if method == "EmitIntrinsicHelpers" {
                codegen::runtime_helpers(&options).unwrap()
            } else {
                codegen::mapper_sequence(&options).unwrap()
            };
            assert_eq!(
                actual.len(),
                expected.len(),
                "{name} {method}: emitted nodes"
            );
            for (i, (a, b)) in actual.iter().zip(&expected).enumerate() {
                assert_eq!(a, b, "{name} {method} node {i}");
            }
        }
    }
}
