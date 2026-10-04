use kitaqfc::{cli, json::Value};
use std::{fs, path::PathBuf};

fn normalize(value: &mut Value, output: &str, root: &str) {
    match value {
        Value::String(s) => *s = s.replace(output, "$OUTPUT").replace(root, "$PORT"),
        Value::Array(values) => values.iter_mut().for_each(|v| normalize(v, output, root)),
        Value::Object(fields) => fields.values_mut().for_each(|v| normalize(v, output, root)),
        _ => (),
    }
}
#[test]
fn kurosaki_metadata_matches_reference_allocations_calls_actions_and_redaction() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/metadata");
    let work = root.join(format!("target/metadata-smoke-{}", std::process::id()));
    fs::create_dir(&work).unwrap();
    let root_text = fs::canonicalize(&root)
        .unwrap()
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned();
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("metadata manifest");
    };
    assert_eq!(cases.len(), 112);
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("metadata case");
        };
        let string = |key: &str| {
            let Value::String(s) = &fields[key] else {
                panic!("string {key}");
            };
            s.clone()
        };
        let name = string("case");
        let mut args = Vec::new();
        let Value::Array(sources) = &fields["sources"] else {
            panic!("sources");
        };
        for source in sources {
            let Value::String(s) = source else {
                panic!("source");
            };
            args.push(root.join(s).to_string_lossy().into_owned());
        }
        let output = work.join(format!("{name}.nes"));
        let metadata = work.join(format!("{name}.json"));
        args.extend([
            "-I".into(),
            root.join("lib").to_string_lossy().into_owned(),
            format!("--mapper={}", string("mapper")),
            "--no-cache".into(),
            "--no-disasm".into(),
            format!("--kurosaki-metadata={}", metadata.display()),
        ]);
        let Value::Array(flags) = &fields["flags"] else {
            panic!("flags");
        };
        for flag in flags {
            let Value::String(s) = flag else {
                panic!("flag");
            };
            args.push(s.replace("$PORT", &root.to_string_lossy()));
        }
        args.extend(["-o".into(), output.to_string_lossy().into_owned()]);
        assert_eq!(cli::run(args), 0, "{name}: compile and metadata");
        let mut actual = Value::parse(&fs::read_to_string(&metadata).unwrap()).unwrap();
        normalize(&mut actual, &output.to_string_lossy(), &root_text);
        let expected =
            Value::parse(&fs::read_to_string(fixtures.join(format!("{name}.json"))).unwrap())
                .unwrap();
        assert_eq!(actual, expected, "{name}: parsed KUROSAKI metadata");
    }
    fs::remove_dir_all(work).unwrap();
}
