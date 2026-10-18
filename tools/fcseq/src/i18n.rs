use super::*;
use std::{borrow::Cow, sync::OnceLock};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Language {
    English,
    Japanese,
}
impl Language {
    pub(super) fn detect() -> Self {
        if cfg!(test) {
            return Self::English;
        }
        if let Ok(s) = std::env::var("FCSEQ_LANGUAGE") {
            return if s.starts_with("ja") {
                Self::Japanese
            } else {
                Self::English
            };
        }
        if let Ok(s) = std::fs::read_to_string(root().join("out/language.txt")) {
            return if s.trim() == "ja" {
                Self::Japanese
            } else {
                Self::English
            };
        }
        for name in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(s) = std::env::var(name) {
                if !s.is_empty() {
                    return if s.starts_with("ja") {
                        Self::Japanese
                    } else {
                        Self::English
                    };
                }
            }
        }
        #[cfg(windows)]
        {
            #[link(name = "kernel32")]
            extern "system" {
                fn GetUserDefaultUILanguage() -> u16;
            }
            if unsafe { GetUserDefaultUILanguage() } & 0x3ff == 0x11 {
                return Self::Japanese;
            }
        }
        #[cfg(target_os = "macos")]
        if let Ok(output) = std::process::Command::new("/usr/bin/defaults")
            .args(["read", "-g", "AppleLanguages"]).output() {
            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout);
                if value.split(|c: char| !c.is_ascii_alphabetic()).find(|s| !s.is_empty()) == Some("ja") {
                    return Self::Japanese;
                }
            }
        }
        Self::English
    }
    pub(super) fn translate<'a>(self, text: &'a str) -> Cow<'a, str> {
        if self == Self::English {
            return Cow::Borrowed(text);
        }
        static CATALOG: OnceLock<Vec<(&str, &str)>> = OnceLock::new();
        let pairs = CATALOG.get_or_init(|| {
            include_str!("locale_ja.tsv")
                .lines()
                .filter_map(|l| l.split_once('\t'))
                .collect()
        });
        if let Some((_, j)) = pairs.iter().find(|(e, _)| *e == text) {
            return Cow::Borrowed(j);
        }
        if let Some((label, value)) = text.split_once("  ") {
            if let Some((_, j)) = pairs.iter().find(|(e, _)| *e == label) {
                return Cow::Owned(format!("{j}  {value}"));
            }
        }
        for (english, japanese) in pairs {
            if english.contains("{}") {
                if let Some(s) = template(text, english, japanese) {
                    return Cow::Owned(s);
                }
            }
        }
        Cow::Borrowed(text)
    }
}
fn template(text: &str, pattern: &str, translated: &str) -> Option<String> {
    let parts: Vec<_> = pattern.split("{}").collect();
    let mut tail = text.strip_prefix(parts[0])?;
    let mut values = vec![];
    for (i, part) in parts.iter().enumerate().skip(1) {
        if i == parts.len() - 1 {
            let value = tail.strip_suffix(part)?;
            values.push(value);
            tail = "";
        } else {
            let at = tail.find(part)?;
            values.push(&tail[..at]);
            tail = &tail[at + part.len()..];
        }
    }
    if !tail.is_empty() {
        return None;
    }
    let mut out = String::new();
    let mut pieces = translated.split("{}");
    out.push_str(pieces.next()?);
    for value in values {
        out.push_str(value);
        out.push_str(pieces.next()?);
    }
    if pieces.next().is_some() {
        return None;
    }
    Some(out)
}
impl Studio {
    pub(super) fn text(
        &self,
        d: &mut DrawList,
        x: f32,
        y: f32,
        text: &str,
        scale: f32,
        color: Color,
    ) {
        let t = self.language.translate(text);
        let scale=if !t.is_ascii(){scale.max(1.25)}else{scale};
        let width = measure_text_width(&t, scale);
        let available = (self.size.width - x - 12.0).max(1.0);
        draw_text(
            d,
            x,
            y,
            &t,
            scale * (available / width.max(1.0)).min(1.0),
            color,
        );
    }
    pub(super) fn toggle_language(&mut self) {
        self.language = if self.language == Language::English {
            Language::Japanese
        } else {
            Language::English
        };
        let dir = root().join("out");
        if !cfg!(test) {
            let result = std::fs::create_dir_all(&dir).and_then(|_| {
                std::fs::write(
                    dir.join("language.txt"),
                    if self.language == Language::Japanese {
                        "ja"
                    } else {
                        "en"
                    },
                )
            });
            if result.is_err() {
                self.status =
                    "Language changed for this session; preference could not be saved".into();
                return;
            }
        }
        self.status = "Display language changed".into();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn japanese_labels_templates_and_unicode_input() {
        assert_eq!(
            Language::Japanese.translate("KEY EDITOR  [E]"),
            "キーエディタ [E]"
        );
        assert_eq!(Language::Japanese.translate("WAVE 2 / 4"), "波形 2 / 4");
        assert_eq!(Language::English.translate("FILE"), "FILE");
        assert_eq!(short("日本語の曲名", 8), "日本語の曲名");
        let mut a = Studio::new(Size::new(1120.0, 780.0), None);
        a.apply(Action::Language);
        assert_eq!(a.language, Language::Japanese);
        a.apply(Action::Input(Field::Name));
        a.handle_event(UiEvent::KeyDown {
            key: "A".into(),
            ctrl: true,
            shift: false,
            alt: false,
            meta: false,
        });
        a.handle_event(UiEvent::ImeCommit {
            text: "日本語の曲名".into(),
        });
        a.handle_event(UiEvent::KeyDown {
            key: "Enter".into(),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        });
        assert_eq!(a.score.name, "日本語の曲名");
        a.draw_list();
    }
}
