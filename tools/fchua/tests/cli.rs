use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn cli(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fchua"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}
fn ok(dir: &Path, args: &[&str]) -> Value {
    let out = cli(dir, args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
fn bad(dir: &Path, args: &[&str]) {
    let out = cli(dir, args);
    assert_eq!(out.status.code(), Some(2), "{args:?}");
    let v: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(v["ok"], false);
}

#[test]
fn create_edit_preview_export_and_reimport() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    ok(p, &["new", "--out", "base.fch"]);
    fs::write(
        p.join("edits.json"),
        serde_json::to_vec(&json!([
            {"op":"checker","tile":1,"a":1,"b":3},
            {"op":"palette","index":1,"value":48},
            {"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":1},
            {"op":"sprite","frame":1,"index":0,"x":-8,"y":0,"tile":1,"attr":64}
        ]))
        .unwrap(),
    )
    .unwrap();
    ok(
        p,
        &[
            "apply",
            "base.fch",
            "--script",
            "edits.json",
            "--out",
            "scene.fch",
        ],
    );
    ok(p, &["validate", "scene.fch", "--strict"]);
    ok(p, &["inspect", "scene.fch"]);
    for view in ["map", "tiles", "sprite", "tile"] {
        ok(
            p,
            &[
                "preview",
                "scene.fch",
                "--out",
                &format!("{view}.png"),
                "--view",
                view,
                "--scale",
                "2",
            ],
        );
    }
    assert_eq!(image::open(p.join("map.png")).unwrap().width(), 512);
    ok(
        p,
        &["export", "scene.fch", "--out", "pack", "--prefix", "demo"],
    );
    for (name, len) in [
        ("demo.chr", 8192),
        ("demo.pal", 32),
        ("demo.nam", 960),
        ("demo.atr", 64),
    ] {
        assert_eq!(fs::metadata(p.join("pack").join(name)).unwrap().len(), len);
    }
    assert!(fs::read_to_string(p.join("pack/demo_assets.c"))
        .unwrap()
        .contains("demo_metasprite_1"));
    ok(
        p,
        &[
            "import-raw",
            "--out",
            "back.fch",
            "--chr",
            "pack/demo.chr",
            "--palette",
            "pack/demo.pal",
            "--nametable",
            "pack/demo.nam",
            "--attributes",
            "pack/demo.atr",
        ],
    );
    let before: Value = serde_json::from_slice(&fs::read(p.join("scene.fch")).unwrap()).unwrap();
    let after: Value = serde_json::from_slice(&fs::read(p.join("back.fch")).unwrap()).unwrap();
    for key in ["chr", "palette", "nametable", "attributes"] {
        assert_eq!(before["document"][key], after["document"][key]);
    }
}
#[test]
fn overwrite_guards_and_invalid_arguments() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    ok(p, &["new", "--out", "a.fch"]);
    let before = fs::read(p.join("a.fch")).unwrap();
    for args in [
        vec!["new", "--out", "a.fch"],
        vec!["new", "--out", "a.json"],
        vec!["new", "--out", "x.fch", "--oops", "yes"],
        vec!["inspect", "a.fch", "--strict"],
        vec!["migrate", "a.fch", "--out", "a.fch", "--force"],
        vec!["preview", "a.fch", "--out", "bad.png", "--scale", "0"],
        vec!["export", "a.fch", "--out", ".", "--prefix", "a", "--force"],
    ] {
        bad(p, &args);
    }
    assert_eq!(fs::read(p.join("a.fch")).unwrap(), before);
    ok(p, &["new", "--out", "a.fch", "--force"]);
}
#[test]
fn batch_validation_and_dry_run_do_not_write() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    ok(p, &["new", "--out", "a.fch"]);
    fs::write(
        p.join("bad.json"),
        r#"[{"op":"fill","tile":0,"color":2},{"op":"fill","tile":512,"color":3}]"#,
    )
    .unwrap();
    bad(
        p,
        &["apply", "a.fch", "--script", "bad.json", "--out", "bad.fch"],
    );
    assert!(!p.join("bad.fch").exists());
    fs::write(p.join("good.json"), r#"[{"op":"fill","tile":0,"color":2}]"#).unwrap();
    let report = ok(
        p,
        &[
            "apply",
            "a.fch",
            "--script",
            "good.json",
            "--out",
            "dry.fch",
            "--dry-run",
        ],
    );
    assert_eq!(report["dry_run"], true);
    assert!(!p.join("dry.fch").exists());
}
#[test]
fn invalid_files_and_strict_scanline_warning() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    ok(p, &["new", "--out", "a.fch"]);
    fs::write(p.join("bad.pal"), [64u8; 32]).unwrap();
    bad(p, &["import-raw", "--palette", "bad.pal", "--out", "b.fch"]);
    fs::write(p.join("bad.chr"), [0u8; 17]).unwrap();
    bad(p, &["import-raw", "--chr", "bad.chr", "--out", "b.fch"]);
    let commands: Vec<Value> = (0..9)
        .map(|i| json!({"op":"sprite","frame":0,"index":i,"x":i*8,"y":0,"tile":0,"attr":0}))
        .collect();
    fs::write(
        p.join("sprites.json"),
        serde_json::to_vec(&commands).unwrap(),
    )
    .unwrap();
    ok(
        p,
        &[
            "apply",
            "a.fch",
            "--script",
            "sprites.json",
            "--out",
            "b.fch",
        ],
    );
    assert_eq!(
        cli(p, &["validate", "b.fch", "--strict"]).status.code(),
        Some(1)
    );
}
#[test]
fn png_import_and_legacy_migration() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    image::RgbaImage::from_pixel(16, 16, image::Rgba([255, 255, 255, 255]))
        .save(p.join("image.png"))
        .unwrap();
    ok(
        p,
        &[
            "import-image",
            "image.png",
            "--out",
            "image.fch",
            "--auto-palette",
        ],
    );
    let mut legacy: Value =
        serde_json::from_slice(&fs::read(p.join("image.fch")).unwrap()).unwrap();
    legacy["format"] = json!("kitaqfc-graphics-project");
    fs::write(
        p.join("old.kfcproject"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    ok(p, &["migrate", "old.kfcproject", "--out", "migrated.fch"]);
    let v: Value = serde_json::from_slice(&fs::read(p.join("migrated.fch")).unwrap()).unwrap();
    assert_eq!(v["format"], "fchua");
}
