//! NSF1 file I/O and a standalone, banked 6502 register-stream driver.
//! Format reference: https://www.nesdev.org/wiki/NSF
use crate::{model::*, services};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub const MAX_FILE: usize = kurosaki_core::nsf::MAX_NSF_BYTES;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub version: u8,
    pub nsf2_flags: u8,
    pub title: String,
    pub artist: String,
    pub copyright: String,
    pub songs: u8,
    pub starting_song: u8,
    pub load: u16,
    pub init: u16,
    pub play: u16,
    pub period_us: u16,
    pub banks: [u8; 8],
    pub chips: u8,
    pub tracks: Vec<kurosaki_core::nsf::NsfTrack>,
}
impl Header {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let h = kurosaki_core::nsf::NsfFile::from_bytes(data)
            .map_err(|e| e.to_string())?
            .header;
        Ok(Self {
            version: h.version,
            nsf2_flags: h.nsf2_flags,
            title: h.title,
            artist: h.artist,
            copyright: h.copyright,
            songs: h.songs,
            starting_song: h.starting_song,
            load: h.load_address,
            init: h.init_address,
            play: h.play_address,
            period_us: h.ntsc_period_us,
            banks: h.banks,
            chips: h.expansion_bits,
            tracks: h.tracks,
        })
    }
    pub fn banked(&self) -> bool {
        self.banks.iter().any(|v| *v != 0)
    }
    pub fn chip_names(&self) -> String {
        let mut names = vec!["2A03 / DPCM"];
        for (bit, name) in [
            (1, "VRC6"),
            (2, "VRC7"),
            (4, "FDS"),
            (8, "MMC5"),
            (16, "N163"),
            (32, "Sunsoft 5B"),
            (64, "VT02"),
            (128, "EPSM"),
        ] {
            if self.chips & bit != 0 {
                names.push(name);
            }
        }
        names.join(" + ")
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Imported {
    pub data: Vec<u8>,
    /// Zero-based selected subsong. NSF's header uses one-based song numbers.
    pub song: u8,
    pub seconds: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
}
impl Imported {
    pub fn select_song(&mut self, song: u8) -> Result<(), String> {
        let h = Header::parse(&self.data)?;
        if song >= h.songs {
            return Err("NSF song is out of range".into());
        }
        self.song = song;
        self.duration_ms = h.tracks[song as usize].time_ms.map(|ms| {
            ms.saturating_add(h.tracks[song as usize].fade_ms.unwrap_or(0))
                .min(600_000)
        });
        self.seconds = self
            .duration_ms
            .map_or(self.seconds, |ms| ms.div_ceil(1000).max(1));
        Ok(())
    }
    pub fn validate(&self) -> Result<Header, String> {
        let h = Header::parse(&self.data)?;
        if self.song >= h.songs
            || !(1..=600).contains(&self.seconds)
            || self.duration_ms.is_some_and(|ms| ms > 600_000)
        {
            return Err(
                "NSF song is out of range or playback length is outside 1..600 seconds".into(),
            );
        }
        Ok(h)
    }
}
pub fn import_bytes(data: Vec<u8>) -> Result<Score, String> {
    let h = Header::parse(&data)?;
    let mut s = Score::empty();
    s.format = "fcseq-3".into();
    s.name = if h.title.is_empty() {
        "Imported NSF".into()
    } else {
        h.title
    };
    let duration_ms = h
        .tracks
        .get(h.starting_song as usize - 1)
        .and_then(|t| {
            t.time_ms
                .map(|ms| ms.saturating_add(t.fade_ms.unwrap_or(0)))
        })
        .map(|ms| ms.min(600_000));
    s.nsf = Some(Imported {
        data,
        song: h.starting_song - 1,
        seconds: duration_ms.map_or(30, |ms| ms.div_ceil(1000).max(1)),
        duration_ms,
    });
    s.validate()?;
    Ok(s)
}
pub fn import(path: &Path) -> Result<Score, String> {
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_FILE as u64 {
        return Err("NSF file exceeds 1 MiB of music data".into());
    }
    import_bytes(std::fs::read(path).map_err(|e| e.to_string())?)
}
pub fn info(score: &Score) -> Result<serde_json::Value, String> {
    let source = score
        .nsf
        .as_ref()
        .ok_or("This project does not contain an imported NSF")?;
    let h = source.validate()?;
    Ok(
        serde_json::json!({"header":h,"selected_song":source.song as u16+1,"seconds":source.seconds,"duration_ms":source.duration_ms,"chips":h.chip_names()}),
    )
}

struct Asm {
    bytes: Vec<u8>,
    labels: BTreeMap<&'static str, u16>,
    fixes: Vec<(usize, &'static str, bool)>,
}
impl Asm {
    fn new() -> Self {
        Self {
            bytes: vec![],
            labels: BTreeMap::new(),
            fixes: vec![],
        }
    }
    fn at(&self) -> u16 {
        0x8000 + self.bytes.len() as u16
    }
    fn label(&mut self, label: &'static str) {
        self.labels.insert(label, self.at());
    }
    fn emit(&mut self, bytes: &[u8]) {
        self.bytes.extend(bytes);
    }
    fn jump(&mut self, opcode: u8, label: &'static str) {
        self.emit(&[opcode]);
        self.fixes.push((self.bytes.len(), label, false));
        self.emit(&[0, 0]);
    }
    fn branch(&mut self, opcode: u8, label: &'static str) {
        self.emit(&[opcode]);
        self.fixes.push((self.bytes.len(), label, true));
        self.emit(&[0]);
    }
    fn finish(mut self) -> Result<Vec<u8>, String> {
        for (pos, label, relative) in self.fixes {
            let addr = *self.labels.get(label).ok_or("Missing NSF driver label")?;
            if relative {
                let delta = addr as i32 - (0x8000 + pos as i32 + 1);
                self.bytes[pos] =
                    i8::try_from(delta).map_err(|_| "NSF driver branch too far")? as u8;
            } else {
                self.bytes[pos..pos + 2].copy_from_slice(&addr.to_le_bytes());
            }
        }
        Ok(self.bytes)
    }
}
// ZP: stream pointer 0/1, delay 2/3, count 4, write address 5/6, bank 8, running 9.
fn driver(starts: &[(u8, u16)], init: &[services::Write]) -> Result<(Vec<u8>, u16), String> {
    let mut a = Asm::new();
    a.label("init");
    a.emit(&[0x78, 0xaa]); // SEI; TAX (NSF host supplies song in A)
    for (table, zp) in [("table_lo", 0), ("table_hi", 1), ("table_bank", 8)] {
        a.jump(0xbd, table);
        a.emit(&[0x85, zp]);
    }
    a.emit(&[0xa5, 8, 0x8d, 0xf9, 0x5f, 0xa9, 1, 0x85, 9]);
    a.jump(0x20, "delay");
    // Initialize the chip before the host starts PLAY or records audio. A full
    // N163 RAM/wave upload plus notes exceeded the first PLAY's frame budget.
    if init.iter().any(|w| w.address == 0xf800) {
        a.emit(&[0xa9, 0x7f, 0x8d, 0, 0xf8, 0xa9, 0, 0x8d, 0, 0x48]);
    }
    for w in init {
        a.emit(&[0xa9, w.value, 0x8d, w.address as u8, (w.address >> 8) as u8]);
    }
    // Allow the primed DMC DAC/filter to settle before capture begins.
    a.emit(&[0xa2, services::DAC_SETTLE_LOOPS]);
    a.label("settle_outer");
    a.emit(&[0xa0, 0]);
    a.label("settle_inner");
    a.emit(&[0x88]);
    a.branch(0xd0, "settle_inner");
    a.emit(&[0xca]);
    a.branch(0xd0, "settle_outer");
    a.emit(&[0x60]);
    let play = a.at();
    a.label("play");
    a.emit(&[0xa5, 9]);
    a.branch(0xf0, "return");
    a.emit(&[0xa5, 2, 0x05, 3]);
    a.branch(0xd0, "decrement");
    a.label("record");
    a.jump(0x20, "read");
    a.emit(&[0x85, 4]);
    a.branch(0xf0, "next");
    a.label("writes");
    a.jump(0x20, "read");
    a.emit(&[0x85, 5]);
    a.jump(0x20, "read");
    a.emit(&[0x85, 6]);
    a.jump(0x20, "read");
    a.emit(&[0xa0, 0, 0x91, 5, 0xc6, 4]);
    a.branch(0xd0, "writes");
    a.label("next");
    a.jump(0x20, "delay");
    a.emit(&[0xa5, 9]);
    a.branch(0xf0, "return");
    a.emit(&[0xa5, 2, 0x05, 3]);
    a.branch(0xf0, "record");
    a.label("decrement");
    a.emit(&[0xa5, 2]);
    a.branch(0xd0, "low");
    a.emit(&[0xc6, 3]);
    a.label("low");
    a.emit(&[0xc6, 2]);
    a.label("return");
    a.emit(&[0x60]);
    a.label("delay");
    a.jump(0x20, "read");
    a.emit(&[0x85, 2]);
    a.jump(0x20, "read");
    a.emit(&[0x85, 3, 0xc9, 0xff]);
    a.branch(0xd0, "delay_end");
    a.emit(&[0xa5, 2, 0xc9, 0xff]);
    a.branch(0xd0, "delay_end");
    a.emit(&[0xa9, 0, 0x85, 9]);
    a.label("delay_end");
    a.emit(&[0x60]);
    a.label("read");
    a.emit(&[0xa0, 0, 0xb1, 0, 0xe6, 0]);
    a.branch(0xd0, "read_end");
    a.emit(&[0xe6, 1, 0x48, 0xa5, 1, 0xc9, 0xa0]);
    a.branch(0xd0, "read_restore");
    a.emit(&[0xa9, 0x90, 0x85, 1, 0xe6, 8, 0xa5, 8, 0x8d, 0xf9, 0x5f]);
    a.label("read_restore");
    a.emit(&[0x68]);
    a.label("read_end");
    a.emit(&[0x09, 0, 0x60]); // Restore N/Z for the byte returned in A.
    a.label("table_lo");
    for (_, p) in starts {
        a.emit(&[*p as u8]);
    }
    a.label("table_hi");
    for (_, p) in starts {
        a.emit(&[(*p >> 8) as u8]);
    }
    a.label("table_bank");
    for (b, _) in starts {
        a.emit(&[*b]);
    }
    Ok((a.finish()?, play))
}
fn text(header: &mut [u8], offset: usize, value: &str) {
    // NSF1 metadata is 31 bytes plus NUL. Do not cut a UTF-8 character in half.
    let mut end = value.len().min(31);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    header[offset..offset + end].copy_from_slice(&value.as_bytes()[..end]);
}
pub fn export_bytes(score: &Score) -> Result<Vec<u8>, String> {
    score.validate()?;
    if let Some(nsf) = &score.nsf {
        return Ok(nsf.data.clone());
    }
    let streams = (0..score.sections.len())
        .map(|i| services::compile(score, i))
        .collect::<Result<Vec<_>, _>>()?;
    let mut music = vec![];
    let mut starts = vec![];
    for s in &streams {
        starts.push((
            u8::try_from(1 + music.len() / 4096).map_err(|_| "NSF exceeds 256 banks")?,
            0x9000 + (music.len() % 4096) as u16,
        ));
        let mut musical = s.clone();
        musical.events[0].writes.drain(..s.init_writes);
        music.extend(services::encode_stream(&musical)?);
    }
    music.resize(music.len().div_ceil(4096) * 4096, 0);
    let sample_bank = 1 + music.len() / 4096;
    if sample_bank + 4 > 256 {
        return Err("NSF exceeds its 1 MiB bank capacity".into());
    }
    let init = &streams[0].events[0].writes[..streams[0].init_writes];
    let (mut code, play) = driver(&starts, init)?;
    if code.len() > 4096 {
        return Err("NSF initialization exceeds the driver bank".into());
    }
    code.resize(4096, 0);
    let mut samples = streams[0].dpcm.clone();
    samples.resize(16384, 0x55);
    let mut data = vec![0; 128];
    data[..5].copy_from_slice(b"NESM\x1a");
    data[5] = 1;
    data[6] = streams.len() as u8;
    data[7] = 1;
    data[8..10].copy_from_slice(&0x8000u16.to_le_bytes());
    data[10..12].copy_from_slice(&0x8000u16.to_le_bytes());
    data[12..14].copy_from_slice(&play.to_le_bytes());
    text(&mut data, 14, &score.name);
    text(&mut data, 46, "FCSEQ");
    let period = if score.tick_rate() == FPS {
        16639u16
    } else {
        250u16
    };
    data[110..112].copy_from_slice(&period.to_le_bytes());
    data[120..122].copy_from_slice(&19997u16.to_le_bytes());
    data[112..120].copy_from_slice(&[
        0,
        1,
        1,
        1,
        sample_bank as u8,
        (sample_bank + 1) as u8,
        (sample_bank + 2) as u8,
        (sample_bank + 3) as u8,
    ]);
    data[123] = score.expansion.nsf_bits();
    data.extend(code);
    data.extend(music);
    data.extend(samples);
    let mut file = kurosaki_core::nsf::NsfFile::from_bytes(&data).map_err(|e| e.to_string())?;
    for ((track, stream), section) in file
        .header
        .tracks
        .iter_mut()
        .zip(&streams)
        .zip(&score.sections)
    {
        track.name = section.name.clone();
        track.time_ms = Some((stream.frames as u64 * u64::from(period) / 1000) as u32);
        track.fade_ms = Some(0);
    }
    file.to_nsf_bytes().map_err(|e| e.to_string())
}
pub fn export(score: &Score, path: &Path) -> Result<serde_json::Value, String> {
    let original = export_bytes(score)?;
    let file = kurosaki_core::nsf::NsfFile::from_bytes(&original).map_err(|e| e.to_string())?;
    let nsfe = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("nsfe"));
    let nsf2 = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("nsf2"));
    let original_preserved = score.nsf.is_some()
        && original.starts_with(b"NSFE") == nsfe
        && (!nsf2 || file.header.version == 2);
    let data = if original_preserved {
        original
    } else if nsfe {
        file.to_nsfe_bytes().map_err(|e| e.to_string())?
    } else if nsf2 {
        file.to_nsf2_bytes().map_err(|e| e.to_string())?
    } else if original.starts_with(b"NSFE") {
        file.to_nsf_bytes().map_err(|e| e.to_string())?
    } else {
        original
    };
    let h = Header::parse(&data)?;
    std::fs::write(path, &data).map_err(|e| e.to_string())?;
    let companion = if !nsfe && score.nsf.is_none() {
        let companion = path.with_extension("nsfe");
        let companion_file =
            kurosaki_core::nsf::NsfFile::from_bytes(&data).map_err(|e| e.to_string())?;
        std::fs::write(
            &companion,
            companion_file.to_nsfe_bytes().map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Some(companion)
    } else {
        None
    };
    Ok(
        serde_json::json!({"format":if nsfe {"nsfe"} else if h.version==2{"nsf-2"}else{"nsf-1"},"songs":h.songs,"bytes":data.len(),"chips":h.chip_names(),"original_preserved":original_preserved,"tracks":h.tracks,"nsfe_companion":companion}),
    )
}
