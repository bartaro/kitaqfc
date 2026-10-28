use fcseq_native::{
    editing::{self, Clipboard, Edit, Selection},
    model::*,
    nsf,
    samples::Sample,
    services,
};
use plita::prelude::*;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::result::Result;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    time::Instant,
};
const BG: Color = Color::rgb8(12, 18, 27);
const PANEL: Color = Color::rgb8(22, 31, 43);
const LINE: Color = Color::rgb8(46, 61, 78);
const TEXT: Color = Color::rgb8(223, 233, 244);
const MUTED: Color = Color::rgb8(138, 159, 181);
const ACCENT: Color = Color::rgb8(79, 211, 205);
#[path = "epsm_ui.rs"]
mod epsm_ui;
#[path = "i18n.rs"]
mod i18n;
#[path = "instrument_ui.rs"]
mod instrument_ui;
#[path = "key_editor.rs"]
mod key_editor;
#[path = "monitor.rs"]
mod monitor;
#[path = "wave_graph.rs"]
mod wave_graph;
use i18n::Language;
fn color(ch: usize) -> Color {
    [
        ACCENT,
        Color::rgb8(178, 151, 239),
        Color::rgb8(107, 214, 164),
        Color::rgb8(241, 178, 91),
        Color::rgb8(240, 132, 154),
        Color::rgb8(119, 173, 248),
    ][ch % 6]
}
fn root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_owned()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}
fn short(s: &str, n: usize) -> String {
    let v: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        format!("{v}..")
    } else {
        v
    }
}
fn logical_key(key: &str) -> String {
    // plita sends e.g. Named(Escape)|Code(Escape); retain support for plain
    // logical names too, without treating the physical key as typed text.
    let logical = key.split('|').next().unwrap_or(key);
    logical
        .strip_prefix("Named(")
        .and_then(|named| named.strip_suffix(')'))
        .unwrap_or(logical)
        .to_uppercase()
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Field {
    Name,
    SectionName,
    Bpm,
    Bars,
    Patch,
    SampleStart,
    SampleEnd,
    SampleGain,
    NsfSong,
    NsfSeconds,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Action {
    Language,
    WaveCurve,
    Menu(u8),
    New,
    Open,
    Save,
    SaveAs,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Duplicate,
    Delete,
    SelectAll,
    Quantize,
    Play,
    Stop,
    Audition,
    Wav,
    Export,
    ImportNsf,
    ExportNsf,
    NsfSong(i8),
    Loop,
    Fit,
    Home,
    Zoom(i8),
    Page(i8),
    Grid,
    TrackPage(i8),
    KeyEditor,
    KeyTrack(i8),
    KeyZoom(i8),
    Section(i8),
    AddSection,
    CloneSection,
    DeleteSection,
    MoveSection(i8),
    Input(Field),
    Chip,
    SetChip(Expansion),
    Pitch(i16),
    Length(i32),
    Row(i32),
    Volume(i16),
    Duty(i16),
    Decay(i16),
    SawRate(i16),
    Instrument(i16),
    NoisePeriod(i16),
    NoiseMode,
    Mute(usize),
    Solo(usize),
    Samples,
    ImportAudio,
    SampleStep(i8),
    SampleRate(i8),
    Normalize,
    Reverse,
    SamplePlay,
    ExportDpcm,
    RemoveSample,
    Wave,
    WavePreset(u8),
    InstrumentTrack(i8),
    InstrumentPlay,
    WaveStep(i8),
    WaveLength(i16),
    WaveInvert,
    WaveRotate(i8),
    N163Channels(i8),
    FmParam(usize, i16),
    EpsmParam(usize, i16),
    EpsmPatch(i8),
    EpsmOperator(i8),
    Timbre(u8, i16),
    Help,
    CloseModal,
}
#[derive(Clone)]
struct Button {
    action: Action,
    rect: Rect,
    label: String,
    icon: Option<&'static str>,
}
impl Button {
    fn new(
        action: Action,
        x: f32,
        y: f32,
        w: f32,
        label: impl Into<String>,
        icon: Option<&'static str>,
    ) -> Self {
        Self {
            action,
            rect: Rect::new(x, y, w, 30.0),
            label: label.into(),
            icon,
        }
    }
}
#[derive(Clone)]
struct Drag {
    origin: Point,
    before: Score,
    selection: Selection,
    ch: usize,
    resize: bool,
}
enum JobOutput {
    Audio(Vec<i16>, bool),
    Done(String),
}
pub struct Studio {
    language: Language,
    wave_curve: bool,
    size: Size,
    score: Score,
    saved: Score,
    path: Option<PathBuf>,
    part: usize,
    page: u16,
    visible: u16,
    first_track: usize,
    key_editor: bool,
    key_low: u8,
    key_rows: u8,
    track: usize,
    cursor: u16,
    snap: u16,
    selection: Selection,
    clipboard: Clipboard,
    undo: Vec<Score>,
    redo: Vec<Score>,
    drag: Option<Drag>,
    marquee: Option<(Point, Point)>,
    wave_before: Option<Score>,
    wave_last: Option<(usize, u8)>,
    instrument_track: usize,
    wave_slot: usize,
    epsm_slot: usize,
    epsm_operator: usize,
    scroll_drag: bool,
    pressed: Option<Action>,
    hover: Point,
    modifier: bool,
    shift: bool,
    menu: Option<u8>,
    modal: Option<u8>,
    input: Option<(Field, String, bool)>,
    sample: usize,
    status: String,
    job: Option<Receiver<Result<JobOutput, String>>>,
    cancel: Option<Arc<AtomicBool>>,
    audio: Option<(OutputStream, OutputStreamHandle)>,
    sink: Option<Sink>,
    started: Option<Instant>,
    playing_loop: bool,
    play_rows: u16,
    monitor: Option<monitor::Monitor>,
    smoke_ticks: Option<u8>,
    last_recovery: Instant,
    recovery_revision: usize,
    revision: usize,
}
impl Studio {
    pub fn new(size: Size, path: Option<PathBuf>) -> Self {
        let mut status =
            "Double-click a lane to add notes. Choose SOUND for expansion chips and samples."
                .into();
        let mut accepted = None;
        let score = if let Some(p) = path {
            match Score::load(&p) {
                Ok(s) => {
                    accepted = Some(p);
                    s
                }
                Err(e) => {
                    status = e;
                    Score::empty()
                }
            }
        } else {
            Score::empty()
        };
        let imported = accepted.as_ref().is_some_and(|p| {
            p.extension().is_some_and(|e| {
                e.eq_ignore_ascii_case("nsf")
                    || e.eq_ignore_ascii_case("nsfe")
                    || e.eq_ignore_ascii_case("nsf2")
            })
        });
        if imported {
            accepted = None;
            status =
                "NSF imported. Choose a song and press PLAY. Save as .fcq to keep it in a project."
                    .into();
        }
        Self {
            size,
            saved: if imported {
                Score::empty()
            } else {
                score.clone()
            },
            score,
            path: accepted,
            part: 0,
            page: 0,
            visible: 64,
            first_track: 0,
            language: Language::detect(),
            wave_curve: true,
            key_editor: false,
            key_low: 48,
            key_rows: 24,
            track: 0,
            cursor: 0,
            snap: 1,
            selection: vec![],
            clipboard: Clipboard::default(),
            undo: vec![],
            redo: vec![],
            drag: None,
            marquee: None,
            wave_before: None,
            wave_last: None,
            instrument_track: 0,
            wave_slot: 0,
            epsm_slot: 0,
            epsm_operator: 0,
            scroll_drag: false,
            pressed: None,
            hover: Point::new(-1.0, -1.0),
            modifier: false,
            shift: false,
            menu: None,
            modal: None,
            input: None,
            sample: 0,
            status,
            job: None,
            cancel: None,
            audio: None,
            sink: None,
            started: None,
            playing_loop: false,
            monitor: None,
            play_rows: 64,
            smoke_ticks: None,
            last_recovery: Instant::now(),
            recovery_revision: 0,
            revision: 0,
        }
    }
    pub fn enable_startup_smoke(&mut self) {
        self.smoke_ticks = Some(0);
    }
    fn dirty(&self) -> bool {
        self.score != self.saved
    }
    fn grid(&self) -> Rect {
        let left = if self.key_editor {
            16.0 + key_editor::KEYBOARD_WIDTH + 12.0
        } else {
            178.0
        };
        Rect::new(
            left,
            174.0,
            (self.size.width - left - 282.0).max(500.0),
            (self.size.height - 250.0).max(460.0),
        )
    }
    fn lane_count(&self) -> usize {
        if self.key_editor {
            return 1;
        }
        ((self.grid().height / 108.0).floor() as usize)
            .clamp(4, 8)
            .min(self.score.expansion.channels())
    }
    fn lane(&self, ch: usize) -> Rect {
        if self.key_editor {
            return self.grid();
        }
        let g = self.grid();
        Rect::new(
            g.x,
            g.y + (ch as f32 - self.first_track as f32) * g.height / self.lane_count() as f32,
            g.width,
            g.height / self.lane_count() as f32,
        )
    }
    fn step(&self) -> f32 {
        self.grid().width / self.visible as f32
    }
    fn rx(&self) -> f32 {
        self.size.width - 256.0
    }
    fn note_rect(&self, ch: usize, n: &Note) -> Rect {
        if self.key_editor {
            return self.key_note_rect(n);
        }
        let l = self.lane(ch);
        let (lo, hi) = self.score.expansion.kind(ch).bounds();
        let y =
            l.bottom() - 17.0 - (n.note - lo) as f32 / (hi - lo).max(1) as f32 * (l.height - 38.0);
        Rect::new(
            l.x + (n.row as f32 - self.page as f32) * self.step() + 1.0,
            y - 6.0,
            (n.length as f32 * self.step() - 2.0).max(5.0),
            14.0,
        )
    }
    fn hit(&self, p: Point) -> Option<(usize, usize)> {
        if !self.grid().contains(p) {
            return None;
        }
        for ch in self.first_track
            ..(self.first_track + self.lane_count()).min(self.score.expansion.channels())
        {
            for (i, n) in self.score.sections[self.part].tracks[ch]
                .notes
                .iter()
                .enumerate()
                .rev()
            {
                if self.note_rect(ch, n).contains(p) {
                    return Some((ch, i));
                }
            }
        }
        None
    }
    fn point_track(&self, p: Point) -> usize {
        if self.key_editor {
            return self.track;
        }
        (self.first_track
            + ((p.y - self.grid().y).max(0.0) / (self.grid().height / self.lane_count() as f32))
                as usize)
            .min(self.score.expansion.channels() - 1)
    }
    fn point_row(&self, p: Point) -> u16 {
        let r = (self.page as f32 + (p.x - self.grid().x) / self.step()).max(0.0) as u16;
        (r / self.snap * self.snap).min(self.score.sections[self.part].rows - 1)
    }
    fn clamp_view(&mut self) {
        self.part = self.part.min(self.score.sections.len() - 1);
        self.track = self.track.min(self.score.expansion.channels() - 1);
        self.instrument_track = self
            .instrument_track
            .min(self.score.expansion.channels() - 1);
        self.wave_slot = self.wave_slot.min(self.score.n163_waves.len() - 1);
        self.first_track = self.first_track.min(
            self.score
                .expansion
                .channels()
                .saturating_sub(self.lane_count()),
        );
        self.page = self.page.min(
            self.score.sections[self.part]
                .rows
                .saturating_sub(self.visible),
        );
        self.cursor = self.cursor.min(self.score.sections[self.part].rows - 1);
        self.sample = self.sample.min(self.score.samples.len().saturating_sub(1));
        if self.key_editor {
            self.first_track = self.track;
            self.clamp_key_range();
        }
    }
    fn record_history(&mut self, before: Score) -> bool {
        if before != self.score {
            self.undo.push(before);
            if self.undo.len() > 48 {
                self.undo.remove(0);
            }
            self.redo.clear();
            self.revision += 1;
            return true;
        }
        false
    }
    fn remember(&mut self, before: Score) {
        if self.record_history(before) { self.stop(); }
    }
    fn toggle_track_mix(&mut self, ch: usize, solo: bool) {
        let before = self.score.clone();
        let part = self.part;
        match editing::transaction(&mut self.score, |score| {
            let track = &mut score.sections[part].tracks[ch];
            if solo { track.solo = !track.solo; } else { track.mute = !track.mute; }
            Ok(())
        }) {
            Ok(()) => {
                if self.record_history(before) { self.update_monitor(); }
            }
            Err(error) => self.status = error,
        }
    }
    fn update_monitor(&mut self) {
        let Some(mut monitor) = self.monitor.take() else { return };
        monitor.sync_flags(&self.score);
        if monitor.is_ready() {
            self.status = if monitor.update() {
                "Updating track playback..."
            } else {
                "Playing / Space to stop"
            }.into();
        } else {
            // Initial rendering has not reached the device yet. Cancel that
            // receiver so the old flags can never start playing afterwards.
            self.launch_render(monitor.score.clone(), monitor.part, monitor.looped);
        }
        self.monitor = Some(monitor);
    }
    fn change(&mut self, f: impl FnOnce(&mut Score) -> Result<(), String>) {
        let before = self.score.clone();
        match editing::transaction(&mut self.score, f) {
            Ok(()) => {
                self.remember(before);
                self.clamp_view();
            }
            Err(e) => self.status = e,
        }
    }
    fn edit(&mut self, edit: Edit) {
        let before = self.score.clone();
        match editing::edit_notes(&mut self.score, self.part, &self.selection, edit) {
            Ok(ids) => {
                self.selection = ids;
                self.remember(before);
                self.status = format!("{} notes selected", self.selection.len());
            }
            Err(e) => self.status = e,
        }
    }
    fn undo(&mut self, redo: bool) {
        let next = if redo {
            self.redo.pop()
        } else {
            self.undo.pop()
        };
        if let Some(next) = next {
            let mix_only = monitor::same_music(&self.score, &next);
            if !mix_only { self.stop(); }
            let old = std::mem::replace(&mut self.score, next);
            if redo {
                self.undo.push(old);
            } else {
                self.redo.push(old);
            }
            self.selection.clear();
            self.clamp_view();
            self.revision += 1;
            if mix_only { self.update_monitor(); }
            self.status = if redo { "Redone" } else { "Undone" }.into();
        }
    }
    fn add_at(&mut self, p: Point) {
        if !self.grid().contains(p) {
            return;
        }
        let ch = self.point_track(p);
        let row = self.point_row(p);
        let (lo, hi) = self.score.expansion.kind(ch).bounds();
        let lane = self.lane(ch);
        let pitch = if self.key_editor {
            self.key_pitch(p.y)
        } else {
            (lo as f32 + (lane.bottom() - 17.0 - p.y) / (lane.height - 38.0) * (hi - lo) as f32)
                .round()
                .clamp(lo as f32, hi as f32) as u8
        };
        let length = self.snap.min(self.score.sections[self.part].rows - row);
        let mut note = Note::new(row, length, pitch);
        if self.score.expansion.kind(ch) == Voice::N163 {
            note.sound = Some(Sound {
                instrument: self.wave_slot as u8,
                ..Sound::default()
            });
        }
        if self.score.expansion.kind(ch) == Voice::Pcm {
            note.sound = Some(Sound {
                instrument: self.sample as u8,
                ..Sound::default()
            });
        }
        if ch == 4 && !self.key_editor {
            note.note = self
                .score
                .samples
                .get(self.sample)
                .map(|s| s.rate)
                .unwrap_or(15);
            note.sound = Some(Sound {
                instrument: self.sample as u8,
                ..Sound::default()
            });
        }
        if ch == 4 {
            note.sound = Some(Sound {
                instrument: self.sample as u8,
                ..Sound::default()
            });
        }
        let before = self.score.clone();
        let index = self.score.sections[self.part].tracks[ch].notes.len();
        let part = self.part;
        match editing::transaction(&mut self.score, |s| {
            s.sections[part].tracks[ch].notes.push(note);
            Ok(())
        }) {
            Ok(()) => {
                self.selection = vec![(ch, index)];
                self.track = ch;
                self.cursor = row;
                self.remember(before);
            }
            Err(e) => self.status = e,
        }
    }
    fn menu_buttons(&self) -> Vec<Button> {
        let Some(menu) = self.menu else {
            return vec![];
        };
        let x = 16.0 + menu as f32 * 96.0;
        let items: Vec<(Action, &str, &str)> = match menu {
            0 => vec![
                (Action::New, "New project     Ctrl+N", "file-plus"),
                (Action::Open, "Open...         Ctrl+O", "folder-open"),
                (Action::Save, "Save            Ctrl+S", "save"),
                (Action::SaveAs, "Save as...  Ctrl+Shift+S", "save"),
                (Action::ImportNsf, "Import NSF...", "folder-open"),
                (Action::ExportNsf, "Export NSF...", "download"),
                (Action::Wav, "Export WAV...", "volume-2"),
                (Action::Export, "Export C + DPCM...", "download"),
            ],
            1 => vec![
                (Action::Undo, "Undo            Ctrl+Z", "undo-2"),
                (Action::Redo, "Redo            Ctrl+Y", "redo-2"),
                (Action::Copy, "Copy            Ctrl+C", "copy"),
                (Action::Cut, "Cut             Ctrl+X", "eraser"),
                (Action::Paste, "Paste at cursor Ctrl+V", "clipboard-paste"),
                (Action::Duplicate, "Duplicate       Ctrl+D", "copy"),
                (Action::SelectAll, "Select lane     Ctrl+A", "check"),
                (Action::Quantize, "Quantize to grid     Q", "grid-2x2"),
                (Action::Delete, "Delete selection   Del", "trash-2"),
            ],
            2 => vec![
                (Action::AddSection, "Add section", "plus"),
                (Action::CloneSection, "Duplicate section", "copy"),
                (
                    Action::Input(Field::SectionName),
                    "Rename section",
                    "pencil",
                ),
                (Action::MoveSection(-1), "Move section left", "arrow-left"),
                (Action::MoveSection(1), "Move section right", "arrow-right"),
                (Action::DeleteSection, "Delete section", "trash-2"),
                (Action::Input(Field::Bars), "Section length (bars)", "move"),
                (Action::Input(Field::Name), "Project name", "pencil"),
            ],
            _ => vec![
                (
                    Action::SetChip(Expansion::None),
                    "2A03 + DPCM (5 channels)",
                    "volume-2",
                ),
                (
                    Action::SetChip(Expansion::Vrc6),
                    "VRC6 (8 channels)",
                    "volume-2",
                ),
                (
                    Action::SetChip(Expansion::Vrc7),
                    "VRC7 (11 channels)",
                    "volume-2",
                ),
                (
                    Action::SetChip(Expansion::Fds),
                    "FDS (6 channels)",
                    "volume-2",
                ),
                (Action::Samples, "DPCM sample editor", "volume-2"),
                (Action::ImportAudio, "Import audio file...", "folder-open"),
                (
                    Action::SetChip(Expansion::N163),
                    "N163 (up to 13 channels)",
                    "volume-2",
                ),
                (
                    Action::SetChip(Expansion::Mmc5),
                    "MMC5 (8 channels)",
                    "volume-2",
                ),
                (
                    Action::SetChip(Expansion::Sunsoft5b),
                    "SUNSOFT 5B (8 channels)",
                    "volume-2",
                ),
                (
                    Action::SetChip(Expansion::Epsm),
                    "EPSM (20 channels)",
                    "volume-2",
                ),
                (
                    Action::SetChip(Expansion::Vt02),
                    "VT02 (10 channels)",
                    "volume-2",
                ),
                (Action::Wave, "Instrument / waveform editor", "settings"),
                (Action::Help, "Keyboard / mouse help", "ellipsis"),
                (Action::Language, "日本語 / English", "settings"),
            ],
        };
        items
            .into_iter()
            .enumerate()
            .map(|(i, (a, t, icon))| {
                Button::new(a, x, 102.0 + i as f32 * 34.0, 290.0, t, Some(icon))
            })
            .collect()
    }
    fn buttons(&self) -> Vec<Button> {
        if self.score.nsf.is_some() {
            return self.nsf_buttons();
        }
        if self.modal.is_some() {
            return self.modal_buttons();
        }
        let mut b = vec![];
        for (i, label) in ["FILE", "EDIT", "SONG", "SOUND"].iter().enumerate() {
            b.push(Button::new(
                Action::Menu(i as u8),
                16.0 + i as f32 * 96.0,
                66.0,
                88.0,
                *label,
                Some(["folder-open", "pencil", "volume-2", "settings"][i]),
            ));
        }
        for (a, x, w, label, icon) in [
            (
                Action::Play,
                420.0,
                94.0,
                if self.started.is_some() {
                    "PLAYING"
                } else {
                    "PLAY"
                },
                "play",
            ),
            (Action::Stop, 522.0, 78.0, "STOP", "square"),
            (
                Action::Loop,
                608.0,
                112.0,
                if self.score.loop_playback {
                    "LOOP ON"
                } else {
                    "LOOP OFF"
                },
                "rotate-cw",
            ),
            (Action::Audition, 728.0, 112.0, "AUDITION", "volume-2"),
        ] {
            b.push(Button::new(a, x, 66.0, w, label, Some(icon)));
        }
        b.push(Button::new(
            Action::KeyEditor,
            852.0,
            66.0,
            168.0,
            if self.key_editor {
                "ALL TRACKS  [E]"
            } else {
                "KEY EDITOR  [E]"
            },
            None,
        ));
        b.push(Button::new(
            Action::Input(Field::Bpm),
            self.size.width - 236.0,
            18.0,
            116.0,
            format!("{:.0} BPM", self.score.bpm),
            Some("step-forward"),
        ));
        b.push(Button::new(
            Action::Samples,
            self.size.width - 112.0,
            18.0,
            96.0,
            "SAMPLES",
            None,
        ));
        for (a, x, w, label) in [
            (Action::Section(-1), 16.0, 30.0, "<".into()),
            (
                Action::Input(Field::SectionName),
                52.0,
                190.0,
                format!(
                    "{:02} {}",
                    self.part + 1,
                    short(&self.score.sections[self.part].name, 16)
                ),
            ),
            (Action::Section(1), 248.0, 30.0, ">".into()),
            (
                Action::Input(Field::Bars),
                292.0,
                92.0,
                format!("{} BARS", self.score.sections[self.part].rows / 16),
            ),
            (
                Action::Grid,
                398.0,
                96.0,
                format!("GRID 1/{}", 16 / self.snap),
            ),
            (Action::Fit, 506.0, 52.0, "FIT".into()),
            (Action::Home, 566.0, 58.0, "HOME".into()),
            (Action::Page(-1), 638.0, 30.0, "<".into()),
            (Action::Page(1), 674.0, 30.0, ">".into()),
            (Action::Zoom(-1), 718.0, 30.0, "-".into()),
            (Action::Zoom(1), 754.0, 30.0, "+".into()),
        ] {
            b.push(Button::new(a, x, 116.0, w, label, None));
        }
        b.push(Button::new(
            Action::Chip,
            self.rx(),
            116.0,
            236.0,
            self.score.expansion.label(),
            Some("volume-2"),
        ));
        for ch in self.first_track..self.first_track + self.lane_count() {
            let l = self.lane(ch);
            if self.key_editor {
                continue;
            }
            b.push(Button::new(
                Action::Mute(ch),
                24.0,
                l.y + 56.0,
                54.0,
                "MUTE",
                None,
            ));
            b.push(Button::new(
                Action::Solo(ch),
                84.0,
                l.y + 56.0,
                54.0,
                "SOLO",
                None,
            ));
        }
        if self.key_editor {
            for (a, x, label) in [
                (Action::KeyTrack(-1), 16.0, "<"),
                (Action::KeyTrack(1), 64.0, ">"),
            ] {
                b.push(Button::new(
                    a,
                    x,
                    self.grid().bottom() + 10.0,
                    40.0,
                    label,
                    None,
                ));
            }
            for (action, offset, label) in [
                (Action::Mute(self.track), 0.0, "MUTE"),
                (Action::Solo(self.track), 124.0, "SOLO"),
            ] {
                let mut button = Button::new(action, self.rx() + offset, 146.0, 112.0, label, None);
                button.rect.height = 24.0;
                b.push(button);
            }
            b.push(Button::new(
                Action::KeyZoom(-1),
                self.rx(),
                68.0,
                30.0,
                "-",
                None,
            ));
            b.push(Button::new(
                Action::KeyZoom(1),
                self.rx() + 196.0,
                68.0,
                30.0,
                "+",
                None,
            ));
            // Key-editor toggle occupies this row; vertical zoom uses the left
            // keyboard header instead, preserving the transport layout.
            let len = b.len();
            let keys = self.keyboard_rect();
            b[len - 2].rect = Rect::new(keys.x, 146.0, 24.0, 24.0);
            b[len - 1].rect = Rect::new(keys.right() - 24.0, 146.0, 24.0, 24.0);
        } else {
            b.push(Button::new(
                Action::TrackPage(-1),
                22.0,
                self.grid().bottom() + 10.0,
                56.0,
                "UP",
                None,
            ));
            b.push(Button::new(
                Action::TrackPage(1),
                84.0,
                self.grid().bottom() + 10.0,
                56.0,
                "DOWN",
                None,
            ));
        }
        if let Some(&(ch, _)) = self.selection.first() {
            let rx = self.rx();
            let kind = self.score.expansion.kind(ch);
            let mut specs = vec![
                (Action::Pitch(-1), Action::Pitch(1)),
                (Action::Length(-1), Action::Length(1)),
                (Action::Row(-1), Action::Row(1)),
            ];
            if !matches!(
                kind,
                Voice::Triangle | Voice::Dpcm | Voice::VtTriangle | Voice::Pcm
            ) {
                if kind != Voice::Saw {
                    specs.push((Action::Volume(-1), Action::Volume(1)));
                }
                specs.push((Action::Decay(-1), Action::Decay(1)));
            }
            match kind {
                Voice::Pulse | Voice::VrcPulse | Voice::MmcPulse | Voice::VtPulse => {
                    specs.push((Action::Duty(-1), Action::Duty(1)))
                }
                Voice::Fm | Voice::Dpcm | Voice::N163 | Voice::EpsmFm | Voice::Pcm => {
                    specs.push((Action::Instrument(-1), Action::Instrument(1)))
                }
                Voice::Noise | Voice::VtNoise => {
                    specs.push((Action::NoisePeriod(-1), Action::NoisePeriod(1)))
                }
                Voice::Saw => specs.push((Action::SawRate(-1), Action::SawRate(1))),
                _ => {}
            }
            for (i, (a, z)) in specs.into_iter().enumerate() {
                let y = 278.0 + i as f32 * 44.0;
                b.push(Button::new(a, rx, y, 30.0, "-", None));
                b.push(Button::new(z, rx + 206.0, y, 30.0, "+", None));
            }
            if kind == Voice::Noise {
                b.push(Button::new(
                    Action::NoiseMode,
                    rx,
                    552.0,
                    236.0,
                    "TOGGLE NOISE MODE",
                    None,
                ));
            }
            b.push(Button::new(
                Action::Duplicate,
                rx,
                598.0,
                112.0,
                "DUPLICATE",
                None,
            ));
            b.push(Button::new(
                Action::Delete,
                rx + 120.0,
                598.0,
                116.0,
                "DELETE",
                None,
            ));
        }
        b.push(Button::new(
            Action::Wave,
            self.rx(),
            644.0,
            236.0,
            "INSTRUMENT / WAVEFORM",
            Some("settings"),
        ));
        b.extend(self.menu_buttons());
        b
    }
    fn enabled(&self, a: Action) -> bool {
        if let Some(source) = &self.score.nsf {
            return match a {
                Action::Language => true,
                Action::Menu(0)
                | Action::New
                | Action::Open
                | Action::Save
                | Action::SaveAs
                | Action::ImportNsf
                | Action::ExportNsf
                | Action::Wav
                | Action::Play
                | Action::Stop
                | Action::Loop
                | Action::Undo
                | Action::Redo
                | Action::Input(Field::Name)
                | Action::Input(Field::NsfSong)
                | Action::Input(Field::NsfSeconds) => true,
                Action::NsfSong(d) => source.validate().is_ok_and(|h| {
                    source.song as i16 + d as i16 >= 0
                        && (source.song as i16 + d as i16) < h.songs as i16
                }),
                _ => false,
            };
        }
        match a {
            Action::Timbre(_, _) => !self.instrument_selection().is_empty(),
            Action::Undo => !self.undo.is_empty(),
            Action::Redo => !self.redo.is_empty(),
            Action::Copy
            | Action::Cut
            | Action::Duplicate
            | Action::Delete
            | Action::Quantize
            | Action::Audition => !self.selection.is_empty(),
            Action::Paste => !self.clipboard.notes.is_empty(),
            Action::DeleteSection => self.score.sections.len() > 1,
            Action::AddSection | Action::CloneSection => self.score.sections.len() < MAX_SECTIONS,
            Action::Section(d) | Action::MoveSection(d) => {
                let p = self.part as i32 + d as i32;
                p >= 0 && p < self.score.sections.len() as i32
            }
            Action::SamplePlay | Action::ExportDpcm | Action::RemoveSample => {
                !self.score.samples.is_empty()
            }
            _ => true,
        }
    }
    fn action_at(&self, p: Point) -> Option<Action> {
        let bs = if self.menu.is_some() {
            self.menu_buttons()
        } else {
            self.buttons()
        };
        bs.into_iter()
            .rev()
            .find(|b| b.rect.contains(p) && self.enabled(b.action))
            .map(|b| b.action)
    }
    fn save(&mut self, save_as: bool) -> bool {
        let path = if save_as || self.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("FCSEQ project", &["fcq"])
                .set_file_name("song.fcq")
                .save_file()
        } else {
            self.path.clone()
        };
        let Some(mut path) = path else {
            return false;
        };
        path.set_extension("fcq");
        match self.score.save(&path) {
            Ok(()) => {
                self.saved = self.score.clone();
                self.path = Some(path);
                self.status = "Project saved".into();
                true
            }
            Err(e) => {
                self.status = e;
                false
            }
        }
    }
    fn can_discard(&mut self) -> bool {
        if !self.dirty() {
            return true;
        }
        match rfd::MessageDialog::new()
            .set_title(self.language.translate("Unsaved FCSEQ project"))
            .set_description(
                self.language
                    .translate("Save your changes before continuing?"),
            )
            .set_buttons(rfd::MessageButtons::YesNoCancel)
            .show()
        {
            rfd::MessageDialogResult::Yes => self.save(false),
            rfd::MessageDialogResult::No => true,
            _ => false,
        }
    }
    fn open(&mut self, path: PathBuf) {
        match Score::load(&path) {
            Ok(score) => {
                if !self.can_discard() {
                    return;
                }
                self.stop();
                self.score = score;
                let imported = path.extension().is_some_and(|e| {
                    e.eq_ignore_ascii_case("nsf")
                        || e.eq_ignore_ascii_case("nsfe")
                        || e.eq_ignore_ascii_case("nsf2")
                });
                self.saved = if imported {
                    Score::empty()
                } else {
                    self.score.clone()
                };
                self.path = if imported { None } else { Some(path) };
                self.part = 0;
                self.page = 0;
                self.first_track = 0;
                self.selection.clear();
                self.undo.clear();
                self.redo.clear();
                self.clamp_view();
                self.status = if imported {
                    "NSF imported. Select a song and press PLAY."
                } else {
                    "Project opened"
                }
                .into();
            }
            Err(e) => self.status = e,
        }
    }
    fn begin_input(&mut self, field: Field) {
        let text = match field {
            Field::Name => self.score.name.clone(),
            Field::SectionName => self.score.sections[self.part].name.clone(),
            Field::Bpm => format!("{}", self.score.bpm),
            Field::Bars => format!("{}", self.score.sections[self.part].rows / 16),
            Field::NsfSong => self
                .score
                .nsf
                .as_ref()
                .map(|n| (n.song as u16 + 1).to_string())
                .unwrap_or_default(),
            Field::NsfSeconds => self
                .score
                .nsf
                .as_ref()
                .map(|n| n.seconds.to_string())
                .unwrap_or_default(),
            Field::Patch => self
                .score
                .vrc7_patch
                .iter()
                .map(|v| format!("{v:02X}"))
                .collect::<Vec<_>>()
                .join(" "),
            _ => {
                let Some(s) = self.score.samples.get(self.sample) else {
                    return;
                };
                match field {
                    Field::SampleStart => {
                        format!("{:.1}", s.start as f64 * 1000.0 / s.sample_rate as f64)
                    }
                    Field::SampleEnd => {
                        format!("{:.1}", s.end as f64 * 1000.0 / s.sample_rate as f64)
                    }
                    _ => format!("{:.2}", s.gain),
                }
            }
        };
        self.input = Some((field, text, true));
    }
    fn finish_input(&mut self) {
        let Some((field, text, _)) = self.input.take() else {
            return;
        };
        let part = self.part;
        let index = self.sample;
        if matches!(field, Field::NsfSong | Field::NsfSeconds) {
            self.stop();
        }
        self.change(|s| {
            match field {
                Field::Name => s.name = text.trim().into(),
                Field::SectionName => s.sections[part].name = text.trim().into(),
                Field::Bpm => s.bpm = text.parse().map_err(|_| "Enter a BPM number")?,
                Field::NsfSong => {
                    let song: u8 = text
                        .parse()
                        .map_err(|_| "Enter a song number starting at 1")?;
                    s.nsf
                        .as_mut()
                        .ok_or("Import an NSF first")?
                        .select_song(song.checked_sub(1).ok_or("Song numbers start at 1")?)?;
                }
                Field::NsfSeconds => {
                    s.nsf.as_mut().ok_or("Import an NSF first")?.seconds =
                        text.parse().map_err(|_| "Enter 1..600 seconds")?;
                    s.nsf.as_mut().unwrap().duration_ms = None;
                }
                Field::Bars => {
                    s.sections[part].rows = text
                        .parse::<u16>()
                        .map_err(|_| "Enter 1..256 bars")?
                        .checked_mul(16)
                        .ok_or("Too many bars")?
                }
                Field::Patch => {
                    let values = text
                        .split_whitespace()
                        .map(|v| {
                            u8::from_str_radix(v, 16).map_err(|_| "Use eight hexadecimal bytes")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    s.vrc7_patch = values
                        .try_into()
                        .map_err(|_| "Use eight hexadecimal bytes")?;
                }
                _ => {
                    let sample = s.samples.get_mut(index).ok_or("Import a sample first")?;
                    let v: f64 = text.parse().map_err(|_| "Enter a number")?;
                    if !v.is_finite() || v < 0.0 {
                        return Err("Enter a positive finite value".into());
                    }
                    match field {
                        Field::SampleStart => {
                            sample.start = (v * sample.sample_rate as f64 / 1000.0).round() as u32
                        }
                        Field::SampleEnd => {
                            sample.end = (v * sample.sample_rate as f64 / 1000.0).round() as u32
                        }
                        _ => sample.gain = v as f32,
                    }
                    sample.validate()?;
                }
            }
            Ok(())
        });
    }
    fn apply(&mut self, a: Action) {
        if !self.enabled(a) {
            return;
        }
        if !matches!(a, Action::Menu(_)) {
            self.menu = None;
        }
        let part = self.part;
        match a {
            Action::Menu(m) => self.menu = if self.menu == Some(m) { None } else { Some(m) },
            Action::New => {
                if self.can_discard() {
                    self.stop();
                    self.score = Score::empty();
                    self.saved = self.score.clone();
                    self.path = None;
                    self.part = 0;
                    self.page = 0;
                    self.first_track = 0;
                    self.selection.clear();
                    self.undo.clear();
                    self.redo.clear();
                    self.clamp_view();
                    self.status = "New project - double-click a lane to add notes".into();
                }
            }
            Action::Open => {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter(
                        "FCSEQ project / NSF / NSFe",
                        &["fcq", "nsf", "nsf2", "nsfe"],
                    )
                    .pick_file()
                {
                    self.open(p)
                }
            }
            Action::Save | Action::SaveAs => {
                self.save(a == Action::SaveAs);
            }
            Action::ImportNsf => {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("NSF / NSFe music", &["nsf", "nsf2", "nsfe"])
                    .pick_file()
                {
                    self.open(p);
                }
            }
            Action::ExportNsf => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("NSFe music (with track lengths)", &["nsfe"])
                    .add_filter("NSF2 music (+ NSFe companion)", &["nsf2"])
                    .add_filter("NSF music (+ NSFe companion)", &["nsf"])
                    .set_file_name("song.nsfe")
                    .save_file()
                {
                    self.start_nsf_export(path);
                }
            }
            Action::NsfSong(delta) => {
                self.stop();
                self.change(|s| {
                    let n = s.nsf.as_mut().ok_or("Import an NSF first")?;
                    let song = (n.song as i16 + delta as i16)
                        .try_into()
                        .map_err(|_| "Song out of range")?;
                    n.select_song(song)?;
                    Ok(())
                });
            }
            Action::Undo => self.undo(false),
            Action::Redo => self.undo(true),
            Action::Copy | Action::Cut => {
                self.clipboard = Clipboard::copy(&self.score, part, &self.selection);
                if a == Action::Cut {
                    self.edit(Edit::Delete);
                }
                self.status = format!(
                    "Copied {} notes; click destination, then Ctrl+V",
                    self.clipboard.notes.len()
                );
            }
            Action::Paste => {
                let before = self.score.clone();
                match self.clipboard.paste(&mut self.score, part, self.cursor) {
                    Ok(ids) => {
                        self.selection = ids;
                        self.remember(before);
                    }
                    Err(e) => self.status = e,
                }
            }
            Action::Duplicate => self.edit(Edit::Duplicate),
            Action::Delete => self.edit(Edit::Delete),
            Action::Quantize => self.edit(Edit::Quantize(self.snap)),
            Action::SelectAll => {
                self.selection = (0..self.score.sections[part].tracks[self.track].notes.len())
                    .map(|i| (self.track, i))
                    .collect()
            }
            Action::Pitch(d) => self.edit(Edit::Transpose(d)),
            Action::Length(d) => self.edit(Edit::Resize(d)),
            Action::Row(d) => self.edit(Edit::Move(d)),
            Action::Volume(d) => self.edit(Edit::Volume(d)),
            Action::Duty(d) => self.edit(Edit::Duty(d)),
            Action::Decay(d) => self.edit(Edit::Decay(d)),
            Action::SawRate(d) => self.edit(Edit::SawRate(d)),
            Action::Instrument(d) => {
                self.edit(Edit::Instrument(d));
            }
            Action::NoisePeriod(d) => self.edit(Edit::NoisePeriod(d)),
            Action::NoiseMode => self.edit(Edit::NoiseMode),
            Action::Play => {
                if self.started.is_some() || self.job.is_some() {
                    self.stop();
                } else {
                    self.start_audio(false);
                }
            }
            Action::Audition => self.start_audio(true),
            Action::Stop => {
                self.stop();
                self.status = if self.job.is_some() {
                    "Playback stopped; file export continues"
                } else {
                    "Stopped"
                }
                .into();
            }
            Action::Wav => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("WAV", &["wav"])
                    .set_file_name("section.wav")
                    .save_file()
                {
                    self.start_job(Some(path), false);
                }
            }
            Action::Export => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Export C driver, register data and DPCM")
                    .pick_folder()
                {
                    self.start_job(Some(path), true);
                }
            }
            Action::Loop => self.change(|s| {
                s.loop_playback = !s.loop_playback;
                Ok(())
            }),
            Action::Fit => {
                self.visible = self.score.sections[part].rows;
                self.page = 0;
            }
            Action::Home => {
                self.page = 0;
                self.cursor = 0;
            }
            Action::Zoom(d) => {
                self.visible = if d > 0 {
                    (self.visible / 2).max(16)
                } else {
                    (self.visible * 2).min(MAX_ROWS)
                };
                self.clamp_view();
            }
            Action::Page(d) => {
                self.page = (self.page as i32 + d as i32 * self.visible as i32 / 2).clamp(
                    0,
                    self.score.sections[part].rows.saturating_sub(self.visible) as i32,
                ) as u16;
            }
            Action::Grid => self.snap = if self.snap == 16 { 1 } else { self.snap * 2 },
            Action::TrackPage(d) => {
                self.first_track =
                    (self.first_track as i32 + d as i32 * self.lane_count() as i32).max(0) as usize;
                self.clamp_view();
            }
            Action::Section(d) => {
                self.stop();
                self.part = (self.part as i32 + d as i32)
                    .clamp(0, self.score.sections.len() as i32 - 1)
                    as usize;
                self.page = 0;
                self.cursor = 0;
                self.selection.clear();
            }
            Action::AddSection => {
                self.change(|s| {
                    s.sections.push(Section::with_channels(
                        &format!("SECTION {}", s.sections.len() + 1),
                        s.expansion.channels(),
                    ));
                    Ok(())
                });
                self.part = self.score.sections.len() - 1;
                self.page = 0;
                self.selection.clear();
            }
            Action::CloneSection => {
                self.change(|s| {
                    let mut section = s.sections[part].clone();
                    section.name += " COPY";
                    s.sections.insert(part + 1, section);
                    Ok(())
                });
                self.part += 1;
                self.selection.clear();
            }
            Action::DeleteSection => {
                self.change(|s| {
                    s.sections.remove(part);
                    Ok(())
                });
                self.selection.clear();
            }
            Action::MoveSection(d) => {
                let other = (part as i32 + d as i32) as usize;
                self.change(|s| {
                    s.sections.swap(part, other);
                    Ok(())
                });
                self.part = other;
            }
            Action::Input(f) => self.begin_input(f),
            Action::KeyEditor => {
                self.key_editor = !self.key_editor;
                if self.key_editor {
                    self.open_key_track(self.track)
                } else {
                    self.first_track = self.track.saturating_sub(1);
                    self.clamp_view();
                }
            }
            Action::KeyTrack(d) => {
                let track = (self.track as i32 + d as i32)
                    .rem_euclid(self.score.expansion.channels() as i32)
                    as usize;
                self.open_key_track(track);
            }
            Action::KeyZoom(d) => {
                self.key_rows = (self.key_rows as i16 - d as i16 * 12).clamp(12, 60) as u8;
                self.clamp_key_range();
            }
            Action::Chip | Action::SetChip(_) => {
                let chip = if let Action::SetChip(chip) = a {
                    chip
                } else {
                    let index = Expansion::ALL
                        .iter()
                        .position(|c| *c == self.score.expansion)
                        .unwrap_or(0);
                    Expansion::ALL[(index + 1) % Expansion::ALL.len()]
                };
                self.change(|s| s.set_expansion(chip));
                self.selection.clear();
                self.status = if self.score.expansion == chip {
                    self.first_track = self
                        .score
                        .expansion
                        .channels()
                        .saturating_sub(self.lane_count());
                    self.track = if chip == Expansion::None { 0 } else { 5 };
                    format!("{} / {} channels", chip.label(), chip.channels())
                } else {
                    self.status.clone()
                };
            }
            Action::Mute(c) => self.toggle_track_mix(c, false),
            Action::Solo(c) => self.toggle_track_mix(c, true),
            Action::Samples => self.modal = Some(0),
            Action::Wave => {
                self.instrument_track = self.selection.first().map(|v| v.0).unwrap_or(self.track);
                self.wave_slot =
                    self.instrument_sound().instrument as usize % self.score.n163_waves.len();
                self.epsm_slot = self.instrument_sound().instrument.min(15) as usize;
                self.modal = Some(1);
            }
            Action::Help => self.modal = Some(2),
            Action::Language => self.toggle_language(),
            Action::WaveCurve => self.wave_curve = !self.wave_curve,
            Action::CloseModal => {
                self.modal = None;
                self.input = None;
            }
            Action::ImportAudio => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Audio", &["wav", "flac", "ogg", "mp3"])
                    .pick_file()
                {
                    match Sample::import(&path) {
                        Ok(sample) => {
                            let limit = (sample.end as usize) < sample.pcm.len();
                            let index = self.score.samples.len();
                            self.change(|s| {
                                s.samples.push(sample);
                                Ok(())
                            });
                            if self.score.samples.len() == index {
                                return;
                            }
                            self.sample = index.min(self.score.samples.len().saturating_sub(1));
                            self.modal = Some(0);
                            self.status=if limit{"Imported source. Initial region shortened to DPCM capacity; full source retained."}else{"Audio imported. Process the region, then add notes in the DPCM lane."}.into();
                        }
                        Err(e) => self.status = e,
                    }
                }
            }
            Action::SampleStep(d) => {
                self.sample = (self.sample as i32 + d as i32)
                    .clamp(0, self.score.samples.len().saturating_sub(1) as i32)
                    as usize;
            }
            Action::SampleRate(_) | Action::Normalize | Action::Reverse => {
                let index = self.sample;
                self.change(|s| {
                    let sample = s.samples.get_mut(index).ok_or("Import audio first")?;
                    match a {
                        Action::SampleRate(d) => {
                            sample.rate = (sample.rate as i16 + d as i16).clamp(0, 15) as u8
                        }
                        Action::Normalize => sample.normalize = !sample.normalize,
                        _ => sample.reverse = !sample.reverse,
                    };
                    sample.validate()?;
                    Ok(())
                });
            }
            Action::SamplePlay => self.play_sample(),
            Action::ExportDpcm => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("DPCM", &["dmc"])
                    .set_file_name("sample.dmc")
                    .save_file()
                {
                    match self.score.samples[self.sample]
                        .encode()
                        .and_then(|bytes| std::fs::write(path, bytes).map_err(|e| e.to_string()))
                    {
                        Ok(()) => self.status = "DPCM exported".into(),
                        Err(e) => self.status = e,
                    }
                }
            }
            Action::RemoveSample => {
                let index = self.sample;
                self.change(|s| {
                    if s.sections.iter().any(|p| {
                        p.tracks[4]
                            .notes
                            .iter()
                            .any(|n| n.sound(4).instrument as usize == index)
                    }) {
                        return Err(
                            "Sample is used by DPCM notes; remove or reassign those notes first"
                                .into(),
                        );
                    }
                    s.samples.remove(index);
                    for p in &mut s.sections {
                        for n in &mut p.tracks[4].notes {
                            let mut sound = n.sound(4);
                            if sound.instrument as usize > index {
                                sound.instrument -= 1;
                            }
                            n.sound = Some(sound);
                        }
                    }
                    Ok(())
                });
            }
            Action::WavePreset(_)
            | Action::InstrumentTrack(_)
            | Action::InstrumentPlay
            | Action::WaveStep(_)
            | Action::WaveLength(_)
            | Action::WaveInvert
            | Action::WaveRotate(_)
            | Action::N163Channels(_)
            | Action::EpsmParam(_, _)
            | Action::EpsmPatch(_)
            | Action::EpsmOperator(_)
            | Action::FmParam(_, _)
            | Action::Timbre(_, _) => self.instrument_action(a),
        }
    }
    fn stop(&mut self) {
        self.monitor = None;
        if let Some(c) = self.cancel.take() {
            c.store(true, Ordering::Relaxed);
            self.job = None;
        }
        if let Some(s) = self.sink.take() {
            s.stop();
        }
        self.started = None;
    }
    fn start_nsf_export(&mut self, mut path: PathBuf) {
        if self.job.is_some() && self.cancel.is_none() {
            self.status = "A file export is already running".into();
            return;
        }
        self.stop();
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("nsfe") || ext.eq_ignore_ascii_case("nsf2"))
        {
            path.set_extension("nsf");
        }
        let score = self.score.clone();
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        self.status = "Exporting NSF...".into();
        std::thread::spawn(move || {
            let result = nsf::export(&score, &path)
                .map(|_| JobOutput::Done(format!("Exported {}", path.display())));
            let _ = tx.send(result);
        });
    }
    fn nsf_buttons(&self) -> Vec<Button> {
        let Some(source) = &self.score.nsf else {
            return vec![];
        };
        let songs = source.validate().map(|h| h.songs).unwrap_or(1);
        let mut buttons = vec![
            Button::new(
                Action::Menu(0),
                16.0,
                66.0,
                88.0,
                "FILE",
                Some("folder-open"),
            ),
            Button::new(
                Action::Play,
                120.0,
                66.0,
                106.0,
                if self.started.is_some() {
                    "PLAYING"
                } else {
                    "PLAY"
                },
                Some("play"),
            ),
            Button::new(Action::Stop, 238.0, 66.0, 96.0, "STOP", Some("square")),
            Button::new(
                Action::Loop,
                346.0,
                66.0,
                116.0,
                if self.score.loop_playback {
                    "LOOP ON"
                } else {
                    "LOOP OFF"
                },
                Some("rotate-cw"),
            ),
            Button::new(
                Action::NsfSong(-1),
                48.0,
                208.0,
                88.0,
                "PREV",
                Some("arrow-left"),
            ),
            Button::new(
                Action::Input(Field::NsfSong),
                148.0,
                208.0,
                260.0,
                format!("SONG {} / {}", source.song as u16 + 1, songs),
                None,
            ),
            Button::new(
                Action::NsfSong(1),
                420.0,
                208.0,
                88.0,
                "NEXT",
                Some("arrow-right"),
            ),
            Button::new(
                Action::Input(Field::NsfSeconds),
                540.0,
                208.0,
                220.0,
                format!("PLAYBACK {} SECONDS", source.seconds),
                None,
            ),
            Button::new(
                Action::SaveAs,
                48.0,
                620.0,
                200.0,
                "SAVE PROJECT .FCQ",
                Some("save"),
            ),
            Button::new(
                Action::ExportNsf,
                268.0,
                620.0,
                180.0,
                "EXPORT .NSF",
                Some("download"),
            ),
            Button::new(
                Action::Wav,
                468.0,
                620.0,
                180.0,
                "EXPORT WAV",
                Some("volume-2"),
            ),
        ];
        buttons.push(Button::new(
            Action::Language,
            self.size.width - 200.0,
            66.0,
            184.0,
            "日本語 / English",
            None,
        ));
        buttons.extend(self.menu_buttons());
        buttons
    }
    fn draw_nsf(&self) -> DrawList {
        let mut d = DrawList::new(BG);
        let Some(source) = &self.score.nsf else {
            return d;
        };
        let Ok(h) = source.validate() else {
            return d;
        };
        d.fill_rect(Rect::new(0.0, 0.0, self.size.width, 58.0), PANEL);
        self.text(&mut d, 20.0, 17.0, "FCSEQ", 1.7, ACCENT);
        self.text(
            &mut d,
            166.0,
            18.0,
            &format!(
                "{}{}",
                short(&self.score.name, 60),
                if self.dirty() { " *" } else { "" }
            ),
            1.0,
            TEXT,
        );
        self.text(
            &mut d,
            166.0,
            40.0,
            "NSF PLAYER / IMPORTED MUSIC",
            0.6,
            MUTED,
        );
        d.fill_rect(Rect::new(24.0, 130.0, self.size.width - 48.0, 555.0), PANEL);
        self.text(&mut d, 48.0, 154.0, "NSF MUSIC PLAYER", 1.4, ACCENT);
        for (i, (label, value)) in [
            ("TITLE", short(&h.title, 90)),
            ("ARTIST", short(&h.artist, 90)),
            ("COPYRIGHT", short(&h.copyright, 90)),
            ("SOUND", h.chip_names()),
            (
                "FORMAT",
                format!(
                    "NSF{} / NTSC / {} us per PLAY / {} bytes",
                    h.version,
                    h.period_us,
                    source.data.len()
                ),
            ),
            (
                "MEMORY",
                format!(
                    "{} / LOAD ${:04X} / INIT ${:04X} / PLAY ${:04X}",
                    if h.banked() { "4 KiB banks" } else { "linear" },
                    h.load,
                    h.init,
                    h.play
                ),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 276.0 + i as f32 * 42.0;
            self.text(&mut d, 48.0, y, label, 0.9, MUTED);
            self.text(&mut d, 180.0, y, &value, 1.0, TEXT);
        }
        let elapsed = self
            .started
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);
        let elapsed = if self.playing_loop && source.seconds > 0 {
            elapsed % source.seconds as f32
        } else {
            elapsed.min(source.seconds as f32)
        };
        let bar = Rect::new(48.0, 544.0, self.size.width - 96.0, 10.0);
        d.fill_rect(bar, LINE);
        d.fill_rect(
            Rect::new(
                bar.x,
                bar.y,
                bar.width * elapsed / source.seconds as f32,
                bar.height,
            ),
            ACCENT,
        );
        self.text(
            &mut d,
            48.0,
            570.0,
            &format!(
                "{:02}:{:02} / {:02}:{:02}   SPACE: play / stop",
                elapsed as u32 / 60,
                elapsed as u32 % 60,
                source.seconds / 60,
                source.seconds % 60
            ),
            0.9,
            TEXT,
        );
        self.text(
            &mut d,
            48.0,
            696.0,
            "Original NSF code and samples are embedded when saving a .fcq project.",
            0.95,
            TEXT,
        );
        self.text(&mut d,48.0,728.0,"NSF import preserves playback data. Note editing is available in new sequencer projects.",0.9,MUTED);
        for b in self.nsf_buttons() {
            self.draw_button(&mut d, &b);
        }
        self.draw_input(&mut d);
        d.fill_rect(
            Rect::new(0.0, self.size.height - 36.0, self.size.width, 36.0),
            PANEL,
        );
        self.text(
            &mut d,
            18.0,
            self.size.height - 24.0,
            &short(&self.status, 150),
            0.8,
            TEXT,
        );
        d
    }
    fn launch_render(&mut self, score: Score, part: usize, looped: bool) {
        if self.job.is_some() && self.cancel.is_none() {
            self.status = "A file export is already running".into();
            return;
        }
        self.stop();
        self.play_rows = score.sections[part].rows;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker = cancel.clone();
        let (tx, rx) = mpsc::channel();
        self.cancel = Some(cancel);
        self.job = Some(rx);
        self.status = "Rendering audio... Space or STOP cancels".into();
        std::thread::spawn(move || {
            let result = services::render(&score, part, Some(&worker))
                .map(|samples| JobOutput::Audio(samples, looped));
            let _ = tx.send(result);
        });
    }
    fn start_audio(&mut self, selected: bool) {
        if self.job.is_some() && self.cancel.is_none() {
            self.status = "A file export is already running".into();
            return;
        }
        let mut s = self.score.clone();
        let mut part = self.part;
        if selected {
            let clip = Clipboard::copy(&s, part, &self.selection);
            for t in &mut s.sections[part].tracks {
                t.notes.clear();
            }
            for (c, n) in clip.notes {
                s.sections[part].tracks[c].notes.push(n);
            }
            let end = s.sections[part]
                .tracks
                .iter()
                .flat_map(|t| &t.notes)
                .map(Note::end)
                .max()
                .unwrap_or(16);
            s.sections[part].rows = end.div_ceil(16) * 16;
            s.sections = vec![s.sections[part].clone()];
            part = 0;
        }
        let looped = self.score.loop_playback && !selected;
        let monitor = s.nsf.is_none().then(|| monitor::Monitor::new(s.clone(), part, self.part, looped));
        self.launch_render(s, part, looped);
        self.monitor = monitor;
    }
    fn play_sample(&mut self) {
        let Some(sample) = self.score.samples.get(self.sample).cloned() else {
            return;
        };
        let mut score = Score::empty();
        let length = ((sample.end - sample.start) as f64 / sample.sample_rate as f64 * score.bpm / 15.0)
            .ceil() as u16;
        score.samples.push(sample.clone());
        score.sections[0].rows = (length + 1).div_ceil(16) * 16;
        score.sections[0].tracks[4]
            .notes
            .push(Note::new(0, length.max(1), sample.rate));
        self.launch_render(score, 0, false);
    }
    fn start_job(&mut self, path: Option<PathBuf>, export: bool) {
        if self.job.is_some() && self.cancel.is_none() {
            self.status = "A file export is already running".into();
            return;
        }
        let Some(path) = path else {
            return;
        };
        self.stop();
        let score = self.score.clone();
        let part = self.part;
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        self.status = if export {
            "Exporting register data and driver..."
        } else {
            "Rendering WAV..."
        }
        .into();
        std::thread::spawn(move || {
            let result = if export {
                services::export(&score, &path).map(|_| ())
            } else {
                services::render_wav(&score, part, &path).map(|_| ())
            }
            .map(|_| JobOutput::Done(format!("Exported {}", path.display())));
            let _ = tx.send(result);
        });
    }
    fn poll(&mut self) {
        if let Some(result) = self.job.as_ref().and_then(|r| r.try_recv().ok()) {
            self.job = None;
            self.cancel = None;
            match result {
                Err(e) => { self.monitor = None; self.status = e; },
                Ok(JobOutput::Done(message)) => self.status = message,
                Ok(JobOutput::Audio(samples, looped)) => {
                    if self.audio.is_none() {
                        self.audio = OutputStream::try_default().ok();
                    }
                    let result = (|| {
                        let handle = &self
                            .audio
                            .as_ref()
                            .ok_or("No audio output device. WAV export is available.")?
                            .1;
                        let sink = Sink::try_new(handle).map_err(|_| "Audio device unavailable")?;
                        if let Some(monitor) = self.monitor.as_mut() {
                            sink.append(monitor.attach(samples));
                        } else {
                            let source = rodio::buffer::SamplesBuffer::new(1, 44100, samples);
                            if looped { sink.append(source.repeat_infinite()); }
                            else { sink.append(source); }
                        }
                        Ok::<_, &str>(sink)
                    })();
                    match result {
                        Ok(sink) => {
                            self.sink = Some(sink);
                            self.started = Some(Instant::now());
                            self.playing_loop = looped;
                            self.status = "Playing / Space to stop".into();
                        }
                        Err(e) => { self.monitor = None; self.status = e.into(); },
                    }
                }
            }
        }
        if let Some(result) = self.monitor.as_mut().and_then(|monitor| monitor.poll()) {
            self.status = match result {
                Ok(()) => "Playing / Space to stop".into(),
                Err(error) => error,
            };
        }
        if self.sink.as_ref().is_some_and(|s| s.empty()) {
            self.stop();
            self.status = "Playback finished".into();
        }
    }
    fn playback_row(&self, start: Instant) -> f64 {
        let mut seconds = start.elapsed().as_secs_f64();
        if self.playing_loop {
            let duration = self.monitor.as_ref().and_then(|m| m.duration_seconds())
                .unwrap_or(self.score.row_tick(self.play_rows as u32) as f64 / self.score.tick_rate());
            if duration > 0.0 { seconds %= duration; }
        }
        (seconds * self.score.bpm / 15.0).min(self.play_rows as f64)
    }
    fn modal_rect(&self) -> Rect {
        Rect::new(
            (self.size.width - 870.0) / 2.0,
            (self.size.height - 620.0) / 2.0,
            870.0,
            620.0,
        )
    }
    fn wave_rect(&self) -> Rect {
        let m = self.modal_rect();
        Rect::new(m.x + 28.0, m.y + 108.0, 814.0, 192.0)
    }
    fn modal_buttons(&self) -> Vec<Button> {
        let m = self.modal_rect();
        let mut b = vec![Button::new(
            Action::CloseModal,
            m.right() - 98.0,
            m.y + 18.0,
            76.0,
            "CLOSE",
            None,
        )];
        match self.modal {
            Some(0) => {
                for (a, x, y, w, label) in [
                    (
                        Action::ImportAudio,
                        28.0,
                        62.0,
                        148.0,
                        "IMPORT AUDIO".to_string(),
                    ),
                    (Action::SampleStep(-1), 190.0, 62.0, 30.0, "<".into()),
                    (Action::SampleStep(1), 226.0, 62.0, 30.0, ">".into()),
                    (
                        Action::Input(Field::SampleStart),
                        28.0,
                        330.0,
                        232.0,
                        "START (ms)".into(),
                    ),
                    (
                        Action::Input(Field::SampleEnd),
                        284.0,
                        330.0,
                        232.0,
                        "END (ms)".into(),
                    ),
                    (
                        Action::Input(Field::SampleGain),
                        540.0,
                        330.0,
                        232.0,
                        "GAIN".into(),
                    ),
                    (Action::SampleRate(-1), 28.0, 416.0, 40.0, "-".into()),
                    (Action::SampleRate(1), 224.0, 416.0, 40.0, "+".into()),
                    (Action::Normalize, 284.0, 416.0, 232.0, "NORMALIZE".into()),
                    (Action::Reverse, 540.0, 416.0, 232.0, "REVERSE".into()),
                    (
                        Action::SamplePlay,
                        28.0,
                        516.0,
                        166.0,
                        "AUDITION DPCM".into(),
                    ),
                    (
                        Action::ExportDpcm,
                        214.0,
                        516.0,
                        166.0,
                        "EXPORT .DMC".into(),
                    ),
                    (
                        Action::RemoveSample,
                        400.0,
                        516.0,
                        166.0,
                        "REMOVE SAMPLE".into(),
                    ),
                ] {
                    b.push(Button::new(a, m.x + x, m.y + y, w, label, None));
                }
            }
            Some(1) => b.extend(self.instrument_buttons()),
            _ => {}
        }
        b
    }
    fn draw_button(&self, d: &mut DrawList, b: &Button) {
        let enabled = self.enabled(b.action);
        let active = match b.action {
            Action::Menu(m) => self.menu == Some(m),
            Action::Mute(ch) => self.score.sections[self.part].tracks[ch].mute,
            Action::Solo(ch) => self.score.sections[self.part].tracks[ch].solo,
            Action::Loop => self.score.loop_playback,
            Action::Normalize => self
                .score
                .samples
                .get(self.sample)
                .is_some_and(|s| s.normalize),
            Action::Reverse => self
                .score
                .samples
                .get(self.sample)
                .is_some_and(|s| s.reverse),
            _ => false,
        };
        d.fill_rect(
            b.rect,
            if active {
                Color::rgb8(35, 76, 84)
            } else if self.pressed == Some(b.action) || b.rect.contains(self.hover) {
                Color::rgb8(42, 57, 74)
            } else {
                PANEL
            },
        );
        d.stroke_rect(b.rect, 1.0, if active { ACCENT } else { LINE });
        let text = if enabled { TEXT } else { MUTED.with_alpha(0.4) };
        if let Some(icon) = b.icon {
            draw_lucide_icon(
                d,
                Rect::new(b.rect.x + 8.0, b.rect.y + 7.0, 16.0, 16.0),
                icon,
                text,
            );
        }
        let label = self.language.translate(&b.label);
        let available = b.rect.width - if b.icon.is_some() { 39.0 } else { 16.0 };
        let base_scale=if label.is_ascii(){0.85}else{1.25};
        let scale = base_scale * (available / measure_text_width(&label, base_scale).max(1.0)).min(1.0);
        draw_text(
            d,
            b.rect.x + if b.icon.is_some() { 31.0 } else { 8.0 },
            b.rect.y + 9.0,
            &label,
            scale,
            text,
        );
    }
    fn draw_modal(&self, d: &mut DrawList) {
        let m = self.modal_rect();
        d.fill_rect(
            Rect::new(0.0, 0.0, self.size.width, self.size.height),
            BG.with_alpha(0.88),
        );
        d.fill_rect(m, PANEL);
        d.stroke_rect(m, 1.0, LINE);
        self.text(
            d,
            m.x + 28.0,
            m.y + 25.0,
            match self.modal {
                Some(0) => "DPCM SAMPLE STUDIO",
                Some(1) => "INSTRUMENT / WAVEFORM STUDIO",
                _ => "FCSEQ / QUICK HELP",
            },
            1.05,
            ACCENT,
        );
        match self.modal {
            Some(0) => {
                let r = self.wave_rect();
                d.fill_rect(r, BG);
                if let Some(s) = self.score.samples.get(self.sample) {
                    self.text(
                        d,
                        m.x + 280.0,
                        m.y + 73.0,
                        &format!(
                            "{:02} / {:02}  {}",
                            self.sample + 1,
                            self.score.samples.len(),
                            short(&s.name, 40)
                        ),
                        0.8,
                        TEXT,
                    );
                    for x in 0..r.width as usize {
                        let a = x * s.pcm.len() / r.width as usize;
                        let z = ((x + 1) * s.pcm.len() / r.width as usize)
                            .max(a + 1)
                            .min(s.pcm.len());
                        let chunk = &s.pcm[a..z];
                        let lo = *chunk.iter().min().unwrap_or(&0) as f32 / 32768.0;
                        let hi = *chunk.iter().max().unwrap_or(&0) as f32 / 32768.0;
                        d.fill_rect(
                            Rect::new(
                                r.x + x as f32,
                                r.y + r.height * (0.5 - hi * 0.46),
                                1.0,
                                ((hi - lo) * r.height * 0.46).max(1.0),
                            ),
                            ACCENT,
                        );
                    }
                    let x1 = r.x + s.start as f32 / s.pcm.len() as f32 * r.width;
                    let x2 = r.x + s.end as f32 / s.pcm.len() as f32 * r.width;
                    d.fill_rect(Rect::new(r.x, r.y, x1 - r.x, r.height), BG.with_alpha(0.75));
                    d.fill_rect(
                        Rect::new(x2, r.y, r.right() - x2, r.height),
                        BG.with_alpha(0.75),
                    );
                    d.fill_rect(Rect::new(x1, r.y, 2.0, r.height), TEXT);
                    d.fill_rect(Rect::new(x2, r.y, 2.0, r.height), TEXT);
                    for (x, value) in [
                        (
                            28.0,
                            format!("{:.1} ms", s.start as f64 * 1000.0 / s.sample_rate as f64),
                        ),
                        (
                            284.0,
                            format!("{:.1} ms", s.end as f64 * 1000.0 / s.sample_rate as f64),
                        ),
                        (540.0, format!("{:.2} x", s.gain)),
                    ] {
                        self.text(d, m.x + x, m.y + 378.0, &value, 1.0, TEXT);
                    }
                    self.text(
                        d,
                        m.x + 86.0,
                        m.y + 426.0,
                        &format!("RATE {:02}", s.rate),
                        0.85,
                        TEXT,
                    );
                    self.text(
                        d,
                        m.x + 28.0,
                        m.y + 474.0,
                        &format!(
                            "{} / 4081 BYTES   {:.0} Hz   SOURCE {:.2}s / {} Hz",
                            s.encoded_size(),
                            fcseq_native::samples::rate(s.rate),
                            s.pcm.len() as f64 / s.sample_rate as f64,
                            s.sample_rate
                        ),
                        0.76,
                        if s.encoded_size() > 4081 {
                            Color::rgb8(255, 130, 115)
                        } else {
                            ACCENT
                        },
                    );
                } else {
                    self.text(
                        d,
                        r.x + 24.0,
                        r.y + 72.0,
                        "Import WAV / FLAC / OGG / MP3 to create a DPCM instrument.",
                        0.86,
                        TEXT,
                    );
                }
                self.text(d,m.x+28.0,m.y+578.0,"Original PCM is retained. Audition plays the encoded DPCM through the 2A03 core.",0.72,MUTED);
            }
            Some(1) => self.draw_instrument(d),
            _ => {
                for(i,text)in["Double-click an empty lane: insert. Drag body: move. Drag right edge: resize.","Ctrl/Shift-click: toggle selection. Drag empty space: box select.","Ctrl+A: select lane. Ctrl+C / X / V: copy / cut / paste at cursor.","Ctrl+D: duplicate. Delete: remove selection. Ctrl+Z / Y: undo / redo.","Arrows: move / transpose. Shift+Up/Down: octave. Q: quantize. G: grid.","F: fit section. Home: start. +/-: zoom. Wheel: lanes; Shift+wheel: timeline.","Space: play/stop. LOOP repeats the selected section. AUDITION plays selection.","Ctrl+N / O / S / Shift+S: new / open / save / save as.","E: open/close Key Editor. Double-click a track header to edit it with a piano keyboard.","DPCM: import audio, trim/process, then double-click the DPCM lane.","C export includes a generic driver and mapper/DPCM placement instructions."].iter().enumerate(){self.text(d,m.x+28.0,m.y+88.0+i as f32*41.0,text,0.75,TEXT);}
            }
        }
        for b in self.modal_buttons() {
            self.draw_button(d, &b);
        }
    }
    fn draw_input(&self, d: &mut DrawList) {
        if let Some((field, text, _)) = &self.input {
            let w = 660.0;
            let x = (self.size.width - w) / 2.0;
            let y = self.size.height / 2.0 - 64.0;
            d.fill_rect(
                Rect::new(0.0, 0.0, self.size.width, self.size.height),
                BG.with_alpha(0.7),
            );
            d.fill_rect(Rect::new(x, y, w, 146.0), PANEL);
            d.stroke_rect(Rect::new(x, y, w, 146.0), 2.0, ACCENT);
            self.text(
                d,
                x + 22.0,
                y + 20.0,
                &format!("EDIT {:?}", field).to_uppercase(),
                0.8,
                ACCENT,
            );
            self.text(
                d,
                x + 22.0,
                y + 60.0,
                &format!("{}|", short(text, 64)),
                1.0,
                TEXT,
            );
            self.text(
                d,
                x + 22.0,
                y + 112.0,
                "Type a value. Enter: apply / Escape: cancel",
                0.75,
                MUTED,
            );
        }
    }
    fn paint_wave(&mut self, p: Point) {
        self.paint_instrument_wave(p);
    }
    fn scrollbar(&self) -> Rect {
        let g = self.grid();
        Rect::new(g.x, g.bottom() + 12.0, g.width, 16.0)
    }
    fn set_scroll(&mut self, p: Point) {
        let r = self.scrollbar();
        let rows = self.score.sections[self.part].rows;
        self.page = ((p.x - r.x) / r.width * rows as f32 - self.visible as f32 / 2.0)
            .round()
            .clamp(0.0, rows.saturating_sub(self.visible) as f32) as u16;
    }
}
impl PlitaProgram for Studio {
    fn minimum_window_size(&self) -> Option<Size> {
        Some(Size::new(1120.0, 780.0))
    }
    fn redraw_when_unchanged(&self) -> bool {
        false
    }
    fn close_requested(&mut self) -> bool {
        self.can_discard()
    }
    fn resize(&mut self, size: Size) -> UiResponse {
        self.size = size;
        self.clamp_view();
        UiResponse::redraw()
    }
    fn tick(&mut self) -> UiResponse {
        if let Some(n) = self.smoke_ticks.as_mut() {
            *n += 1;
            if *n == 16 {
                let out = std::env::var_os("FCSEQ_SMOKE_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| root().join("verification/ui"));
                let _ = std::fs::create_dir_all(&out);
                let _=std::fs::write(out.join("native_event_loop.json"),serde_json::to_vec_pretty(&serde_json::json!({"toolkit":"plita","ticks":16,"empty_startup":self.score.sections[0].tracks.iter().all(|t|t.notes.is_empty()),"imported_nsf":self.score.nsf.is_some(),"channels":self.score.expansion.channels(),"viewport":[self.size.width,self.size.height]})).unwrap());
                std::process::exit(0);
            }
        }
        let active = self.job.is_some() || self.started.is_some();
        self.poll();
        if let Some(start) = self.started {
            let row = self.playback_row(start) as u32;
            if self.modal.is_none() {
                self.page = (row.min(self.score.sections[self.part].rows as u32 - 1) as u16
                    / self.visible)
                    * self.visible;
                self.clamp_view();
            }
        }
        if self.dirty()
            && self.recovery_revision != self.revision
            && self.last_recovery.elapsed().as_secs() > 3
        {
            let out = root().join("out");
            let _ = std::fs::create_dir_all(&out);
            if self
                .score
                .save(&out.join(format!("recovery_{}.fcq", std::process::id())))
                .is_ok()
            {
                self.recovery_revision = self.revision;
            }
            self.last_recovery = Instant::now();
        }
        if active {
            UiResponse::redraw()
        } else {
            UiResponse::none()
        }
    }
    fn handle_event(&mut self, event: UiEvent) -> UiResponse {
        if self.input.is_some() {
            match event {
                UiEvent::KeyDown { key, ctrl, .. } => {
                    let key = logical_key(&key);
                    match key.as_str() {
                        "ENTER" | "RETURN" => self.finish_input(),
                        "ESCAPE" => self.input = None,
                        "BACKSPACE" => {
                            let (_, text, replace) = self.input.as_mut().unwrap();
                            if *replace {
                                text.clear();
                                *replace = false;
                            } else {
                                text.pop();
                            }
                        }
                        "A" if ctrl => self.input.as_mut().unwrap().2 = true,
                        _ => {}
                    }
                }
                UiEvent::TextInput { text } | UiEvent::ImeCommit { text } => {
                    let (_, value, replace) = self.input.as_mut().unwrap();
                    if *replace {
                        value.clear();
                        *replace = false;
                    }
                    value.extend(
                        text.chars()
                            .filter(|c| !c.is_control())
                            .take(120usize.saturating_sub(value.len())),
                    );
                }
                _ => {}
            }
            return UiResponse::redraw();
        }
        match event {
            UiEvent::PointerDown {
                position,
                button: PointerButton::Primary,
            } => {
                self.pressed = self.action_at(position);
                if self.menu.is_some() {
                    if self.pressed.is_none() {
                        self.menu = None;
                    }
                    return UiResponse::redraw();
                }
                if self.pressed.is_some() {
                    return UiResponse::redraw();
                }
                if self.score.nsf.is_some() {
                    return UiResponse::redraw();
                }
                if self.modal == Some(1)
                    && self.wave_editable()
                    && self.wave_rect().contains(position)
                {
                    self.wave_before = Some(self.score.clone());
                    self.wave_last = None;
                    self.paint_wave(position);
                    return UiResponse::redraw();
                }
                if self.modal.is_some() {
                    return UiResponse::redraw();
                }
                if self.key_editor && self.keyboard_rect().contains(position) {
                    self.audition_key(self.key_pitch(position.y));
                    return UiResponse::redraw();
                }
                if self.scrollbar().contains(position) {
                    self.scroll_drag = true;
                    self.set_scroll(position);
                    return UiResponse::redraw();
                }
                if let Some(id) = self.hit(position) {
                    self.track = id.0;
                    self.cursor = self.score.sections[self.part].tracks[id.0].notes[id.1].row;
                    if self.modifier {
                        if let Some(i) = self.selection.iter().position(|v| *v == id) {
                            self.selection.remove(i);
                        } else {
                            self.selection.push(id);
                        }
                        return UiResponse::redraw();
                    }
                    if !self.selection.contains(&id) {
                        self.selection = vec![id];
                    }
                    let r = self.note_rect(
                        id.0,
                        &self.score.sections[self.part].tracks[id.0].notes[id.1],
                    );
                    self.drag = Some(Drag {
                        origin: position,
                        before: self.score.clone(),
                        selection: self.selection.clone(),
                        ch: id.0,
                        resize: position.x > r.right() - 6.0,
                    });
                } else if self.grid().contains(position) {
                    self.selection.clear();
                    self.track = self.point_track(position);
                    self.cursor = self.point_row(position);
                    self.marquee = Some((position, position));
                } else if position.x < 178.0
                    && position.y >= self.grid().y
                    && position.y < self.grid().bottom()
                {
                    self.track = self.point_track(position);
                    self.selection.clear();
                } else if position.y < 58.0
                    && position.x > 160.0
                    && position.x < self.size.width - 260.0
                {
                    self.begin_input(Field::Name);
                }
            }
            UiEvent::PointerMove { position } => {
                self.hover = position;
                if self.wave_before.is_some() {
                    self.paint_wave(position);
                }
                if self.scroll_drag {
                    self.set_scroll(position);
                }
                if let Some((_, p)) = self.marquee.as_mut() {
                    *p = position;
                }
                if let Some(drag) = self.drag.clone() {
                    let raw = ((position.x - drag.origin.x) / self.step() / self.snap as f32)
                        .round() as i32
                        * self.snap as i32;
                    let mut candidate = drag.before.clone();
                    let edit = if drag.resize {
                        Edit::Resize(raw)
                    } else {
                        Edit::Move(raw)
                    };
                    let moved =
                        editing::edit_notes(&mut candidate, self.part, &drag.selection, edit);
                    if moved.is_ok() && !drag.resize {
                        let (lo, hi) = self.score.expansion.kind(drag.ch).bounds();
                        let delta = if self.key_editor {
                            (-(position.y - drag.origin.y) / self.key_height()).round() as i16
                        } else {
                            (-(position.y - drag.origin.y) / (self.lane(drag.ch).height - 38.0)
                                * (hi - lo) as f32)
                                .round() as i16
                        };
                        if let Err(e) = editing::edit_notes(
                            &mut candidate,
                            self.part,
                            &drag.selection,
                            Edit::Transpose(delta),
                        ) {
                            self.status = e;
                            return UiResponse::redraw();
                        }
                    }
                    match moved {
                        Ok(_) => self.score = candidate,
                        Err(e) => self.status = e,
                    }
                }
            }
            UiEvent::PointerUp { position, .. } => {
                self.scroll_drag = false;
                if let Some(d) = self.drag.take() {
                    self.remember(d.before);
                }
                if let Some(before) = self.wave_before.take() {
                    self.remember(before);
                }
                if let Some((a, z)) = self.marquee.take() {
                    if (z.x - a.x).abs() + (z.y - a.y).abs() > 6.0 {
                        let r = Rect::new(
                            a.x.min(z.x),
                            a.y.min(z.y),
                            (a.x - z.x).abs(),
                            (a.y - z.y).abs(),
                        );
                        for ch in self.first_track..self.first_track + self.lane_count() {
                            for (i, n) in self.score.sections[self.part].tracks[ch]
                                .notes
                                .iter()
                                .enumerate()
                            {
                                let nr = self.note_rect(ch, n);
                                if nr.x < r.right()
                                    && nr.right() > r.x
                                    && nr.y < r.bottom()
                                    && nr.bottom() > r.y
                                {
                                    self.selection.push((ch, i));
                                }
                            }
                        }
                    }
                }
                if let Some(a) = self.pressed.take() {
                    if self.action_at(position) == Some(a) {
                        self.apply(a);
                    }
                }
            }
            UiEvent::PointerDoubleClick {
                position,
                button: PointerButton::Primary,
            } => {
                if !self.key_editor
                    && self.score.nsf.is_none()
                    && self.modal.is_none()
                    && self.menu.is_none()
                    && position.x < 178.0
                    && position.y >= self.grid().y
                    && position.y < self.grid().bottom()
                {
                    self.open_key_track(self.point_track(position));
                    return UiResponse::redraw();
                }
                if self.score.nsf.is_none()
                    && self.modal.is_none()
                    && self.menu.is_none()
                    && self.hit(position).is_none()
                {
                    self.marquee = None;
                    self.add_at(position);
                }
            }
            UiEvent::PointerWheel {
                position,
                delta_x,
                delta_y,
            } => {
                if self.modal.is_none() && self.menu.is_none() && self.grid().contains(position) {
                    if self.shift || delta_x.abs() > delta_y.abs() {
                        let d = if delta_x != 0.0 { delta_x } else { delta_y };
                        self.page = (self.page as i32 - d.signum() as i32 * 16).max(0) as u16;
                        self.clamp_view();
                    } else if self.key_editor {
                        self.key_low =
                            (self.key_low as i16 + delta_y.signum() as i16 * 3).clamp(0, 127) as u8;
                        self.clamp_key_range();
                    } else {
                        self.first_track =
                            (self.first_track as i32 - delta_y.signum() as i32).max(0) as usize;
                        self.clamp_view();
                    }
                }
            }
            UiEvent::KeyDown {
                key, ctrl, shift, ..
            } => {
                self.modifier = ctrl || shift;
                self.shift = shift;
                let k = logical_key(&key);
                if k == "E" && !ctrl && self.modal.is_none() && self.score.nsf.is_none() {
                    self.apply(Action::KeyEditor);
                    return UiResponse::redraw();
                }
                if k == "ESCAPE" {
                    self.menu = None;
                    self.modal = None;
                    self.selection.clear();
                    return UiResponse::redraw();
                }
                if self.modal.is_some() {
                    if ctrl && k == "Z" {
                        self.undo(shift);
                    }
                    if ctrl && k == "Y" {
                        self.undo(true);
                    }
                    if k == " " || k == "SPACE" {
                        self.stop();
                    }
                    return UiResponse::redraw();
                }
                let action = if ctrl {
                    match k.as_str() {
                        "N" => Some(Action::New),
                        "O" => Some(Action::Open),
                        "S" => Some(if shift { Action::SaveAs } else { Action::Save }),
                        "Z" => Some(if shift { Action::Redo } else { Action::Undo }),
                        "Y" => Some(Action::Redo),
                        "C" => Some(Action::Copy),
                        "X" => Some(Action::Cut),
                        "V" => Some(Action::Paste),
                        "D" => Some(Action::Duplicate),
                        "A" => Some(Action::SelectAll),
                        _ => None,
                    }
                } else {
                    match k.as_str() {
                        " " | "SPACE" => Some(Action::Play),
                        "DELETE" | "BACKSPACE" => Some(Action::Delete),
                        "ARROWUP" | "UP" => Some(Action::Pitch(if shift { 12 } else { 1 })),
                        "ARROWDOWN" | "DOWN" => Some(Action::Pitch(if shift { -12 } else { -1 })),
                        "ARROWLEFT" | "LEFT" => Some(Action::Row(-(self.snap as i32))),
                        "ARROWRIGHT" | "RIGHT" => Some(Action::Row(self.snap as i32)),
                        "G" => Some(Action::Grid),
                        "Q" => Some(Action::Quantize),
                        "F" => Some(Action::Fit),
                        "HOME" => Some(Action::Home),
                        "+" | "=" => Some(Action::Zoom(1)),
                        "-" => Some(Action::Zoom(-1)),
                        "F1" => Some(Action::Help),
                        _ => None,
                    }
                };
                if let Some(a) = action {
                    if self.selection.is_empty() {
                        if let Action::Row(delta) = a {
                            self.cursor = (self.cursor as i32 + delta)
                                .clamp(0, self.score.sections[self.part].rows as i32 - 1)
                                as u16;
                            return UiResponse::redraw();
                        }
                    }
                    self.apply(a);
                }
            }
            UiEvent::KeyUp { ctrl, shift, .. } => {
                self.modifier = ctrl || shift;
                self.shift = shift;
            }
            UiEvent::FocusChanged { focused: false } => {
                if let Some(d) = self.drag.take() {
                    self.remember(d.before);
                }
                if let Some(before) = self.wave_before.take() {
                    self.remember(before);
                }
                self.modifier = false;
                self.shift = false;
                self.pressed = None;
                self.marquee = None;
                self.scroll_drag = false;
            }
            UiEvent::FileDropped { path } => {
                if path.extension().is_some_and(|e| {
                    e.eq_ignore_ascii_case("fcq")
                        || e.eq_ignore_ascii_case("nsf")
                        || e.eq_ignore_ascii_case("nsfe")
                }) {
                    self.open(path);
                } else {
                    match Sample::import(&path) {
                        Ok(sample) => {
                            let index = self.score.samples.len();
                            self.change(|s| {
                                s.samples.push(sample);
                                Ok(())
                            });
                            self.sample = index.min(self.score.samples.len().saturating_sub(1));
                            self.modal = Some(0);
                        }
                        Err(e) => self.status = e,
                    }
                }
            }
            _ => {}
        }
        UiResponse::redraw()
    }
    fn draw_list(&self) -> DrawList {
        if self.score.nsf.is_some() {
            return self.draw_nsf();
        }
        let mut d = DrawList::new(BG);
        d.fill_rect(Rect::new(0.0, 0.0, self.size.width, 58.0), PANEL);
        self.text(&mut d, 20.0, 17.0, "FCSEQ", 1.7, ACCENT);
        self.text(
            &mut d,
            166.0,
            18.0,
            &format!(
                "{}{}",
                short(&self.score.name, 48),
                if self.dirty() { " *" } else { "" }
            ),
            0.98,
            TEXT,
        );
        self.text(&mut d, 166.0, 40.0, "FAMICOM MUSIC STUDIO", 0.52, MUTED);
        let g = self.grid();
        let rx = self.rx();
        d.fill_rect(Rect::new(rx - 12.0, g.y, 260.0, g.height + 40.0), PANEL);
        if self.key_editor {
            self.draw_keyboard(&mut d);
        } else {
            for ch in self.first_track..self.first_track + self.lane_count() {
                let l = self.lane(ch);
                d.fill_rect(
                    Rect::new(16.0, l.y, 150.0, l.height - 2.0),
                    if ch == self.track {
                        Color::rgb8(31, 44, 58)
                    } else {
                        PANEL
                    },
                );
                self.text(
                    &mut d,
                    26.0,
                    l.y + 15.0,
                    &self.score.expansion.channel_name(ch),
                    0.95,
                    color(ch),
                );
                self.text(
                    &mut d,
                    26.0,
                    l.y + 37.0,
                    &format!(
                        "{} NOTES",
                        self.score.sections[self.part].tracks[ch].notes.len()
                    ),
                    0.6,
                    MUTED,
                );
                d.fill_rect(
                    l,
                    if ch % 2 == 0 {
                        Color::rgb8(16, 24, 34)
                    } else {
                        Color::rgb8(20, 29, 40)
                    },
                );
                for k in 1..4 {
                    d.fill_rect(
                        Rect::new(l.x, l.y + k as f32 * l.height / 4.0, l.width, 1.0),
                        Color::rgb8(28, 39, 51),
                    );
                }
            }
        }
        d.push_clip(g);
        let rows = self.score.sections[self.part].rows;
        let line_step = if self.visible > 256 {
            16
        } else {
            self.snap.max(1)
        };
        for j in (0..=self.visible).step_by(line_step as usize) {
            let row = self.page + j;
            let x = g.x + j as f32 * self.step();
            d.fill_rect(
                Rect::new(x, g.y, if row % 16 == 0 { 2.0 } else { 1.0 }, g.height),
                if row % 16 == 0 {
                    LINE
                } else {
                    Color::rgb8(29, 41, 54)
                },
            );
        }
        if self.page + self.visible > rows {
            let x = g.x + rows.saturating_sub(self.page) as f32 * self.step();
            d.fill_rect(
                Rect::new(x, g.y, g.right() - x, g.height),
                BG.with_alpha(0.75),
            );
        }
        for ch in self.first_track..self.first_track + self.lane_count() {
            for (i, n) in self.score.sections[self.part].tracks[ch]
                .notes
                .iter()
                .enumerate()
            {
                let r = self.note_rect(ch, n);
                if r.right() < g.x || r.x > g.right() {
                    continue;
                }
                let audible = self.score.sections[self.part].audible(ch);
                d.fill_rect(
                    r,
                    if audible {
                        color(ch)
                    } else {
                        color(ch).with_alpha(0.3)
                    },
                );
                if self.selection.contains(&(ch, i)) {
                    d.stroke_rect(r, 2.0, TEXT);
                }
                d.fill_rect(
                    Rect::new(r.right() - 3.0, r.y + 3.0, 2.0, r.height - 6.0),
                    BG.with_alpha(0.5),
                );
                if r.width > 32.0 {
                    self.text(
                        &mut d,
                        r.x + 3.0,
                        r.y + 3.0,
                        &self.score.expansion.pitch_name(n.note, ch),
                        0.5,
                        BG,
                    );
                }
            }
        }
        let row = if let Some(start) = self.started {
            self.playback_row(start) as f32
        } else {
            self.cursor as f32
        };
        let x = g.x + (row - self.page as f32) * self.step();
        d.fill_rect(
            Rect::new(x, g.y, 2.0, g.height),
            if self.started.is_some() {
                Color::rgb8(255, 126, 109)
            } else {
                ACCENT.with_alpha(0.5)
            },
        );
        if let Some((a, z)) = self.marquee {
            d.stroke_rect(
                Rect::new(
                    a.x.min(z.x),
                    a.y.min(z.y),
                    (a.x - z.x).abs(),
                    (a.y - z.y).abs(),
                ),
                1.0,
                TEXT,
            );
        }
        d.pop_clip();
        for j in (0..self.visible).step_by(if self.visible > 512 {
            128
        } else if self.visible > 128 {
            64
        } else {
            16
        }) {
            self.text(
                &mut d,
                g.x + j as f32 * self.step() + 3.0,
                155.0,
                &format!("{:03}", (self.page + j) / 16 + 1),
                0.64,
                MUTED,
            );
        }
        let scroll = self.scrollbar();
        d.fill_rect(scroll, PANEL);
        let total = rows.max(self.visible) as f32;
        d.fill_rect(
            Rect::new(
                scroll.x + self.page as f32 / total * scroll.width,
                scroll.y,
                self.visible as f32 / total * scroll.width,
                scroll.height,
            ),
            LINE,
        );
        self.text(&mut d, rx, 194.0, "NOTE INSPECTOR", 0.93, ACCENT);
        if let Some(&(ch, i)) = self.selection.first() {
            let n = &self.score.sections[self.part].tracks[ch].notes[i];
            let sound = n.sound(ch);
            let kind = self.score.expansion.kind(ch);
            self.text(
                &mut d,
                rx,
                226.0,
                &format!(
                    "{}  /  {} SELECTED",
                    self.score.expansion.pitch_name(n.note, ch),
                    self.selection.len()
                ),
                0.91,
                color(ch),
            );
            let mut values = vec![
                format!("{} {}", if ch == 4 { "RATE" } else { "PITCH" }, n.note),
                format!("LENGTH {}", n.length),
                format!("ROW {}", n.row),
            ];
            if !matches!(
                kind,
                Voice::Triangle | Voice::Dpcm | Voice::VtTriangle | Voice::Pcm
            ) {
                if kind != Voice::Saw {
                    values.push(format!("VOLUME {}", sound.volume));
                }
                values.push(format!("DECAY {} FR", sound.decay));
            }
            match kind {
                Voice::Pulse | Voice::VrcPulse | Voice::MmcPulse | Voice::VtPulse => {
                    values.push(format!("DUTY {}", sound.duty))
                }
                Voice::Fm | Voice::EpsmFm => values.push(format!("FM PATCH {}", sound.instrument)),
                Voice::N163 => values.push(format!("WAVE {}", sound.instrument + 1)),
                Voice::Saw => values.push(format!(
                    "SAW RATE {}",
                    sound.saw_rate.unwrap_or(sound.volume * 2)
                )),
                Voice::Dpcm | Voice::Pcm => values.push(format!("SAMPLE {}", sound.instrument + 1)),
                Voice::Noise | Voice::VtNoise => values.push(format!(
                    "PERIOD {}",
                    sound
                        .noise_period
                        .unwrap_or([13, 8, 2, 3][(n.note - 1) as usize])
                )),
                _ => {}
            }
            for (i, value) in values.iter().enumerate() {
                self.text(
                    &mut d,
                    rx + 42.0,
                    289.0 + i as f32 * 44.0,
                    value,
                    0.90,
                    TEXT,
                );
            }
            if kind == Voice::Dpcm {
                if let Some(sample) = self.score.samples.get(sound.instrument as usize) {
                    self.text(&mut d, rx, 494.0, &short(&sample.name, 24), 0.8, MUTED);
                }
            }
        } else {
            for (i, text) in [
                "SELECT OR ADD NOTES",
                "Double-click: add",
                "Drag: move / resize",
                "Ctrl-click: multi-select",
                "Box drag: select region",
                "Ctrl+A: select lane",
                "Ctrl+D: duplicate",
                "Q: quantize to grid",
                "F1: all shortcuts",
            ]
            .iter()
            .enumerate()
            {
                self.text(
                    &mut d,
                    rx,
                    242.0 + i as f32 * 33.0,
                    text,
                    0.90,
                    if i == 0 { TEXT } else { MUTED },
                );
            }
        }
        self.text(
            &mut d,
            rx,
            g.bottom() + 10.0,
            &format!(
                "NTSC {:.2} BPM / {} TICKS",
                self.score.bpm,
                if self.score.average_ticks_per_row().floor() == self.score.average_ticks_per_row().ceil() {
                    self.score.ticks_per_row().to_string()
                } else {
                    format!("{:.0}/{:.0}", self.score.average_ticks_per_row().floor(), self.score.average_ticks_per_row().ceil())
                }
            ),
            0.65,
            MUTED,
        );
        // Menus are drawn after the normal surface so they remain on top.
        for b in self.buttons().iter().take(
            self.buttons()
                .len()
                .saturating_sub(self.menu_buttons().len()),
        ) {
            self.draw_button(&mut d, b);
        }
        if self.menu.is_some() {
            let buttons = self.menu_buttons();
            if let (Some(first), Some(last)) = (buttons.first(), buttons.last()) {
                d.fill_rect(
                    Rect::new(
                        first.rect.x - 4.0,
                        first.rect.y - 4.0,
                        298.0,
                        last.rect.bottom() - first.rect.y + 8.0,
                    ),
                    PANEL,
                );
            }
            for b in buttons {
                self.draw_button(&mut d, &b);
            }
        }
        if self.modal.is_some() {
            self.draw_modal(&mut d);
        }
        self.draw_input(&mut d);
        d.fill_rect(
            Rect::new(0.0, self.size.height - 36.0, self.size.width, 36.0),
            PANEL,
        );
        self.text(
            &mut d,
            18.0,
            self.size.height - 24.0,
            &short(&self.status, 150),
            0.7,
            TEXT,
        );
        d
    }
}
pub fn smoke() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Studio::new(Size::new(1360.0, 900.0), None);
    assert!(app.path.is_none());
    assert!(app.score.sections[0]
        .tracks
        .iter()
        .all(|t| t.notes.is_empty()));
    for ch in 0..4 {
        for i in 0..8 {
            app.score.sections[0].tracks[ch].notes.push(Note::new(
                i * 8,
                if ch == 3 { 1 } else { 6 },
                if ch == 3 {
                    (i % 4 + 1) as u8
                } else {
                    [60, 64, 67, 72][ch] + (i % 3) as u8 * 2
                },
            ));
        }
    }
    app.score.samples.push(Sample {
        name: "DPCM WORKSHOP".into(),
        pcm: (0..8820)
            .map(|i| {
                ((i as f64 * std::f64::consts::TAU * 330.0 / 44100.0).sin()
                    * (1.0 - i as f64 / 8820.0)
                    * 24000.0) as i16
            })
            .collect(),
        sample_rate: 44100,
        start: 400,
        end: 7200,
        gain: 0.8,
        normalize: true,
        reverse: false,
        rate: 12,
        initial_level: 64,
    });
    for row in [0, 16, 32, 48] {
        app.score.sections[0].tracks[4]
            .notes
            .push(Note::new(row, 4, 12));
    }
    app.selection = vec![(0, 0), (0, 1)];
    let before = app.score.clone();
    app.edit(Edit::Transpose(1));
    app.undo(false);
    assert_eq!(app.score, before);
    app.selection = vec![(0, 0)];
    let out = std::env::var_os("FCSEQ_SMOKE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("verification/ui"));
    std::fs::create_dir_all(&out)?;
    app.score.save(&out.join("ui_fixture.fcq"))?;
    let base_score = app.score.clone();
    for (name, modal, menu) in [
        ("editor", None, None),
        ("instruments", Some(1), None),
        ("samples", Some(0), None),
        ("sound_menu", None, Some(3)),
        ("file_menu", None, Some(0)),
        ("n163_wave", Some(1), None),
        ("fds_wave", Some(1), None),
        ("vrc7_patch", Some(1), None),
        ("vrc6_pulse", Some(1), None),
        ("vrc6_saw", Some(1), None),
        ("triangle", Some(1), None),
        ("noise", Some(1), None),
        ("n163_small", Some(1), None),
        ("nsf_player", None, None),
        ("key_editor", None, None),
        ("key_editor_small", None, None),
        ("epsm_fm", Some(1), None),
        ("epsm_ssg", Some(1), None),
        ("key_editor_ja", None, None),
        ("fds_wave_ja", Some(1), None),
        ("epsm_fm_ja", Some(1), None),
        ("help_ja", Some(2), None),
    ] {
        app.score = base_score.clone();
        app.key_editor = false;
        app.language = if name.ends_with("_ja") {
            Language::Japanese
        } else {
            Language::English
        };
        app.size = Size::new(1360.0, 900.0);
        app.instrument_track = 0;
        app.selection = vec![(0, 0)];
        app.wave_slot = 0;
        let instrument = match name {
            "epsm_fm" | "epsm_fm_ja" => Some((Expansion::Epsm, 8)),
            "epsm_ssg" => Some((Expansion::Epsm, 5)),
            "n163_wave" | "n163_small" => Some((Expansion::N163, 5)),
            "fds_wave" | "fds_wave_ja" => Some((Expansion::Fds, 5)),
            "vrc7_patch" => Some((Expansion::Vrc7, 5)),
            "vrc6_pulse" => Some((Expansion::Vrc6, 5)),
            "vrc6_saw" => Some((Expansion::Vrc6, 7)),
            "triangle" => Some((Expansion::None, 2)),
            "noise" => Some((Expansion::None, 3)),
            _ => None,
        };
        if let Some((chip, ch)) = instrument {
            app.score.set_expansion(chip)?;
            app.instrument_track = ch;
            if ch >= 5 {
                let mut note = Note::new(0, 8, 60);
                note.sound = Some(Sound {
                    instrument: 0,
                    ..Sound::default()
                });
                app.score.sections[0].tracks[ch].notes.push(note);
            }
            app.selection = vec![(ch, 0)];
            if name == "n163_small" {
                app.size = Size::new(1120.0, 780.0);
            }
            app.score.save(&out.join(format!("{name}.fcq")))?;
        }
        if name.starts_with("key_editor") {
            app.open_key_track(0);
            if name.ends_with("small") {
                app.size = Size::new(1120.0, 780.0);
            }
        }
        if name == "nsf_player" {
            let mut second = app.score.sections[0].clone();
            second.name = "Second".into();
            app.score.sections.push(second);
            let data = nsf::export_bytes(&app.score)?;
            std::fs::write(out.join("ui_fixture.nsf"), &data)?;
            app.score = nsf::import_bytes(data)?;
            app.score.name = "NSF WORKSHOP".into();
            app.selection.clear();
            app.score.save(&out.join("imported_fixture.fcq"))?;
        }
        app.modal = modal;
        app.menu = menu;
        let draw = app.draw_list();
        let mut commands = vec![];
        for c in draw.commands() {
            if let DrawCommand::FillTriangles { vertices } = c {
                commands.push(serde_json::json!({"kind":"triangles","vertices":vertices.iter().map(|v|serde_json::json!([v.position.x,v.position.y,v.color.r,v.color.g,v.color.b,v.color.a])).collect::<Vec<_>>()}));
                continue;
            }
            match c{DrawCommand::FillRect{rect,color}=>commands.push(serde_json::json!({"kind":"rect","rect":[rect.x,rect.y,rect.width,rect.height],"color":[color.r,color.g,color.b,color.a]})),DrawCommand::ImageRgba{rect,width,height,pixels}=>commands.push(serde_json::json!({"kind":"image","rect":[rect.x,rect.y,rect.width,rect.height],"width":width,"height":height,"pixels":pixels.to_vec()})),DrawCommand::PushClip{rect}=>commands.push(serde_json::json!({"kind":"clip","rect":[rect.x,rect.y,rect.width,rect.height]})),DrawCommand::PopClip=>commands.push(serde_json::json!({"kind":"pop"})),_=>{}}
        }
        std::fs::write(
            out.join(format!("{name}_draw.json")),
            serde_json::to_vec(
                &serde_json::json!({"width":app.size.width,"height":app.size.height,"commands":commands}),
            )?,
        )?;
    }
    std::fs::write(
        out.join("ui_smoke.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"empty_startup":true,"edit_undo":true,"draw_screens":22}),
        )?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> Studio {
        Studio::new(Size::new(1360.0, 900.0), None)
    }
    #[test]
    fn n163_and_fds_wave_drawing_interpolates_and_undoes() {
        for chip in [Expansion::N163, Expansion::Fds] {
            let mut a = app();
            a.score.set_expansion(chip).unwrap();
            a.track = 5;
            a.apply(Action::Wave);
            let before = a.score.clone();
            let r = a.wave_rect();
            a.handle_event(UiEvent::PointerDown {
                position: Point::new(r.x + 1.0, r.bottom() - 1.0),
                button: PointerButton::Primary,
            });
            a.handle_event(UiEvent::PointerMove {
                position: Point::new(r.right() - 1.0, r.y + 1.0),
            });
            a.handle_event(UiEvent::PointerUp {
                position: Point::new(r.right() - 1.0, r.y + 1.0),
                button: PointerButton::Primary,
            });
            assert_ne!(a.score, before);
            a.score.validate().unwrap();
            let wave = if chip == Expansion::N163 {
                &a.score.n163_waves[a.wave_slot]
            } else {
                &a.score.fds_wave
            };
            assert!(wave.windows(2).all(|w| w[0] <= w[1]));
            key(&mut a, "Z", true, false);
            assert_eq!(a.score, before);
            key(&mut a, "Y", true, false);
            assert_ne!(a.score, before);
        }
    }
    #[test]
    fn instrument_controls_stay_inside_small_window_and_modify_selected_voice() {
        for chip in [
            Expansion::None,
            Expansion::Vrc6,
            Expansion::Vrc7,
            Expansion::Fds,
            Expansion::N163,
        ] {
            let mut a = Studio::new(Size::new(1120.0, 780.0), None);
            a.score.set_expansion(chip).unwrap();
            let ch = if chip == Expansion::None { 0 } else { 5 };
            a.score.sections[0].tracks[ch]
                .notes
                .push(Note::new(0, 4, 60));
            a.selection = vec![(ch, 0)];
            a.apply(Action::Wave);
            for b in a.modal_buttons() {
                assert!(
                    b.rect.x >= 0.0
                        && b.rect.y >= 0.0
                        && b.rect.right() <= 1120.0
                        && b.rect.bottom() <= 780.0
                );
            }
            if chip == Expansion::Vrc7 {
                let before = a.score.clone();
                a.apply(Action::FmParam(0, 1));
                assert_ne!(before, a.score);
            } else {
                a.apply(Action::Timbre(0, -1));
                assert_eq!(a.score.sections[0].tracks[ch].notes[0].sound(ch).volume, 11);
            }
            if chip == Expansion::N163 {
                let before = a.score.clone();
                a.apply(Action::WaveLength(4));
                assert_eq!(a.score, before);
                assert!(a.status.contains("N163"));
            }
            a.draw_list();
        }
    }
    fn key(a: &mut Studio, k: &str, ctrl: bool, shift: bool) {
        a.handle_event(UiEvent::KeyDown {
            key: k.into(),
            ctrl,
            shift,
            alt: false,
            meta: false,
        });
    }
    #[test]
    fn native_named_keys_reach_editor_shortcuts_and_modal_cancel() {
        let mut a = app();
        a.score.sections[0].tracks[0].notes.push(Note::new(0, 4, 60));
        a.selection = vec![(0, 0)];
        key(&mut a, "Named(ArrowUp)|Code(ArrowUp)", false, false);
        assert_eq!(a.score.sections[0].tracks[0].notes[0].note, 61);
        key(&mut a, "Named(ArrowRight)|Code(ArrowRight)", false, false);
        assert_eq!(a.score.sections[0].tracks[0].notes[0].row, 1);
        key(&mut a, "Named(F1)|Code(F1)", false, false);
        assert_eq!(a.modal, Some(2));
        key(&mut a, "Named(Escape)|Code(Escape)", false, false);
        assert!(a.modal.is_none());
        a.selection = vec![(0, 0)];
        key(&mut a, "Named(Delete)|Code(Delete)", false, false);
        assert!(a.score.sections[0].tracks[0].notes.is_empty());
        key(&mut a, "z|Code(KeyZ)", true, false);
        assert_eq!(a.score.sections[0].tracks[0].notes.len(), 1);
    }
    #[test]
    fn section_rename_cancels_native_escape_and_commits_native_enter() {
        for language in [Language::English, Language::Japanese] {
            for escape in ["Escape", "Named(Escape)|Code(Escape)"] {
                let mut a = app();
                a.language = language;
                a.score.sections[0].tracks[0].notes.push(Note::new(0, 4, 60));
                a.selection = vec![(0, 0)];
                a.saved = a.score.clone();
                let before = a.score.clone();
                // Open the same menu item and input overlay as a real click.
                a.menu = Some(2);
                let b = a.menu_buttons().into_iter().find(|b|
                    b.action == Action::Input(Field::SectionName)).unwrap();
                let p = Point::new(b.rect.x + 5.0, b.rect.y + 5.0);
                a.handle_event(UiEvent::PointerDown { position: p, button: PointerButton::Primary });
                a.handle_event(UiEvent::PointerUp { position: p, button: PointerButton::Primary });
                assert!(a.input.is_some());
                a.handle_event(UiEvent::ImeCommit { text: "変更後の名前".into() });
                key(&mut a, escape, false, false);
                assert!(a.input.is_none(), "{escape}");
                assert_eq!(a.score, before);
                assert_eq!(a.selection, vec![(0, 0)]);
                assert!(a.undo.is_empty());
                assert!(!a.dirty());
                a.begin_input(Field::SectionName);
                a.handle_event(UiEvent::TextInput { text: "New section".into() });
                key(&mut a, "Named(Backspace)|Code(Backspace)", false, false);
                key(&mut a, "Named(Enter)|Code(Enter)", false, false);
                assert!(a.input.is_none());
                assert_eq!(a.score.sections[0].name, "New sectio");
                a.undo(false);
                assert_eq!(a.score, before);
            }
        }
    }
    #[test]
    fn empty_startup_pointer_resize_and_history() {
        let mut a = app();
        assert!(a.path.is_none());
        assert!(!a.dirty());
        let p = Point::new(a.grid().x + 10.2 * a.step(), a.lane(0).y + 52.0);
        a.handle_event(UiEvent::PointerDoubleClick {
            position: p,
            button: PointerButton::Primary,
        });
        assert_eq!(a.selection, vec![(0, 0)]);
        let r = a.note_rect(0, &a.score.sections[0].tracks[0].notes[0]);
        let down = Point::new(r.right() - 1.0, r.y + 4.0);
        let up = Point::new(down.x + 2.0 * a.step(), down.y);
        a.handle_event(UiEvent::PointerDown {
            position: down,
            button: PointerButton::Primary,
        });
        a.handle_event(UiEvent::PointerMove { position: up });
        a.handle_event(UiEvent::PointerUp {
            position: up,
            button: PointerButton::Primary,
        });
        assert_eq!(a.score.sections[0].tracks[0].notes[0].length, 3);
        key(&mut a, "Z", true, false);
        assert_eq!(a.score.sections[0].tracks[0].notes[0].length, 1);
        key(&mut a, "Y", true, false);
        assert_eq!(a.score.sections[0].tracks[0].notes[0].length, 3);
    }
    #[test]
    fn clicking_or_rejected_edit_does_not_pollute_history() {
        let mut a = app();
        a.score.sections[0].tracks[0]
            .notes
            .push(Note::new(0, 2, 60));
        a.selection = vec![(0, 0)];
        a.edit(Edit::Move(-1));
        assert!(a.undo.is_empty());
        let r = a.note_rect(0, &a.score.sections[0].tracks[0].notes[0]);
        let p = Point::new(r.x + 3.0, r.y + 3.0);
        a.handle_event(UiEvent::PointerDown {
            position: p,
            button: PointerButton::Primary,
        });
        a.handle_event(UiEvent::PointerUp {
            position: p,
            button: PointerButton::Primary,
        });
        assert!(a.undo.is_empty());
    }
    #[test]
    fn keyboard_copy_paste_group_move_and_shortcuts() {
        let mut a = app();
        a.score.sections[0].tracks[0].notes = vec![Note::new(0, 2, 60), Note::new(4, 2, 64)];
        key(&mut a, "A", true, false);
        assert_eq!(a.selection.len(), 2);
        key(&mut a, "C", true, false);
        a.cursor = 12;
        key(&mut a, "V", true, false);
        assert_eq!(a.score.sections[0].tracks[0].notes.len(), 4);
        key(&mut a, "ArrowUp", false, true);
        assert_eq!(a.score.sections[0].tracks[0].notes[2].note, 72);
        key(&mut a, "G", false, false);
        assert_eq!(a.snap, 2);
        a.apply(Action::Fit);
        assert_eq!(a.visible, 64);
    }
    #[test]
    fn section_delete_undo_redo_and_view_stay_valid() {
        let mut a = app();
        a.apply(Action::AddSection);
        a.apply(Action::CloneSection);
        assert_eq!(a.part, 2);
        a.undo(false);
        assert!(a.part < a.score.sections.len());
        a.undo(true);
        assert!(a.part < a.score.sections.len());
        a.apply(Action::DeleteSection);
        assert!(a.part < a.score.sections.len());
        a.draw_list();
    }
    #[test]
    fn direct_inputs_are_transactional() {
        let mut a = app();
        a.score.sections[0].tracks[0]
            .notes
            .push(Note::new(63, 1, 60));
        let before = a.score.clone();
        a.input = Some((Field::Bars, "1".into(), false));
        a.finish_input();
        assert_eq!(a.score, before);
        assert!(a.undo.is_empty());
        a.input = Some((Field::Bpm, "120".into(), false));
        a.finish_input();
        assert_eq!(a.score.bpm, 120.0);
        a.undo(false);
        assert_eq!(a.score, before);
    }
    #[test]
    fn mute_solo_buttons_keep_playing_and_undo_tracks_the_mix() {
        for key_editor in [false, true] {
            let mut a = app();
            a.score.sections[0].rows = 16;
            for ch in 0..2 { a.score.sections[0].tracks[ch].notes.push(Note::new(0, 16, 60 + ch as u8 * 7)); }
            if key_editor { a.open_key_track(1); }
            let mut monitor = monitor::Monitor::new(a.score.clone(), 0, 0, true);
            let _source = monitor.attach(services::render(&a.score, 0, None).unwrap());
            a.monitor = Some(monitor);
            let started = Instant::now();
            a.started = Some(started);
            for action in [Action::Mute(1), Action::Solo(1)] {
                let button = a.buttons().into_iter().find(|b| b.action == action).unwrap();
                let p = Point::new(button.rect.x + 4.0, button.rect.y + 4.0);
                a.handle_event(UiEvent::PointerDown { position: p, button: PointerButton::Primary });
                a.handle_event(UiEvent::PointerUp { position: p, button: PointerButton::Primary });
                assert_eq!(a.started, Some(started));
                assert!(a.monitor.is_some());
            }
            assert!(a.score.sections[0].tracks[1].mute && a.score.sections[0].tracks[1].solo);
            assert!((0..5).any(|ch| a.score.sections[0].audible(ch)) == false);
            a.undo(false);
            assert!(!a.score.sections[0].tracks[1].solo);
            assert_eq!(a.started, Some(started));
            a.undo(false);
            assert!(!a.score.sections[0].tracks[1].mute);
            a.undo(true);
            assert!(a.score.sections[0].tracks[1].mute);
            assert_eq!(a.started, Some(started));
            a.open_key_track(0);
            let button = a.buttons().into_iter().find(|b| b.action == Action::Solo(0)).unwrap();
            let p = Point::new(button.rect.x + 4.0, button.rect.y + 4.0);
            a.handle_event(UiEvent::PointerDown { position: p, button: PointerButton::Primary });
            a.handle_event(UiEvent::PointerUp { position: p, button: PointerButton::Primary });
            assert!(a.score.sections[0].tracks[0].solo);
            assert!(!a.score.sections[0].tracks[1].solo);
            assert!(a.score.sections[0].tracks[1].mute);
            assert_eq!(a.started, Some(started));
            a.stop();
            assert!(a.monitor.is_none());
        }
    }
    #[test]
    fn selected_audition_keeps_mix_flags_and_pending_render_is_cancelled() {
        let mut a = app();
        a.score.sections[0].tracks[0].notes.push(Note::new(0, 4, 69));
        a.score.sections[0].tracks[0].mute = true;
        a.selection = vec![(0, 0)];
        a.start_audio(true);
        assert!(a.monitor.as_ref().unwrap().score.sections[0].tracks[0].mute);
        let old = a.cancel.as_ref().unwrap().clone();
        a.apply(Action::Mute(0));
        assert!(old.load(Ordering::Relaxed));
        assert!(!a.monitor.as_ref().unwrap().score.sections[0].tracks[0].mute);
        assert!(a.job.is_some());
        a.stop();
        assert!(a.monitor.is_none());
        let (tx, rx) = mpsc::channel();
        a.job = Some(rx);
        a.apply(Action::Solo(0));
        assert!(a.job.is_some());
        tx.send(Ok(JobOutput::Done("Export completed".into()))).unwrap();
        a.poll();
        assert_eq!(a.status, "Export completed");
    }
    #[test]
    fn cancelled_playback_cannot_start_when_old_worker_finishes() {
        let mut a = app();
        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        a.job = Some(rx);
        a.cancel = Some(cancel.clone());
        a.stop();
        assert!(cancel.load(Ordering::Relaxed));
        assert!(tx.send(Ok(JobOutput::Audio(vec![0; 4410], false))).is_err());
        a.poll();
        assert!(a.started.is_none());
        assert!(a.audio.is_none());
    }
    #[test]
    fn stopping_playback_preserves_export_completion() {
        let mut a = app();
        let (tx, rx) = mpsc::channel();
        a.job = Some(rx);
        a.stop();
        assert!(a.job.is_some());
        tx.send(Ok(JobOutput::Done("Export completed".into())))
            .unwrap();
        a.poll();
        assert_eq!(a.status, "Export completed");
        assert!(a.job.is_none());
    }
    #[test]
    fn nsf_player_selection_duration_and_note_protection() {
        let mut a = app();
        let mut score = Score::empty();
        score.sections.push(Section::empty("Second"));
        score.sections[0].tracks[0].notes.push(Note::new(0, 4, 69));
        a.score = nsf::import_bytes(nsf::export_bytes(&score).unwrap()).unwrap();
        a.saved = a.score.clone();
        a.apply(Action::NsfSong(1));
        assert_eq!(a.score.nsf.as_ref().unwrap().song, 1);
        assert!(!a.enabled(Action::NsfSong(1)));
        assert!(!a.enabled(Action::Paste));
        assert!(!a.enabled(Action::SetChip(Expansion::Vrc7)));
        a.input = Some((Field::NsfSeconds, "1".into(), false));
        a.finish_input();
        assert_eq!(a.score.nsf.as_ref().unwrap().seconds, 1);
        a.input = Some((Field::NsfSong, "0".into(), false));
        a.finish_input();
        assert_eq!(a.score.nsf.as_ref().unwrap().song, 1);
        a.undo(false);
        assert_eq!(a.score.nsf.as_ref().unwrap().seconds, 7);
        assert_eq!(a.score.nsf.as_ref().unwrap().duration_ms, Some(6406));
        let p = Point::new(200.0, 400.0);
        a.handle_event(UiEvent::PointerDoubleClick {
            position: p,
            button: PointerButton::Primary,
        });
        assert!(a.score.sections[0]
            .tracks
            .iter()
            .all(|t| t.notes.is_empty()));
        for b in a.nsf_buttons() {
            assert!(b.icon.is_none() || lucide_icon_names().any(|name| Some(name) == b.icon));
        }
        a.draw_list();
    }
    #[test]
    fn menu_buttons_have_real_icons_and_do_not_hit_when_disabled() {
        let mut a = app();
        for menu in 0..4 {
            a.menu = Some(menu);
            for b in a.menu_buttons() {
                assert!(
                    lucide_icon_names().any(|name| Some(name) == b.icon),
                    "{:?}",
                    b.icon
                );
            }
        }
        a.menu = Some(1);
        let b = a
            .menu_buttons()
            .into_iter()
            .find(|b| b.action == Action::Undo)
            .unwrap();
        assert!(a
            .action_at(Point::new(b.rect.x + 4.0, b.rect.y + 4.0))
            .is_none());
    }
}
