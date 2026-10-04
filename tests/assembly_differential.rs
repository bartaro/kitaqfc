use kitaqfc::{assembler::image, assembly_json};
use std::{fs, path::PathBuf};
#[test]
fn assembler_matches_reference_image_for_all_cartridge_profiles() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/assembly");
    for name in [
        "nrom",
        "uxrom",
        "cnrom",
        "axrom",
        "mmc1",
        "mmc3",
        "mmc5",
        "vrc6",
        "vrc7",
        "fme7",
        "surom512",
        "fds_legacy",
    ] {
        let nodes = assembly_json::parse(
            &kitaqfc::io::read_utf8(root.join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        let mut options = image::Options::default();
        options
            .cartridge
            .set_mapper(if name == "fds_legacy" { "fds" } else { name })
            .unwrap();
        if name == "fds_legacy" {
            options.fds_ram = false
        }
        let actual = image::assemble(&nodes, &options).unwrap();
        let expected = fs::read(root.join(format!("{name}.nes"))).unwrap();
        assert_eq!(actual.rom.len(), expected.len(), "{name}: image size");
        if let Some(i) = actual.rom.iter().zip(&expected).position(|(a, b)| a != b) {
            panic!(
                "{name}: byte mismatch at {i:#x}: Rust={:02X} reference={:02X}",
                actual.rom[i], expected[i]
            )
        }
    }
}
