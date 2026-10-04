use kitaqfc::{assembler::image, codegen, lowerer, parser};
use std::{fs, path::PathBuf};
#[test]
fn native_c_to_rom_boot_matches_all_twelve_reference_profiles() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let parsed = parser::parse_files(
        &[root.join("tests/codegen/boot.c")],
        &[root.join("lib")],
        false,
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let tree = lowerer::lower(&parsed.tree, &lowerer::Options::default()).tree;
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
        let mut options = codegen::Options::default();
        options
            .image
            .cartridge
            .set_mapper(if name == "fds_legacy" { "fds" } else { name })
            .unwrap();
        if name == "fds_legacy" {
            options.image.fds_ram = false
        }
        let assembly = codegen::compile_all(&tree, &options).unwrap();
        let output = image::assemble(&assembly, &options.image).unwrap();
        assert_eq!(
            output.rom,
            fs::read(root.join(format!("tests/assembly/{name}.nes"))).unwrap(),
            "{name}: full native C-to-ROM path"
        );
    }
}
#[test]
fn native_c_to_fds_matches_reference_for_overlays_and_two_sides() {
    use kitaqfc::fds::{Metadata, disk};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cases = root.join("tests/fds");
    for name in [
        "native",
        "no_bypass",
        "raw",
        "legacy",
        "overlay",
        "overlay_trim",
        "metadata",
        "metadata_overlay",
    ] {
        let parsed = parser::parse_files(
            &[cases.join(format!("{name}.c"))],
            &[root.join("lib")],
            false,
        );
        assert!(parsed.diagnostics.is_empty());
        let tree = lowerer::lower(&parsed.tree, &lowerer::Options::default()).tree;
        let mut options = codegen::Options::default();
        options.image.cartridge.set_mapper("fds").unwrap();
        options.image.fds_ram = name != "legacy";
        options.image.fds.disk.license_bypass = !matches!(name, "no_bypass" | "overlay");
        options.image.fds.trim_fill = matches!(name, "overlay_trim" | "metadata_overlay");
        if name.starts_with("metadata") {
            options.image.fds.disk.metadata = Metadata::load(cases.join("files.json")).unwrap();
        }
        if name == "metadata_overlay" {
            options.image.fds.prefix = " x!".into();
        }
        if name == "raw" {
            options.image.fds.disk.header = false;
            options.image.fds.disk.game_code = "ab".into();
        }
        let assembly = codegen::compile_all(&tree, &options).unwrap();
        let image = image::assemble(&assembly, &options.image).unwrap();
        assert_eq!(
            image.rom,
            fs::read(cases.join(format!("{name}.nes"))).unwrap(),
            "{name}: C-to-ROM"
        );
        options.image.fds.disk.native_layout = options.image.fds_ram;
        let result = disk::build(
            &image.prg,
            &image.chr,
            &options.image.fds.disk,
            &image.fds_overlays,
        )
        .unwrap();
        assert_eq!(
            result.image,
            fs::read(cases.join(format!("{name}.fds"))).unwrap(),
            "{name}: C-to-FDS"
        );
    }
}
