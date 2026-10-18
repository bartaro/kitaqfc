use std::{
    borrow::Cow,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Language {
    English,
    Japanese,
}

impl Language {
    fn parse(value: &str) -> Option<Self> {
        match value
            .trim()
            .split(['-', '_', '.', '@'])
            .next()?
            .to_ascii_lowercase()
            .as_str()
        {
            "ja" => Some(Self::Japanese),
            "en" => Some(Self::English),
            _ => None,
        }
    }

    pub(super) fn detect(path: Option<&Path>) -> Self {
        let saved = path.and_then(|p| fs::read_to_string(p).ok());
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .into_iter()
            .filter_map(|key| env::var(key).ok())
            .find(|s| !s.trim().is_empty());
        Self::resolve(
            env::var("FCHUA_LANGUAGE").ok().as_deref(),
            saved.as_deref(),
            locale.as_deref(),
            system_language(),
        )
    }

    fn resolve(
        override_value: Option<&str>,
        saved: Option<&str>,
        locale: Option<&str>,
        system: Self,
    ) -> Self {
        override_value
            .and_then(Self::parse)
            .or_else(|| saved.and_then(Self::parse))
            .or_else(|| {
                locale
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| Self::parse(s).unwrap_or(Self::English))
            })
            .unwrap_or(system)
    }

    pub(super) fn save(self, path: &Path) -> std::io::Result<()> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        temp.write_all(match self {
            Self::English => b"en\n",
            Self::Japanese => b"ja\n",
        })?;
        temp.as_file().sync_all()?;
        temp.persist(path).map_err(|e| e.error)?;
        Ok(())
    }

    pub(super) fn toggle(self) -> Self {
        match self {
            Self::English => Self::Japanese,
            Self::Japanese => Self::English,
        }
    }

    pub(super) fn translate<'a>(self, text: &'a str) -> Cow<'a, str> {
        if self == Self::English {
            return Cow::Borrowed(text);
        }
        if let Some((_, translation)) = catalog().iter().find(|(key, _)| *key == text) {
            return Cow::Borrowed(translation);
        }
        for (key, translation) in catalog() {
            if key.contains("{}") {
                if let Some(value) = translate_template(text, key, translation) {
                    return Cow::Owned(value);
                }
            }
        }
        Cow::Borrowed(text)
    }
}

pub(super) fn preference_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("FCHUA_CONFIG_DIR").filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(path).join("language.txt"));
    }
    #[cfg(windows)]
    return env::var_os("APPDATA")
        .filter(|s| !s.is_empty())
        .map(|p| PathBuf::from(p).join("FCHUA/language.txt"));
    #[cfg(target_os = "macos")]
    return env::var_os("HOME")
        .filter(|s| !s.is_empty())
        .map(|p| PathBuf::from(p).join("Library/Application Support/FCHUA/language.txt"));
    #[cfg(not(any(windows, target_os = "macos")))]
    env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .filter(|s| !s.is_empty())
                .map(|p| PathBuf::from(p).join(".config"))
        })
        .map(|p| p.join("fchua/language.txt"))
}

fn system_language() -> Language {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        if unsafe { GetUserDefaultUILanguage() } & 0x3ff == 0x11 {
            return Language::Japanese;
        }
    }
    #[cfg(target_os = "macos")]
    if let Ok(output) = std::process::Command::new("/usr/bin/defaults")
        .args(["read", "-g", "AppleLanguages"])
        .output()
    {
        if output.status.success()
            && String::from_utf8_lossy(&output.stdout)
                .split(|c: char| !c.is_ascii_alphabetic())
                .find(|s| !s.is_empty())
                == Some("ja")
        {
            return Language::Japanese;
        }
    }
    Language::English
}

fn catalog() -> &'static Vec<(&'static str, &'static str)> {
    static CATALOG: OnceLock<Vec<(&str, &str)>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        include_str!("locale_ja.tsv")
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .collect()
    })
}

// Only match complete messages; captured paths and numbers remain verbatim.
fn translate_template(text: &str, pattern: &str, translation: &str) -> Option<String> {
    let parts: Vec<_> = pattern.split("{}").collect();
    let mut tail = text.strip_prefix(parts[0])?;
    let mut values = Vec::new();
    for (i, part) in parts.iter().enumerate().skip(1) {
        if i == parts.len() - 1 {
            values.push(tail.strip_suffix(part)?);
        } else {
            let at = tail.find(part)?;
            values.push(&tail[..at]);
            tail = &tail[at + part.len()..];
        }
    }
    let mut translated = translation.split("{}");
    let mut out = translated.next()?.to_owned();
    for value in values {
        out.push_str(value);
        out.push_str(translated.next()?);
    }
    translated.next().is_none().then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_precedence_and_invalid_preferences() {
        use Language::{English as En, Japanese as Ja};
        assert_eq!(
            Language::resolve(Some("EN-us"), Some("ja"), Some("ja_JP.UTF-8"), Ja),
            En
        );
        assert_eq!(Language::resolve(None, Some("ja\n"), Some("en_US"), En), Ja);
        assert_eq!(
            Language::resolve(Some("invalid"), Some("broken"), Some("ja_JP.UTF-8"), En),
            Ja
        );
        assert_eq!(Language::resolve(None, None, Some("fr_FR"), Ja), En);
        assert_eq!(Language::resolve(None, None, None, Ja), Ja);
        assert_eq!(Language::parse("jargon"), None);
    }

    #[test]
    fn preference_round_trip_and_write_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings/language.txt");
        Language::Japanese.save(&path).unwrap();
        assert_eq!(
            Language::parse(&fs::read_to_string(&path).unwrap()),
            Some(Language::Japanese)
        );
        Language::English.save(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "en\n");
        assert!(Language::Japanese.save(&path.join("child")).is_err());
    }

    #[test]
    fn catalog_and_templates_preserve_user_data() {
        let mut keys = std::collections::HashSet::new();
        for (key, translation) in catalog() {
            assert!(keys.insert(key), "duplicate key: {key}");
            assert!(!translation.is_empty(), "empty translation: {key}");
            assert_eq!(
                key.matches("{}").count(),
                translation.matches("{}").count(),
                "{key}"
            );
        }
        assert_eq!(Language::Japanese.translate("File"), "ファイル");
        assert_eq!(Language::English.translate("File"), "File");
        assert_eq!(
            Language::Japanese.translate("Saved C:\\作品\\New project {}.fch"),
            "保存しました: C:\\作品\\New project {}.fch"
        );
        assert_eq!(
            Language::Japanese.translate("frame 2: 9 sprites on one scanline (NES limit: 8)"),
            "フレーム 2: 同一走査線に9個のスプライト (上限8個)"
        );
        assert_eq!(
            Language::Japanese.translate("my Map project.fch"),
            "my Map project.fch"
        );
    }
}
