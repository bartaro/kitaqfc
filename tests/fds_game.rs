use kitaqfc::{io, json::Value};
use std::{fs, path::PathBuf, process::Command};
#[test]
fn complete_fds_game_matches_frozen_csharp_disk() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game = root.join("tests/games/kitaq_blocks_fds");
    let Value::Object(manifest) =
        Value::parse(&io::read_utf8(game.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!()
    };
    let string = |v: &Value| -> String {
        let Value::String(s) = v else { panic!() };
        s.clone()
    };
    let Value::Array(sources) = &manifest["sources"] else {
        panic!()
    };
    let Value::Array(flags) = &manifest["flags"] else {
        panic!()
    };
    let mut args = sources
        .iter()
        .map(|s| game.join(string(s)).display().to_string())
        .collect::<Vec<_>>();
    args.extend(
        flags
            .iter()
            .map(|f| string(f).replace("$GAME", &game.display().to_string())),
    );
    let serial = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch =
        std::env::temp_dir().join(format!("kitaqfc-fds-game-{}-{serial}", std::process::id()));
    fs::create_dir(&scratch).unwrap();
    let output = scratch.join("out.fds");
    args.extend(["-o".into(), output.display().to_string()]);
    let result = Command::new(env!("CARGO_BIN_EXE_kitaqfc"))
        .current_dir(&scratch)
        .args(args)
        .output()
        .unwrap();
    let bytes = fs::read(&output);
    let _ = fs::remove_dir_all(scratch);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        bytes.unwrap(),
        fs::read(root.join("tests/games/kitaq_blocks_fds.fds")).unwrap()
    );
}
