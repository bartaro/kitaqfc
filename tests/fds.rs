use kitaqfc::{
    assembler::{fds, image},
    assembly_json,
    fds::{Metadata, disk},
};
use std::{fs, path::PathBuf};
#[test]
fn native_disk_and_image_bytes_match_reference_for_eight_layouts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fds");
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
        let nodes = assembly_json::parse(
            &kitaqfc::io::read_utf8(root.join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        let mut options = image::Options::default();
        options.cartridge.set_mapper("fds").unwrap();
        options.fds_ram = name != "legacy";
        options.fds.disk.license_bypass = !matches!(name, "no_bypass" | "overlay");
        options.fds.trim_fill = matches!(name, "overlay_trim" | "metadata_overlay");
        if name.starts_with("metadata") {
            options.fds.disk.metadata = Metadata::load(root.join("files.json")).unwrap();
        }
        if name == "metadata_overlay" {
            options.fds.prefix = " x!".into();
        }
        if name == "raw" {
            options.fds.disk.header = false;
            options.fds.disk.game_code = "ab".into();
        }
        let image = image::assemble(&nodes, &options).unwrap();
        assert_eq!(
            image.rom,
            fs::read(root.join(format!("{name}.nes"))).unwrap(),
            "{name}: PRG/CHR bytes"
        );
        options.fds.disk.native_layout = options.fds_ram;
        let result = disk::build(
            &image.prg,
            &image.chr,
            &options.fds.disk,
            &image.fds_overlays,
        )
        .unwrap();
        assert_eq!(
            result.image,
            fs::read(root.join(format!("{name}.fds"))).unwrap(),
            "{name}: disk bytes"
        );
    }
}
#[test]
fn metadata_aliases_delimited_input_and_representable_fields() {
    let json = "{'files':[{'fileId':7,'length':'0x0010','loadAddress':'0x6000','fileType':0,'isOverlay':'yes','loadOnBoot':0,'label':'ASSET','src':'asset.bin','disk_side':1,'fileNumber':4}]}";
    let delimited = "id size load type overlay name source side number boot\n7 16 $6000 0 overlay ASSET asset.bin 1 4 no\n";
    let a = Metadata::parse(json, PathBuf::new()).unwrap();
    let b = Metadata::parse(delimited, PathBuf::new()).unwrap();
    assert_eq!(a.files, b.files);
    assert_eq!(a.runtime_table(&[]), [7, 16, 0, 0, 0x60, 0x80, 255]);
    assert!(Metadata::parse("[{\"id\":255}]", PathBuf::new()).is_err());
    assert!(
        Metadata::parse("[{\"id\":1},{\"id\":1}]", PathBuf::new())
            .unwrap_err()
            .contains("duplicate")
    );
}
#[test]
fn fds_constraints_reject_capacity_and_overlay_conflicts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fds");
    let nodes =
        assembly_json::parse(&kitaqfc::io::read_utf8(root.join("overlay.json")).unwrap()).unwrap();
    let mut options = image::Options::default();
    options.cartridge.set_mapper("fds").unwrap();
    let image = image::assemble(&nodes, &options).unwrap();
    assert!(
        disk::build(
            &image.prg,
            &image.chr,
            &options.fds.disk,
            &image.fds_overlays
        )
        .unwrap_err()
        .contains("KQFC2503")
    );
    options.fds.start_id = 0;
    assert!(
        fds::overlay_metadata(2, &options.fds)
            .unwrap_err()
            .contains("KQFC2508")
    );
    options.fds.start_id = 254;
    assert!(
        fds::overlay_metadata(3, &options.fds)
            .unwrap_err()
            .contains("KQFC2507")
    );
    options.fds.auto_overlay = false;
    assert!(
        image::assemble(&nodes, &options)
            .unwrap_err()
            .contains("KQFC2505")
    );
}
