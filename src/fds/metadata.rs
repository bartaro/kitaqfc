use regex::Regex;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    pub id: i32,
    pub size: i32,
    pub load: i32,
    pub file_type: i32,
    pub overlay: bool,
    pub boot: bool,
    pub name: String,
    pub source: String,
    pub side: i32,
    pub number: i32,
}
impl Default for File {
    fn default() -> Self {
        Self {
            id: 0,
            size: 0,
            load: 0,
            file_type: 0,
            overlay: false,
            boot: true,
            name: String::new(),
            source: String::new(),
            side: 0,
            number: -1,
        }
    }
}
impl File {
    pub fn runtime_record(&self) -> [u8; 6] {
        [
            self.id as u8,
            self.size as u8,
            (self.size >> 8) as u8,
            self.load as u8,
            (self.load >> 8) as u8,
            (self.file_type & 0x7f) as u8 | if self.overlay { 0x80 } else { 0 },
        ]
    }
}
#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub source_path: PathBuf,
    pub files: Vec<File>,
}
fn boolean(text: &str) -> bool {
    matches!(
        text.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "overlay"
    )
}
fn number(text: &str) -> Result<i32, String> {
    let t = text.trim();
    if let Some(hex) = t.strip_prefix('$').or_else(|| {
        t.get(..2)
            .filter(|s| s.eq_ignore_ascii_case("0x"))
            .map(|_| &t[2..])
    }) {
        u32::from_str_radix(hex, 16)
            .map(|n| n as i32)
            .map_err(|_| format!("invalid number: {t}"))
    } else {
        t.parse::<i32>().map_err(|_| format!("invalid number: {t}"))
    }
}
// The original metadata reader deliberately accepts single quotes and aliases,
// but does not unescape JSON strings. Preserve that input dialect.
fn property(obj: &str, aliases: &str, pattern: &str, quoted: bool) -> Option<String> {
    for key in aliases.split('|') {
        let re = Regex::new(&format!(
            r#"(?i)["']{}["']\s*:\s*(["']?)({})"#,
            regex::escape(key),
            pattern
        ))
        .unwrap();
        if let Some(c) = re.captures(obj) {
            let quote = c.get(1).unwrap().as_str();
            let value = c.get(2).unwrap();
            if quoted && quote.is_empty() {
                continue;
            }
            if !quote.is_empty() && !obj[value.end()..].starts_with(quote) {
                continue;
            }
            return Some(value.as_str().into());
        }
    }
    None
}
fn integer(obj: &str, aliases: &str) -> Result<Option<i32>, String> {
    property(obj, aliases, r"[-+]?0x[0-9a-f]+|[-+]?[0-9]+", false)
        .map(|v| number(&v))
        .transpose()
}
fn flag(obj: &str, aliases: &str) -> Option<bool> {
    property(obj, aliases, "true|false|yes|no|1|0", false).map(|v| boolean(&v))
}
fn string(obj: &str, aliases: &str) -> String {
    property(obj, aliases, r#"[^"']*"#, true).unwrap_or_default()
}
impl Metadata {
    pub fn new(source_path: PathBuf, mut files: Vec<File>) -> Self {
        files.sort_by_key(|f| f.id);
        Self { source_path, files }
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Ok(Self::default());
        }
        let full = std::fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&crate::io::read_utf8(&full)?, full)
    }
    pub fn parse(text: &str, source_path: PathBuf) -> Result<Self, String> {
        let mut files = Vec::new();
        if text.trim_start().starts_with(['{', '[']) {
            for obj in Regex::new(r"(?s)\{[^{}]*\}").unwrap().find_iter(text) {
                let s = obj.as_str();
                let Some(id) = integer(s, "id|file_id|fileId|number|file")? else {
                    continue;
                };
                files.push(File {
                    id,
                    size: integer(s, "size|length|len|bytes")?.unwrap_or(0),
                    load: integer(s, "load|load_addr|loadAddress|addr|address|dst")?.unwrap_or(0),
                    file_type: integer(s, "type|file_type|fileType")?.unwrap_or(0),
                    overlay: flag(s, "overlay|is_overlay|isOverlay").unwrap_or(false),
                    boot: flag(s, "boot|boot_file|bootFile|load_on_boot|loadOnBoot")
                        .unwrap_or(true),
                    name: string(s, "name|filename|file_name|label"),
                    source: string(s, "source|src|path|data|binary|asset"),
                    side: integer(s, "side|disk_side")?.unwrap_or(0),
                    number: integer(s, "number|file_number|fileNumber|index")?.unwrap_or(-1),
                });
            }
        } else {
            for (line_no, line) in text.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() || line.starts_with(['#', ';']) {
                    continue;
                }
                let p = line
                    .split([',', '\t', ' '])
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                if p.len() < 3 || p[0].eq_ignore_ascii_case("id") {
                    continue;
                }
                let n = |i: usize, field: &str, default: i32| -> Result<i32, String> {
                    match p.get(i) {
                        Some(t) => number(t).map_err(|_| {
                            format!("invalid FDS metadata {field} at line {}: {t}", line_no + 1)
                        }),
                        None => Ok(default),
                    }
                };
                files.push(File {
                    id: n(0, "id", 0)?,
                    size: n(1, "size", 0)?,
                    load: n(2, "load", 0)?,
                    file_type: n(3, "type", 0)?,
                    overlay: p.get(4).is_some_and(|v| boolean(v)),
                    name: p.get(5).unwrap_or(&"").to_string(),
                    source: p.get(6).unwrap_or(&"").to_string(),
                    side: n(7, "side", 0)?,
                    number: n(8, "number", -1)?,
                    boot: p.get(9).is_none_or(|v| boolean(v)),
                });
            }
        }
        let result = Self::new(source_path, files);
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.files.is_empty() {
            return Err(format!(
                "FDS metadata has no file records: {}",
                self.source_path.display()
            ));
        }
        if self.files.len() > 42 {
            return Err(format!(
                "FDS metadata supports up to 42 files in the current 6-byte runtime table: {}",
                self.files.len()
            ));
        }
        let mut ids = BTreeSet::new();
        for f in &self.files {
            if !(0..=254).contains(&f.id) {
                return Err(format!("FDS file id must be 0..254: {}", f.id));
            }
            if !ids.insert(f.id) {
                return Err(format!("duplicate FDS file id: {}", f.id));
            }
            for (value, range, label) in [
                (f.size, 0..=65535, "size"),
                (f.load, 0..=65535, "load address"),
                (f.file_type, 0..=127, "type"),
                (f.side, 0..=254, "side"),
                (f.number, -1..=254, "number"),
            ] {
                if !range.contains(&value) {
                    return Err(format!(
                        "FDS file {label} out of range for id {}: {value}",
                        f.id
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn runtime_table(&self, additional: &[File]) -> Vec<u8> {
        let mut files = self.files.iter().chain(additional).collect::<Vec<_>>();
        files.sort_by_key(|f| f.id);
        let mut bytes = Vec::with_capacity(files.len() * 6 + 1);
        for f in files {
            bytes.extend(f.runtime_record())
        }
        bytes.push(255);
        bytes
    }
    pub fn boot_bank_file_id(&self) -> u8 {
        if self.files.is_empty() {
            0
        } else {
            self.files
                .iter()
                .find(|f| f.boot && f.file_type == 0 && f.load == 0x6000 && f.size == 0x4000)
                .map_or(255, |f| f.id as u8)
        }
    }
    pub fn normalized_json(&self) -> String {
        let q = crate::json::quote;
        let mut s = String::from("{\n  \"format\": \"kitaqfc-fds-metadata-v1\",\n  \"files\": [\n");
        for (i, f) in self.files.iter().enumerate() {
            s.push_str(&format!("    {{ \"id\": {}, \"size\": {}, \"load\": \"0x{:04X}\", \"type\": {}, \"overlay\": {}",f.id,f.size,f.load,f.file_type,f.overlay));
            if !f.name.is_empty() {
                s.push_str(&format!(", \"name\": {}", q(&f.name)))
            }
            if !f.source.is_empty() {
                s.push_str(&format!(", \"source\": {}", q(&f.source)))
            }
            if !f.boot {
                s.push_str(", \"boot\": false")
            }
            if f.side != 0 {
                s.push_str(&format!(", \"side\": {}", f.side))
            }
            if f.number >= 0 {
                s.push_str(&format!(", \"number\": {}", f.number))
            }
            s.push_str(" }");
            if i + 1 < self.files.len() {
                s.push(',')
            }
            s.push('\n');
        }
        s.push_str("  ]\n}\n");
        s
    }
}
