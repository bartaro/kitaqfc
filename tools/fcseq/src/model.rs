use serde::{Deserialize, Serialize};
use std::path::Path;
pub const FPS: f64 = 60.098813897;
pub const MAX_ROWS: u16 = 4096;
pub const MAX_SECTIONS: usize = 16;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Expansion {
    #[default]
    None,
    Vrc6,
    Vrc7,
    Fds,
    N163,
}
impl Expansion {
    pub fn channels(self) -> usize {
        match self {
            Self::None => 5,
            Self::Vrc6 => 8,
            Self::Vrc7 => 11,
            Self::Fds => 6,
            Self::N163 => 13,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "2A03",
            Self::Vrc6 => "2A03 + VRC6",
            Self::Vrc7 => "2A03 + VRC7",
            Self::Fds => "2A03 + FDS",
            Self::N163 => "2A03 + N163",
        }
    }
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "none" | "2a03" => Ok(Self::None),
            "vrc6" => Ok(Self::Vrc6),
            "vrc7" => Ok(Self::Vrc7),
            "fds" => Ok(Self::Fds),
            "n163" | "namco163" => Ok(Self::N163),
            _ => Err("Choose none, vrc6, vrc7, fds or n163".into()),
        }
    }
    pub fn kind(self, ch: usize) -> Voice {
        match ch {
            0 | 1 => Voice::Pulse,
            2 => Voice::Triangle,
            3 => Voice::Noise,
            4 => Voice::Dpcm,
            _ => match self {
                Self::Vrc6 if ch == 7 => Voice::Saw,
                Self::Vrc6 => Voice::VrcPulse,
                Self::Vrc7 => Voice::Fm,
                Self::Fds => Voice::Fds,
                Self::N163 => Voice::N163,
                _ => Voice::Pulse,
            },
        }
    }
    pub fn channel_name(self, ch: usize) -> String {
        match ch {
            0 => "PULSE 1".into(),
            1 => "PULSE 2".into(),
            2 => "TRIANGLE".into(),
            3 => "NOISE".into(),
            4 => "DPCM".into(),
            _ => match self {
                Self::Vrc6 if ch == 7 => "VRC6 SAW".into(),
                Self::Vrc6 => format!("VRC6 PULSE {}", ch - 4),
                Self::Vrc7 => format!("VRC7 FM {}", ch - 4),
                Self::Fds => "FDS WAVE".into(),
                Self::N163 => format!("N163 WAVE {}", ch - 4),
                _ => "?".into(),
            },
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voice {
    Pulse,
    Triangle,
    Noise,
    Dpcm,
    VrcPulse,
    Saw,
    Fm,
    Fds,
    N163,
}
impl Voice {
    pub fn bounds(self) -> (u8, u8) {
        match self {
            Self::Pulse => (33, 95),
            Self::Triangle => (24, 95),
            Self::Noise => (1, 4),
            Self::Dpcm => (0, 15),
            Self::Fds => (24, 92),
            _ => (24, 95),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sound {
    pub volume: u8,
    pub duty: u8,
    pub decay: u8,
    pub instrument: u8,
    pub noise_period: Option<u8>,
    pub noise_short: bool,
    pub saw_rate: Option<u8>,
}
impl Default for Sound {
    fn default() -> Self {
        Self {
            volume: 12,
            duty: 2,
            decay: 0,
            instrument: 1,
            noise_period: None,
            noise_short: false,
            saw_rate: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub row: u16,
    pub length: u16,
    pub note: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<Sound>,
}
impl Note {
    pub fn new(row: u16, length: u16, note: u8) -> Self {
        Self {
            row,
            length,
            note,
            sound: None,
        }
    }
    pub fn sound(&self, ch: usize) -> Sound {
        self.sound.clone().unwrap_or_else(|| Sound {
            volume: if ch == 1 { 6 } else { 12 },
            duty: if ch == 1 { 1 } else { 2 },
            decay: if ch == 3 { 6 } else { 0 },
            instrument: if ch == 4 { 0 } else { 1 },
            ..Sound::default()
        })
    }
    pub fn end(&self) -> u16 {
        self.row.saturating_add(self.length)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub notes: Vec<Note>,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub solo: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    pub rows: u16,
    pub tracks: Vec<Track>,
}
impl Section {
    pub fn empty(name: &str) -> Self {
        Self::with_channels(name, 5)
    }
    pub fn with_channels(name: &str, channels: usize) -> Self {
        Self {
            name: name.into(),
            rows: 64,
            tracks: vec![Track::default(); channels],
        }
    }
    pub fn audible(&self, ch: usize) -> bool {
        !self.tracks[ch].mute && (!self.tracks.iter().any(|t| t.solo) || self.tracks[ch].solo)
    }
}
fn default_wave() -> Vec<u8> {
    (0..64)
        .map(|i| ((i as f64 * std::f64::consts::TAU / 64.0).sin() * 31.0 + 32.0).round() as u8)
        .collect()
}
fn default_patch() -> [u8; 8] {
    [0x03, 0x21, 0x05, 0x06, 0xe8, 0x81, 0x42, 0x27]
}
fn default_n163_channels() -> u8 {
    8
}
fn default_n163_waves() -> Vec<Vec<u8>> {
    (0..4)
        .map(|shape| crate::instruments::wave_preset(32, 15, shape))
        .collect()
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Score {
    pub format: String,
    pub name: String,
    pub bpm: f64,
    pub sections: Vec<Section>,
    #[serde(default)]
    pub expansion: Expansion,
    #[serde(default)]
    pub loop_playback: bool,
    #[serde(default = "default_wave")]
    pub fds_wave: Vec<u8>,
    #[serde(default = "default_patch")]
    pub vrc7_patch: [u8; 8],
    #[serde(default = "default_n163_channels")]
    pub n163_channels: u8,
    #[serde(default = "default_n163_waves")]
    pub n163_waves: Vec<Vec<u8>>,
    #[serde(default)]
    pub samples: Vec<crate::samples::Sample>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nsf: Option<crate::nsf::Imported>,
}
impl Score {
    pub fn empty() -> Self {
        Self {
            format: "fcseq-2".into(),
            name: "Untitled".into(),
            bpm: 150.0,
            sections: vec![Section::empty("SECTION 1")],
            expansion: Expansion::None,
            loop_playback: false,
            fds_wave: default_wave(),
            vrc7_patch: default_patch(),
            n163_channels: default_n163_channels(),
            n163_waves: default_n163_waves(),
            samples: vec![],
            nsf: None,
        }
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("nsf") || e.eq_ignore_ascii_case("nsfe"))
        {
            return crate::nsf::import(path);
        }
        Self::from_json(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        let mut s: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if s.format == "fcseq-1" && s.expansion == Expansion::None {
            for part in &mut s.sections {
                if part.tracks.len() == 4 {
                    part.tracks.push(Track::default());
                }
            }
        }
        s.validate()?;
        s.format = if s.nsf.is_some() {
            "fcseq-3"
        } else {
            "fcseq-2"
        }
        .into();
        Ok(s)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("nsf"))
        {
            return Err("Use Export NSF for .nsf files; save projects as .fcq".into());
        }
        self.validate()?;
        let mut s = self.clone();
        s.format = if s.nsf.is_some() {
            "fcseq-3"
        } else {
            "fcseq-2"
        }
        .into();
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&s).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    }
    pub fn frames_per_row(&self) -> u32 {
        (FPS * 15.0 / self.bpm).round() as u32
    }
    pub fn set_expansion(&mut self, expansion: Expansion) -> Result<(), String> {
        if self.nsf.is_some() {
            return Err("Imported NSF uses its original sound chips".into());
        }
        if expansion == self.expansion {
            return Ok(());
        }
        if self
            .sections
            .iter()
            .any(|s| s.tracks.iter().skip(5).any(|t| !t.notes.is_empty()))
        {
            return Err("Expansion tracks contain notes. Clear them or create a new project before switching chips.".into());
        }
        for s in &mut self.sections {
            s.tracks.truncate(5);
            s.tracks.resize(expansion.channels(), Track::default());
        }
        self.expansion = expansion;
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if let Some(nsf) = &self.nsf {
            nsf.validate()?;
            if self
                .sections
                .iter()
                .flat_map(|s| &s.tracks)
                .any(|t| !t.notes.is_empty())
            {
                return Err(
                    "Imported NSF is a playback document. Create a new project to edit notes."
                        .into(),
                );
            }
        }
        if !["fcseq-1", "fcseq-2", "fcseq-3"].contains(&self.format.as_str())
            || !self.bpm.is_finite()
            || !(30.0..=300.0).contains(&self.bpm)
            || !(1..=MAX_SECTIONS).contains(&self.sections.len())
        {
            return Err("Invalid format, tempo (30..300), or section count (1..16)".into());
        }
        if self.fds_wave.len() != 64 || self.fds_wave.iter().any(|v| *v > 63) {
            return Err("FDS wave requires 64 samples in 0..63".into());
        }
        if !(1..=8).contains(&self.n163_channels)
            || !(1..=16).contains(&self.n163_waves.len())
            || self
                .n163_waves
                .iter()
                .any(|w| w.is_empty() || w.len() % 4 != 0 || w.iter().any(|v| *v > 15))
            || self.n163_waves.iter().map(Vec::len).sum::<usize>() > self.n163_wave_capacity()
        {
            return Err("N163: use 1..8 channels, 1..16 waves of 4-bit samples, lengths divisible by 4; wave RAM must fit 256 - 16 * channels samples".into());
        }
        if self.samples.len() > 64 {
            return Err("At most 64 DPCM samples".into());
        }
        for sample in &self.samples {
            sample.validate()?;
        }
        for (part, s) in self.sections.iter().enumerate() {
            if s.rows < 16
                || s.rows > MAX_ROWS
                || s.rows % 16 != 0
                || s.tracks.len() != self.expansion.channels()
            {
                return Err(format!(
                    "Section {} needs 1..256 bars and {} tracks",
                    part + 1,
                    self.expansion.channels()
                ));
            }
            for (ch, t) in s.tracks.iter().enumerate() {
                if self.expansion == Expansion::N163
                    && ch >= 5 + self.n163_channels as usize
                    && !t.notes.is_empty()
                {
                    return Err("N163 track is disabled. Increase active channels in the instrument editor.".into());
                }
                let mut used = vec![false; s.rows as usize];
                let kind = self.expansion.kind(ch);
                let (lo, hi) = kind.bounds();
                for n in &t.notes {
                    if n.length == 0
                        || u32::from(n.row) + u32::from(n.length) > u32::from(s.rows)
                        || !(lo..=hi).contains(&n.note)
                    {
                        return Err(format!(
                            "Section {}, {}: note outside time/pitch range {}..{}",
                            part + 1,
                            self.expansion.channel_name(ch),
                            lo,
                            hi
                        ));
                    }
                    let sound = n.sound(ch);
                    if sound.volume > 15
                        || sound.duty > if kind == Voice::VrcPulse { 7 } else { 3 }
                        || sound.instrument > if kind == Voice::Dpcm { 63 } else { 15 }
                        || sound.noise_period.is_some_and(|p| p > 15)
                        || sound.saw_rate.is_some_and(|p| p > 63)
                    {
                        return Err("Invalid sound setting".into());
                    }
                    if kind == Voice::Dpcm && sound.instrument as usize >= self.samples.len() {
                        return Err(
                            "DPCM note references a missing sample. Import audio first.".into()
                        );
                    }
                    if kind == Voice::N163 {
                        let wave = self
                            .n163_waves
                            .get(sound.instrument as usize)
                            .ok_or("N163 note references a missing wave")?;
                        if crate::instruments::n163_frequency(
                            n.note,
                            wave.len(),
                            self.n163_channels,
                        ) > 0x3ffff
                        {
                            return Err("N163 pitch exceeds the chip range for this wave length and channel count; shorten the wave or reduce channels".into());
                        }
                    }
                    for r in n.row..n.row + n.length {
                        if used[r as usize] {
                            return Err(format!(
                                "Overlapping notes in {} / {}",
                                s.name,
                                self.expansion.channel_name(ch)
                            ));
                        }
                        used[r as usize] = true;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn n163_wave_capacity(&self) -> usize {
        256 - 16 * self.n163_channels as usize
    }
}
pub fn pitch_bounds(ch: usize) -> (u8, u8) {
    Expansion::None.kind(ch).bounds()
}
pub fn note_name(n: u8, ch: usize) -> String {
    if ch == 3 {
        return ["KICK", "SNARE", "CLOSED HAT", "OPEN HAT"][(n.clamp(1, 4) - 1) as usize].into();
    }
    if ch == 4 {
        return format!("RATE {}", n);
    }
    format!(
        "{}{}",
        ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"][(n % 12) as usize],
        n as i16 / 12 - 1
    )
}
