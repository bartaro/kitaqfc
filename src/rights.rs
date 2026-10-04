//! Portable publication text scanner; reports locations without disclosing denylisted terms.
use std::{
    fs,
    path::{Path, PathBuf},
};
mod unicode_casefold;
const SUFFIXES: &[&str] = &[
    "c", "cc", "cpp", "cs", "h", "hpp", "json", "jsonl", "md", "ps1", "py", "rs", "sh", "toml",
    "txt", "xml", "yaml", "yml", "html", "htm", "js", "css", "svg",
];
const NAMES: &[&str] = &["cargo.lock", "license", "notice"];
const EXCLUDED: &[&str] = &[
    ".git",
    ".kitaqfc_cache",
    ".pytest_cache",
    "bin",
    "node_modules",
    "obj",
    "out",
    "target",
];
fn lines(s: &str) -> Vec<&str> {
    s.split([
        '\n', '\r', '\u{b}', '\u{c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{85}', '\u{2028}',
        '\u{2029}',
    ])
    .collect()
}
fn folded(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            unicode_casefold::map(c)
                .map(|s| s.chars().collect::<Vec<_>>())
                .unwrap_or_else(|| vec![c])
        })
        .collect()
}
fn visit(
    root: &Path,
    denylist: &Path,
    terms: &[String],
    hits: &mut Vec<(PathBuf, usize)>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(root)
        .map_err(|e| format!("{}: {e}", root.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            if !EXCLUDED.contains(&folded(&entry.file_name().to_string_lossy()).as_str()) {
                visit(&path, denylist, terms, hits)?;
            }
        } else if kind.is_file() || (kind.is_symlink() && path.is_file()) {
            if path.canonicalize().map_err(|e| e.to_string())? == denylist {
                continue;
            }
            let name = folded(&entry.file_name().to_string_lossy());
            let suffix = path
                .extension()
                .map(|s| folded(&s.to_string_lossy()))
                .unwrap_or_default();
            if !NAMES.contains(&name.as_str()) && !SUFFIXES.contains(&suffix.as_str()) {
                continue;
            }
            let data = fs::read(&path).map_err(|e| e.to_string())?;
            let text = String::from_utf8_lossy(&data).replace("\r\n", "\n");
            for (i, line) in lines(&text).into_iter().enumerate() {
                let line = folded(line);
                if terms.iter().any(|term| line.contains(term)) {
                    hits.push((path.clone(), i + 1));
                }
            }
        }
    }
    Ok(())
}
pub fn scan(root: &Path, denylist: &Path) -> Result<Vec<(PathBuf, usize)>, String> {
    let denylist = denylist.canonicalize().map_err(|e| e.to_string())?;
    let data = fs::read_to_string(&denylist)
        .map_err(|e| e.to_string())?
        .replace("\r\n", "\n");
    let terms = lines(data.trim_start_matches('\u{feff}'))
        .into_iter()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(folded)
        .collect::<Vec<_>>();
    let mut hits = Vec::new();
    visit(root, &denylist, &terms, &mut hits)?;
    Ok(hits)
}
pub fn cli(args: &[String]) -> Result<bool, String> {
    let usage = "Usage: kitaqfc-rights-name-guard [--root DIRECTORY] --denylist FILE";
    if args == ["--help"] {
        println!("{usage}");
        return Ok(true);
    }
    if args == ["--version"] {
        println!(
            "kitaqfc-rights-name-guard {} (Rust)",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(true);
    }
    let mut root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut deny = None;
    let mut at = 0;
    while at < args.len() {
        let arg = &args[at];
        at += 1;
        let value = args.get(at).ok_or(usage)?;
        if arg == "--root" {
            root = PathBuf::from(value);
        } else if arg == "--denylist" {
            deny = Some(PathBuf::from(value));
        } else {
            return Err(format!("Unknown option: {arg}"));
        }
        at += 1;
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let hits = scan(&root, &deny.ok_or(usage)?)?;
    for (path, line) in &hits {
        println!(
            "{}:{line}: protected-term match",
            path.strip_prefix(&root)
                .map_err(|e| e.to_string())?
                .display()
        );
    }
    if hits.is_empty() {
        println!("PASS rights-name-guard matches=0");
    } else {
        println!("FAIL rights-name-guard matches={}", hits.len());
    }
    Ok(hits.is_empty())
}
