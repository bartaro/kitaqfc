//! Content-addressed cache for plain NES builds. Diagnostic and analysis runs
//! execute the pipeline, so a cache hit cannot hide diagnostics or reports.
use crate::{hash, io, json::Value, tokenizer};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
pub struct Entry {
    directory: PathBuf,
}
impl Entry {
    pub fn for_invocation(
        args: &[String],
        sources: &[PathBuf],
        includes: &[PathBuf],
        extra: &[PathBuf],
    ) -> Option<Self> {
        let scan = tokenizer::tokenize_files(sources, includes);
        if !scan.diagnostics.is_empty() {
            return None;
        }
        let exe = std::env::current_exe().ok()?;
        let mut content = format!("kitaqfc-rust-cache-v1\n{}\n", hash::file(exe));
        let mut i = 0;
        while i < args.len() {
            if args[i] == "-o" {
                i += 2;
                continue;
            }
            if !matches!(args[i].as_str(), "--cache" | "--no-cache" | "--no-banner") {
                content.push_str(&args[i]);
                content.push('\n');
            }
            i += 1;
        }
        let files = scan
            .dependencies
            .into_iter()
            .chain(extra.iter().cloned())
            .collect::<BTreeSet<_>>();
        for path in files {
            let bytes = std::fs::read(&path).ok()?;
            content.push_str(&format!("{}|{}\n", path.display(), hash::sha256(&bytes)));
        }
        Some(Self {
            directory: Path::new(".kitaqfc_cache/native-v1").join(hash::sha256(content.as_bytes())),
        })
    }
    pub fn key(&self) -> &str {
        self.directory.file_name().unwrap().to_str().unwrap()
    }
    pub fn restore(&self, output: &Path) -> Result<bool, String> {
        let Ok(text) = io::read_utf8(self.directory.join("manifest.json")) else {
            return Ok(false);
        };
        let Ok(Value::Object(manifest)) = Value::parse(&text) else {
            return Ok(false);
        };
        let Some(Value::String(expected)) = manifest.get("primary_sha256") else {
            return Ok(false);
        };
        let Ok(bytes) = std::fs::read(self.directory.join("primary.nes")) else {
            return Ok(false);
        };
        if hash::sha256(&bytes) != *expected {
            return Ok(false);
        }
        io::write_bytes(output, &bytes)?;
        Ok(true)
    }
    pub fn save(&self, output: &Path) -> Result<(), String> {
        let bytes = std::fs::read(output).map_err(|e| e.to_string())?;
        io::write_bytes(self.directory.join("primary.nes"), &bytes)?;
        let manifest =
            Value::Object([("primary_sha256".into(), Value::String(hash::sha256(&bytes)))].into());
        io::write_utf8(self.directory.join("manifest.json"), &manifest.stringify())
    }
}
