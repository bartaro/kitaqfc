use kitaqfc::{assets, json::Value, rights};
use std::{fs, path::PathBuf, process::Command};
fn stage(name: &str) -> PathBuf {
    let dest = std::env::temp_dir().join(format!("kitaqfc-helper-{name}-{}", std::process::id()));
    fs::create_dir_all(&dest).unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/asset-helper-fixtures");
    for entry in fs::read_dir(source).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::copy(&path, dest.join(path.file_name().unwrap())).unwrap();
        }
    }
    dest
}
#[test]
fn raw_asset_pack_matches_python_chr_c_headers_and_report_records() {
    let root = stage("pack");
    let mut report = assets::pack(&root.join("pack.json")).unwrap();
    for suffix in ["chr", "c", "h"] {
        assert_eq!(
            fs::read(root.join(format!("actual/packed.{suffix}"))).unwrap(),
            fs::read(root.join(format!("expected.{suffix}"))).unwrap(),
            "{suffix}"
        );
    }
    if let Value::Object(ref mut fields) = report {
        fields.remove("manifest");
    }
    assert_eq!(
        report,
        Value::parse(&fs::read_to_string(root.join("expected-report.json")).unwrap()).unwrap()
    );
    let bad = root.join("bad.json");
    fs::write(&bad, r#"{"chr":[{"input":"bg.chr","tile_offset":511}]}"#).unwrap();
    assert!(assets::pack(&bad).is_err());
    fs::write(
        &bad,
        r#"{"metasprites":[{"name":"bad","data":[[0,0,256,0]]}]}"#,
    )
    .unwrap();
    assert!(assets::pack(&bad).is_err());
    fs::write(
        &bad,
        r#"{"palettes":[{"name":"colors","input":"attrs.atr"}]}"#,
    )
    .unwrap();
    assert!(assets::pack(&bad).is_err());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn indexed_png_plane_layout_attributes_deduplication_and_interlacing() {
    let root = stage("png");
    assets::png_build(&root.join("png.json")).unwrap();
    for file in ["bg.chr", "bg.nam", "bg.atr", "sprite.chr"] {
        assert_eq!(
            fs::read(root.join(format!("actual/{file}"))).unwrap(),
            fs::read(root.join(format!("expected-{file}"))).unwrap(),
            "{file}"
        );
    }
    let meta =
        Value::parse(&fs::read_to_string(root.join("actual/sprite.metasprite.json")).unwrap())
            .unwrap();
    let Value::Object(meta) = meta else { panic!() };
    let Value::Array(frames) = &meta["frames"] else {
        panic!()
    };
    assert_eq!(frames.len(), 2);
    assert_eq!(
        fs::read(root.join("actual/bg.pal")).unwrap(),
        [vec![vec![13, 1, 9, 48]; 4].concat(), vec![13; 16]].concat()
    );
    for depth in [1, 2, 4, 8] {
        fs::write(root.join("depth.json"),format!(r#"{{"spritesheets":[{{"name":"d","input":"depth-{depth}.png","out_prefix":"actual/depth-{depth}"}}]}}"#)).unwrap();
        assets::png_build(&root.join("depth.json")).unwrap();
        assert_eq!(
            fs::metadata(root.join(format!("actual/depth-{depth}.chr")))
                .unwrap()
                .len(),
            16
        );
    }
    for (input, key) in [
        ("rgb.png", "spritesheets"),
        ("mixed.png", "spritesheets"),
        ("mixed-quadrant.png", "backgrounds"),
    ] {
        fs::write(
            root.join("bad.json"),
            format!(
                r#"{{"{key}":[{{"name":"bad","input":"{input}","out_prefix":"actual/invalid"}}]}}"#
            ),
        )
        .unwrap();
        assert!(
            assets::png_build(&root.join("bad.json")).is_err(),
            "{input}"
        );
        assert!(!root.join("actual/invalid.chr").exists());
    }
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_png_pipeline_outputs_are_consumed_by_native_nes_compiler() {
    let root = stage("pipeline");
    let run = Command::new(env!("CARGO_BIN_EXE_kitaqfc-asset-pipeline"))
        .arg(root.join("pipeline.json"))
        .output()
        .unwrap();
    assert!(run.status.success(), "{:?}", run);
    let blob = fs::read(root.join("actual/pipeline.chr")).unwrap();
    assert_eq!(blob.len(), 8192);
    let background = fs::read(root.join("actual/bg.chr")).unwrap();
    assert_eq!(&blob[..background.len()], background);
    let sprite = fs::read(root.join("actual/sprite.chr")).unwrap();
    assert_eq!(&blob[2048..2048 + sprite.len()], sprite);
    fs::write(root.join("game.c"),"#include \"fc.h\"\n#include \"actual/pipeline.h\"\nu8 observed; void main(){observed=bg_nam[0];while(1){}}\n").unwrap();
    let output = root.join("game.nes");
    let run = Command::new(env!("CARGO_BIN_EXE_kitaqfc"))
        .args([root.join("game.c"), root.join("actual/pipeline.c")])
        .arg("--mapper=nrom")
        .arg(format!(
            "--nes-chr={}",
            root.join("actual/pipeline.chr").display()
        ))
        .arg(format!(
            "-I{}",
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("lib")
                .display()
        ))
        .arg("--no-cache")
        .arg("-o")
        .arg(&output)
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(run.status.success(), "{:?}", run);
    let rom = fs::read(output).unwrap();
    assert_eq!(&rom[rom.len() - 8192..], &blob);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn rights_scanner_preserves_unicode_folding_line_numbers_exclusions_and_privacy() {
    let root = stage("rights");
    let deny = root.join("denylist.txt");
    fs::write(&deny, "\u{feff}# local terms\nSTRASSE\nσ\n").unwrap();
    fs::write(
        root.join("test.html"),
        "clean\r\nStraße\r\nς\u{2028}clean\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("out")).unwrap();
    fs::write(root.join("out/ignored.html"), "STRASSE").unwrap();
    fs::write(root.join("ignored.bin"), "STRASSE").unwrap();
    let hits = rights::scan(&root, &deny).unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].1, 2);
    assert_eq!(hits[1].1, 3);
    let run = Command::new(env!("CARGO_BIN_EXE_kitaqfc-rights-name-guard"))
        .arg("--root")
        .arg(&root)
        .arg("--denylist")
        .arg(&deny)
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(1));
    let stdout = String::from_utf8(run.stdout).unwrap();
    assert!(!stdout.contains("STRASSE"));
    assert!(!stdout.contains("Straße"));
    assert!(stdout.contains("matches=2"));
    fs::write(root.join("test.html"), "clean").unwrap();
    assert!(rights::scan(&root, &deny).unwrap().is_empty());
    fs::remove_dir_all(root).unwrap();
}
