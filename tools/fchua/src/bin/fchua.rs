use kitaqfc_graphics_editor_lib::{commands, image_io, KitaqfcGraphicsDocument as Document};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

const HELP: &str = r#"FCHUA - NES graphics tools for KITAQFC
Usage: fchua <command> [arguments] [options]

  new --out project.fch [--demo] [--force]
  inspect project.fch
  validate project.fch [--strict]
  import-image artwork.png --out project.fch [--project base.fch] [--auto-palette] [--force]
  import-raw --out project.fch [--project base.fch] [--chr tiles.chr]
             [--palette colors.pal] [--nametable map.nam] [--attributes map.atr] [--force]
  apply project.fch --script edits.json --out edited.fch [--dry-run] [--force]
  preview project.fch --out image.png [--view map|tiles|sprite|tile] [--scale 1..8] [--force]
  export project.fch --out directory [--prefix scene] [--force]
  migrate legacy.kfcproject --out project.fch [--force]

All successful commands emit one JSON object. Errors emit JSON to stderr.
Exit codes: 0 success, 1 strict validation warnings, 2 invalid arguments/data/IO.
Existing output requires --force; input files cannot be overwritten.
Scripts are JSON arrays of operations; see examples/edits.json and the manual.
No GUI, GPU, Python, network access, or plita runtime is required.
"#;

struct Args {
    command: String,
    positionals: Vec<String>,
    options: BTreeMap<String, String>,
}
impl Args {
    fn parse(mut raw: Vec<String>) -> Result<Self, String> {
        if raw.is_empty() {
            return Err("missing command; use --help".into());
        }
        let command = raw.remove(0);
        let mut positionals = Vec::new();
        let mut options = BTreeMap::new();
        let mut iter = raw.into_iter();
        while let Some(arg) = iter.next() {
            if let Some(key) = arg.strip_prefix("--") {
                let value = if ["force", "demo", "strict", "auto-palette", "dry-run"].contains(&key)
                {
                    "true".into()
                } else {
                    iter.next()
                        .filter(|v| !v.starts_with("--"))
                        .ok_or_else(|| format!("missing value for --{key}"))?
                };
                if options.insert(key.into(), value).is_some() {
                    return Err(format!("duplicate --{key}"));
                }
            } else {
                positionals.push(arg);
            }
        }
        Ok(Self {
            command,
            positionals,
            options,
        })
    }
    fn check(&self, n: usize, allowed: &[&str]) -> Result<(), String> {
        if self.positionals.len() != n {
            return Err(format!("{} expects {n} positional arguments", self.command));
        }
        for key in self.options.keys() {
            if !allowed.contains(&key.as_str()) {
                return Err(format!("unknown option --{key}"));
            }
        }
        Ok(())
    }
    fn required(&self, key: &str) -> Result<&str, String> {
        self.options
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("--{key} is required"))
    }
    fn value<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.options.get(key).map(String::as_str).unwrap_or(default)
    }
    fn flag(&self, key: &str) -> bool {
        self.options.contains_key(key)
    }
    fn input(&self) -> Result<Document, String> {
        Document::load_project(&self.positionals[0]).map_err(|e| e.to_string())
    }
    fn base(&self) -> Result<Document, String> {
        match self.options.get("project") {
            Some(p) => Document::load_project(p).map_err(|e| e.to_string()),
            None => Ok(Document::new_blank()),
        }
    }
    fn output(&self, extension: Option<&str>) -> Result<PathBuf, String> {
        let path = PathBuf::from(self.required("out")?);
        if let Some(ext) = extension {
            if !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(ext))
            {
                return Err(format!("output must end with .{ext}"));
            }
        }
        let full = absolute(&path)?;
        for src in self.positionals.iter().chain(
            self.options
                .iter()
                .filter(|(k, _)| {
                    [
                        "project",
                        "script",
                        "chr",
                        "palette",
                        "nametable",
                        "attributes",
                    ]
                    .contains(&k.as_str())
                })
                .map(|(_, v)| v),
        ) {
            if full == absolute(Path::new(src))? {
                return Err("input and output paths must differ".into());
            }
        }
        if path.exists() && !self.flag("force") {
            return Err(format!("{} already exists; use --force", path.display()));
        }
        Ok(path)
    }
}
fn absolute(p: &Path) -> Result<PathBuf, String> {
    if p.exists() {
        return p.canonicalize().map_err(|e| e.to_string());
    }
    let parent = p
        .parent()
        .filter(|x| !x.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(parent
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(p.file_name().ok_or("missing filename")?))
}
fn save(d: &Document, path: &Path) -> Result<(), String> {
    d.save_project(path).map_err(|e| e.to_string())
}
fn run(a: Args) -> Result<(Value, i32), String> {
    match a.command.as_str() {
        "new" => {
            a.check(0, &["out", "demo", "force"])?;
            let out = a.output(Some("fch"))?;
            let d = if a.flag("demo") {
                Document::new_demo()
            } else {
                Document::new_blank()
            };
            save(&d, &out)?;
            Ok((
                json!({"ok":true,"path":out,"report":d.analysis_report()}),
                0,
            ))
        }
        "inspect" | "validate" => {
            a.check(
                1,
                if a.command == "validate" {
                    &["strict"]
                } else {
                    &[]
                },
            )?;
            let d = a.input()?;
            let report = d.analysis_report();
            let code = if a.flag("strict") && !report.warnings.is_empty() {
                1
            } else {
                0
            };
            Ok((
                json!({"ok":code==0,"format":"fchua","version":1,"report":report}),
                code,
            ))
        }
        "migrate" => {
            a.check(1, &["out", "force"])?;
            let d = a.input()?;
            let out = a.output(Some("fch"))?;
            save(&d, &out)?;
            Ok((json!({"ok":true,"path":out}), 0))
        }
        "import-image" => {
            a.check(1, &["out", "force", "project", "auto-palette"])?;
            let out = a.output(Some("fch"))?;
            let (d, report) = image_io::import_png(
                &a.base()?,
                Path::new(&a.positionals[0]),
                a.flag("auto-palette"),
            )?;
            save(&d, &out)?;
            Ok((
                json!({"ok":true,"path":out,"import":report,"warnings":d.analysis_report().warnings}),
                0,
            ))
        }
        "import-raw" => {
            a.check(
                0,
                &[
                    "out",
                    "force",
                    "project",
                    "chr",
                    "palette",
                    "nametable",
                    "attributes",
                ],
            )?;
            if !["chr", "palette", "nametable", "attributes"]
                .iter()
                .any(|k| a.flag(k))
            {
                return Err("at least one raw input is required".into());
            }
            let out = a.output(Some("fch"))?;
            let mut d = a.base()?;
            for (key, size) in [
                ("chr", 8192),
                ("palette", 32),
                ("nametable", 960),
                ("attributes", 64),
            ] {
                if let Some(p) = a.options.get(key) {
                    let bytes = fs::read(p).map_err(|e| e.to_string())?;
                    if key != "chr" && bytes.len() != size {
                        return Err(format!("{key} must be exactly {size} bytes"));
                    }
                    match key {
                        "chr" => d.try_load_chr_bytes(&bytes).map_err(|e| e.to_string())?,
                        "palette" => {
                            if bytes.iter().any(|v| *v > 63) {
                                return Err("palette values must be 0..63".into());
                            }
                            d.load_palette_bytes(&bytes);
                        }
                        "nametable" => d.load_nametable_bytes(&bytes),
                        _ => d.load_attribute_bytes(&bytes),
                    }
                }
            }
            save(&d, &out)?;
            Ok((json!({"ok":true,"path":out}), 0))
        }
        "apply" => {
            a.check(1, &["out", "force", "script", "dry-run"])?;
            let out = a.output(Some("fch"))?;
            let mut d = a.input()?;
            let commands: Vec<commands::Command> = serde_json::from_slice(
                &fs::read(a.required("script")?).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            commands::apply(&mut d, &commands)?;
            if !a.flag("dry-run") {
                save(&d, &out)?;
            }
            Ok((
                json!({"ok":true,"path":out,"dry_run":a.flag("dry-run"),"operations":commands.len(),"report":d.analysis_report()}),
                0,
            ))
        }
        "preview" => {
            a.check(1, &["out", "force", "view", "scale"])?;
            let out = a.output(Some("png"))?;
            let scale = a.value("scale", "1").parse().map_err(|_| "invalid scale")?;
            image_io::preview_png(&a.input()?, &out, a.value("view", "map"), scale)?;
            Ok((json!({"ok":true,"path":out}), 0))
        }
        "export" => {
            a.check(1, &["out", "force", "prefix"])?;
            let out = a.output(None)?;
            let d = a.input()?;
            let prefix = a.value("prefix", "scene");
            if prefix.is_empty()
                || prefix.as_bytes()[0].is_ascii_digit()
                || !prefix
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                return Err("prefix must be a C identifier".into());
            }
            // Never overwrite the input indirectly through a generated filename.
            if absolute(&Path::new(&a.positionals[0]))?
                == absolute(&out).map(|p| p.join(format!("{prefix}.fch")))?
            {
                return Err("export would overwrite input project".into());
            }
            let report = d
                .export_raw_assets(&out, prefix)
                .map_err(|e| e.to_string())?;
            Ok((
                json!({"ok":true,"directory":out,"manifest":report.manifest_path,"warnings":d.analysis_report().warnings}),
                0,
            ))
        }
        _ => Err(format!("unknown command {}; use --help", a.command)),
    }
}
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args == ["--help"] || args == ["-h"] {
        print!("{HELP}");
        return;
    }
    if args == ["--version"] {
        println!("FCHUA {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    match Args::parse(args).and_then(run) {
        Ok((value, code)) => {
            println!("{value}");
            std::process::exit(code);
        }
        Err(error) => {
            eprintln!("{}", json!({"ok":false,"error":error}));
            std::process::exit(2);
        }
    }
}
