//! One register-event path drives native KUROSAKI preview and portable C export.
use crate::{model::*, samples::Sample};
use kurosaki_core::{
    apu::ApuState,
    mapper::{FdsMapper, Mapper, NromMapper},
    mapper_scaffolds::VrcFamilyMapper,
    Mirroring, TraceConfig, TraceSink,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
pub const CPU: f64 = 1_789_773.0;
/// The DMC's held DAC level affects triangle/noise gain even without samples.
/// Use the same baseline in preview, C streams and the NSF INIT routine.
pub const INITIAL_DMC_DAC: u8 = 64;
pub const DAC_SETTLE_LOOPS: u8 = 64;
// LDX #loops; (LDY #0; 256 * DEY/BNE; DEX/BNE), excluding the final RTS.
const DAC_SETTLE_CYCLES: u64 = 1 + 1286 * DAC_SETTLE_LOOPS as u64;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Write {
    pub address: u16,
    pub value: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub frame: u32,
    pub writes: Vec<Write>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stream {
    #[serde(default = "default_tick_rate")]
    pub tick_rate: f64,
    pub expansion: Expansion,
    pub frames: u32,
    pub events: Vec<Frame>,
    pub dpcm: Vec<u8>,
    pub sample_offsets: Vec<u16>,
    /// Hardware/wave setup prefix of frame zero; NSF runs it in INIT.
    #[serde(default)]
    pub init_writes: usize,
}
fn default_tick_rate() -> f64 {
    FPS
}
fn emit(events: &mut BTreeMap<u32, Vec<Write>>, frame: u32, address: u16, value: u8) {
    events
        .entry(frame)
        .or_default()
        .push(Write { address, value });
}
fn fm(events: &mut BTreeMap<u32, Vec<Write>>, frame: u32, register: u8, value: u8) {
    emit(events, frame, 0x9010, register);
    emit(events, frame, 0x9030, value);
}
fn n163(events: &mut BTreeMap<u32, Vec<Write>>, frame: u32, register: u8, value: u8) {
    emit(events, frame, 0xf800, register);
    emit(events, frame, 0x4800, value);
}
fn n163_volume(ch: usize, count: u8, volume: u8) -> u8 {
    volume | if ch == 5 { (count - 1) << 4 } else { 0 }
}
fn hz(note: u8) -> f64 {
    440.0 * 2f64.powf((note as f64 - 69.0) / 12.0)
}
pub fn fm_pitch(note: u8) -> (u16, u8) {
    let freq = hz(note);
    for block in 0..8 {
        let n = (freq * 524288.0 / (49716.0 * 2f64.powi(block))).round();
        if n <= 511.0 {
            return (n as u16, block as u8);
        }
    }
    (511, 7)
}
fn sample_bank(
    samples: &[Sample],
    used: &std::collections::BTreeSet<usize>,
) -> Result<(Vec<u8>, Vec<u16>), String> {
    let mut bank = vec![];
    let mut offsets = vec![];
    for (i, sample) in samples.iter().enumerate() {
        if !used.contains(&i) {
            offsets.push(0);
            continue;
        }
        while bank.len() % 64 != 0 {
            bank.push(0x55);
        }
        offsets.push(bank.len() as u16);
        bank.extend(sample.encode()?);
        if bank.len() > 16320 {
            return Err("DPCM bank exceeds 16320 bytes. Trim samples or remove unused samples before export/playback.".into());
        }
    }
    Ok((bank, offsets))
}
pub fn compile(score: &Score, part: usize) -> Result<Stream, String> {
    if score.nsf.is_some() {
        return Err("Imported NSF retains its 6502 code; use Export NSF or WAV".into());
    }
    score.validate()?;
    let sec = score.sections.get(part).ok_or("Section out of range")?;
    let tick_rate = score.tick_rate();
    let frames = score.row_tick(sec.rows as u32);
    let used = score
        .sections
        .iter()
        .flat_map(|p| p.tracks[4].notes.iter())
        .map(|n| n.sound(4).instrument as usize)
        .collect();
    let (dpcm, offsets) = sample_bank(&score.samples, &used)?;
    if score.expansion == Expansion::Fds && dpcm.len() > 8192 {
        return Err("FDS DPCM RAM at $C000..$DFFF holds at most 8192 bytes".into());
    }
    let mut events = BTreeMap::new();
    for (a, v) in [
        (0x4015, 15),
        (0x4017, 0x40),
        (0x4001, 8),
        (0x4005, 8),
        (0x4000, 0x30),
        (0x4004, 0x30),
        (0x4008, 0x80),
        (0x400c, 0x30),
        (0x4011, INITIAL_DMC_DAC),
    ] {
        emit(&mut events, 0, a, v);
    }
    match score.expansion {
        Expansion::Vrc6 => {
            emit(&mut events, 0, 0x9003, 0);
            for a in [0x9002, 0xa002, 0xb002] {
                emit(&mut events, 0, a, 0);
            }
        }
        Expansion::Vrc7 => {
            for (i, v) in score.vrc7_patch.iter().enumerate() {
                fm(&mut events, 0, i as u8, *v);
            }
            for i in 0..6 {
                fm(&mut events, 0, 0x20 + i, 0);
            }
        }
        Expansion::Fds => {
            for (a, v) in [
                (0x4023, 2),
                (0x4080, 0x80),
                (0x4083, 0x80),
                (0x4084, 0x80),
                (0x4087, 0x80),
                (0x4089, 0x80),
            ] {
                emit(&mut events, 0, a, v);
            }
            for (i, v) in score.fds_wave.iter().enumerate() {
                emit(&mut events, 0, 0x4040 + i as u16, *v);
            }
            emit(&mut events, 0, 0x4089, 0);
        }
        Expansion::N163 => {
            emit(&mut events, 0, 0xe000, 0);
            emit(&mut events, 0, 0xf800, 0x80);
            for _ in 0..128 {
                emit(&mut events, 0, 0x4800, 0);
            }
            emit(&mut events, 0, 0xf800, 0x80);
            for wave in &score.n163_waves {
                for pair in wave.chunks_exact(2) {
                    emit(&mut events, 0, 0x4800, pair[0] | (pair[1] << 4));
                }
            }
            n163(&mut events, 0, 0x7f, (score.n163_channels - 1) << 4);
        }
        _ => {}
    }
    for write in crate::extra_audio::init(score) {
        emit(&mut events, 0, write.address, write.value);
    }
    let init_writes = events.get(&0).map_or(0, Vec::len);
    // Ends precede starts at the same frame, including unsorted input notes.
    for (ch, track) in sec.tracks.iter().enumerate() {
        if !sec.audible(ch) {
            continue;
        }
        for n in &track.notes {
            let end = score.row_tick(n.end() as u32);
            match score.expansion.kind(ch) {
                Voice::Pulse => emit(&mut events, end, 0x4000 + ch as u16 * 4, 0x30),
                Voice::Triangle => emit(&mut events, end, 0x4008, 0x80),
                Voice::Noise => emit(&mut events, end, 0x400c, 0x30),
                Voice::Dpcm => emit(&mut events, end, 0x4015, 15),
                Voice::VrcPulse | Voice::Saw => {
                    emit(&mut events, end, 0x9002 + (ch - 5) as u16 * 0x1000, 0)
                }
                Voice::Fm => {
                    let (n, block) = fm_pitch(n.note);
                    fm(
                        &mut events,
                        end,
                        0x20 + (ch - 5) as u8,
                        ((n >> 8) as u8) | (block << 1),
                    );
                }
                Voice::Fds => emit(&mut events, end, 0x4080, 0x80),
                Voice::N163 => n163(
                    &mut events,
                    end,
                    0x7f - (ch - 5) as u8 * 8,
                    n163_volume(ch, score.n163_channels, 0),
                ),
                _ => {
                    for w in crate::extra_audio::off(score, ch) {
                        emit(&mut events, end, w.address, w.value)
                    }
                }
            }
        }
    }
    for (ch, track) in sec.tracks.iter().enumerate() {
        if !sec.audible(ch) {
            continue;
        }
        let kind = score.expansion.kind(ch);
        for n in &track.notes {
            let start = score.row_tick(n.row as u32);
            let duration = score.row_tick(n.end() as u32) - start;
            let sound = n.sound(ch);
            let timer =
                (CPU / (if kind == Voice::Triangle {
                    32.0
                } else if kind == Voice::Saw {
                    14.0
                } else {
                    16.0
                }) / hz(n.note)
                    - 1.0)
                    .round() as u16;
            match kind {
                Voice::Pulse => {
                    let base = 0x4000 + ch as u16 * 4;
                    emit(
                        &mut events,
                        start,
                        base,
                        if score.expansion == Expansion::Vt02 {
                            crate::extra_audio::swap_vt_duty(
                                (sound.duty << 6) | 0x30 | sound.volume,
                            )
                        } else {
                            (sound.duty << 6) | 0x30 | sound.volume
                        },
                    );
                    emit(&mut events, start, base + 2, timer as u8);
                    emit(&mut events, start, base + 3, ((timer >> 8) as u8) | 8);
                }
                Voice::Triangle => {
                    emit(&mut events, start, 0x4008, 0xff);
                    emit(&mut events, start, 0x400a, timer as u8);
                    emit(&mut events, start, 0x400b, ((timer >> 8) as u8) | 8);
                }
                Voice::Noise => {
                    emit(&mut events, start, 0x400c, 0x30 | sound.volume);
                    emit(
                        &mut events,
                        start,
                        0x400e,
                        sound
                            .noise_period
                            .unwrap_or([13, 8, 2, 3][(n.note - 1) as usize])
                            | if sound.noise_short { 0x80 } else { 0 },
                    );
                    emit(&mut events, start, 0x400f, 8);
                }
                Voice::Dpcm => {
                    let i = sound.instrument as usize;
                    let sample = &score.samples[i];
                    let size = sample.encoded_size();
                    for (a, v) in [
                        (0x4015, 15),
                        (0x4010, n.note),
                        (0x4011, sample.initial_level),
                        (0x4012, (offsets[i] / 64) as u8),
                        (0x4013, ((size - 1) / 16) as u8),
                        (0x4015, 31),
                    ] {
                        emit(&mut events, start, a, v);
                    }
                }
                Voice::VrcPulse | Voice::Saw => {
                    let base = 0x9000 + (ch - 5) as u16 * 0x1000;
                    emit(
                        &mut events,
                        start,
                        base,
                        if kind == Voice::Saw {
                            sound.saw_rate.unwrap_or(sound.volume * 2)
                        } else {
                            (sound.duty << 4) | sound.volume
                        },
                    );
                    emit(&mut events, start, base + 1, timer as u8);
                    emit(&mut events, start, base + 2, 0x80 | ((timer >> 8) as u8));
                }
                Voice::Fm => {
                    let (num, block) = fm_pitch(n.note);
                    let c = (ch - 5) as u8;
                    fm(&mut events, start, 0x20 + c, 0);
                    fm(
                        &mut events,
                        start,
                        0x30 + c,
                        (sound.instrument << 4) | (15 - sound.volume),
                    );
                    fm(&mut events, start, 0x10 + c, num as u8);
                    fm(
                        &mut events,
                        start,
                        0x20 + c,
                        0x10 | ((num >> 8) as u8) | (block << 1),
                    );
                }
                Voice::Fds => {
                    let freq = (hz(n.note) * (65536.0 * 64.0) / CPU).round() as u16;
                    emit(
                        &mut events,
                        start,
                        0x4080,
                        0x80 | ((sound.volume as u16 * 32 / 15) as u8),
                    );
                    emit(&mut events, start, 0x4082, freq as u8);
                    emit(&mut events, start, 0x4083, 0x40 | ((freq >> 8) as u8));
                }
                Voice::N163 => {
                    let wave = &score.n163_waves[sound.instrument as usize];
                    let offset: usize = score
                        .n163_waves
                        .iter()
                        .take(sound.instrument as usize)
                        .map(Vec::len)
                        .sum();
                    let freq =
                        crate::instruments::n163_frequency(n.note, wave.len(), score.n163_channels);
                    let base = 0x78 - (ch - 5) as u8 * 8;
                    for (reg, value) in [
                        (0, freq as u8),
                        (1, 0),
                        (2, (freq >> 8) as u8),
                        (3, 0),
                        (4, ((256 - wave.len()) as u8 & 0xfc) | (freq >> 16) as u8),
                        (5, 0),
                        (6, offset as u8),
                        (7, n163_volume(ch, score.n163_channels, sound.volume)),
                    ] {
                        n163(&mut events, start, base + reg, value);
                    }
                }
                _ => {
                    for w in crate::extra_audio::on(score, ch, n, &sound) {
                        emit(&mut events, start, w.address, w.value)
                    }
                }
            }
            if kind == Voice::Pcm {
                let sample = &score.samples[sound.instrument as usize];
                let values =
                    sample.pcm8(tick_rate * 2f64.powf((60.0 - f64::from(n.note)) / 12.0))?;
                let addr = if score.expansion == Expansion::Mmc5 {
                    0x5011
                } else {
                    0x4031
                };
                let mut previous = 128;
                for (age, value) in values
                    .into_iter()
                    .take(duration as usize)
                    .enumerate()
                {
                    let v = (128 + (i32::from(value) - 128) * i32::from(sound.volume) / 15)
                        .clamp(1, 255) as u8;
                    if v != previous {
                        emit(&mut events, start + age as u32, addr, v);
                        previous = v;
                    }
                }
                let end = (start
                    + sample.pcm8_length(tick_rate * 2f64.powf((60.0 - f64::from(n.note)) / 12.0))
                        as u32)
                    .min(score.row_tick(n.end() as u32));
                emit(&mut events, end, addr, 128);
            }
            if sound.decay > 0
                && !matches!(
                    kind,
                    Voice::Triangle | Voice::Dpcm | Voice::VtTriangle | Voice::Pcm
                )
            {
                let end = duration;
                let mut previous = sound.volume;
                let decay = (f64::from(sound.decay) * tick_rate / FPS).round().max(1.0) as u32;
                for age in 1..end.min(decay + 1) {
                    let volume = (sound.volume as u32 * (decay - age.min(decay)) / decay) as u8;
                    if volume == previous {
                        continue;
                    }
                    previous = volume;
                    let f = start + age;
                    match kind {
                        Voice::Pulse => emit(
                            &mut events,
                            f,
                            0x4000 + ch as u16 * 4,
                            if score.expansion == Expansion::Vt02 {
                                crate::extra_audio::swap_vt_duty((sound.duty << 6) | 0x30 | volume)
                            } else {
                                (sound.duty << 6) | 0x30 | volume
                            },
                        ),
                        Voice::Noise => emit(&mut events, f, 0x400c, 0x30 | volume),
                        Voice::VrcPulse => emit(
                            &mut events,
                            f,
                            0x9000 + (ch - 5) as u16 * 0x1000,
                            (sound.duty << 4) | volume,
                        ),
                        Voice::Saw => emit(
                            &mut events,
                            f,
                            0xb000,
                            if sound.volume == 0 {
                                0
                            } else {
                                (sound.saw_rate.unwrap_or(sound.volume * 2) as u16 * volume as u16
                                    / sound.volume as u16) as u8
                            },
                        ),
                        Voice::Fm => fm(
                            &mut events,
                            f,
                            0x30 + (ch - 5) as u8,
                            (sound.instrument << 4) | (15 - volume),
                        ),
                        Voice::Fds => emit(
                            &mut events,
                            f,
                            0x4080,
                            0x80 | ((volume as u16 * 32 / 15) as u8),
                        ),
                        Voice::N163 => n163(
                            &mut events,
                            f,
                            0x7f - (ch - 5) as u8 * 8,
                            n163_volume(ch, score.n163_channels, volume),
                        ),
                        _ => {
                            for w in crate::extra_audio::volume(score, ch, &sound, volume) {
                                emit(&mut events, f, w.address, w.value)
                            }
                        }
                    }
                }
            }
        }
    }
    // Resolve the one shared SSG mixer from simultaneous notes, including
    // mute/solo and ends/starts on the same tick.
    if matches!(score.expansion, Expansion::Sunsoft5b | Expansion::Epsm) {
        let times: Vec<u32> = sec
            .tracks
            .iter()
            .skip(5)
            .take(3)
            .flat_map(|t| {
                t.notes
                    .iter()
                    .flat_map(|n| [score.row_tick(n.row as u32), score.row_tick(n.end() as u32)])
            })
            .collect();
        let mut previous = 0x3f;
        for time in times.into_iter().collect::<std::collections::BTreeSet<_>>() {
            let mut mask = 0x3f;
            for ch in 5..8 {
                if sec.audible(ch) {
                    if let Some(n) = sec.tracks[ch]
                        .notes
                        .iter()
                        .find(|n| score.row_tick(n.row as u32) <= time && time < score.row_tick(n.end() as u32))
                    {
                        let sound = n.sound(ch);
                        if sound.tone {
                            mask &= !(1 << (ch - 5));
                        }
                        if sound.noise {
                            mask &= !(1 << (ch - 2));
                        }
                    }
                }
            }
            if mask != previous {
                let (a, d) = if score.expansion == Expansion::Epsm {
                    (0x401c, 0x401d)
                } else {
                    (0xc000, 0xe000)
                };
                emit(&mut events, time, a, 7);
                emit(&mut events, time, d, mask);
                previous = mask;
            }
        }
    }
    events.entry(frames).or_default();
    Ok(Stream {
        tick_rate,
        expansion: score.expansion,
        frames,
        events: events
            .into_iter()
            .map(|(frame, writes)| Frame { frame, writes })
            .collect(),
        dpcm,
        sample_offsets: offsets,
        init_writes,
    })
}
pub fn render_stream(stream: &Stream, cancel: Option<&AtomicBool>) -> Result<Vec<i16>, String> {
    let mut apu = ApuState::default();
    apu.sample_rate = 44100;
    apu.vt02 = stream.expansion == Expansion::Vt02;
    apu.capture_full_audio = true;
    let mut mapper: Box<dyn Mapper> = match stream.expansion {
        Expansion::Vrc6 | Expansion::Vrc7 => Box::new(VrcFamilyMapper::new(
            if stream.expansion == Expansion::Vrc6 {
                24
            } else {
                85
            },
            vec![0; 32768],
            vec![0; 8192],
            Mirroring::Horizontal,
        )),
        Expansion::Fds => Box::new(FdsMapper::new(vec![0; 8192], vec![])),
        Expansion::N163 => Box::new(crate::n163::N163::default()),
        Expansion::Mmc5 | Expansion::Sunsoft5b | Expansion::Epsm | Expansion::Vt02 => Box::new(
            kurosaki_core::expansion_audio::ExpansionAudio::new(stream.expansion.nsf_bits()),
        ),
        _ => Box::new(NromMapper::new(
            vec![0; 32768],
            vec![0; 8192],
            Mirroring::Horizontal,
        )),
    };
    let mut sink = TraceSink::default();
    let cfg = TraceConfig::none();
    let init = stream
        .events
        .first()
        .filter(|e| e.frame == 0)
        .map(|e| &e.writes[..stream.init_writes.min(e.writes.len())])
        .unwrap_or(&[]);
    for w in init {
        apu.write_vt_register(w.address, w.value);
        if w.address <= 0x4017 {
            apu.write_register(w.address, w.value, 0, 0, cfg, &mut sink);
        } else {
            mapper.write_prg(w.address, w.value, 0, 0, cfg, &mut sink);
        }
    }
    // NSF runs hardware setup and lets its DAC/filter settle during INIT,
    // before capture. Do that here too, without starting notes or advancing
    // the score, so silent projects do not acquire a DC startup transient.
    if !init.is_empty() {
        let mut settled = 0;
        while settled < DAC_SETTLE_CYCLES {
            let step = (DAC_SETTLE_CYCLES - settled).min(16);
            apu.clock_cpu_cycles_with_timed_expansion(
                step,
                0,
                settled,
                cfg,
                &mut sink,
                |cycles, rate| {
                    mapper.clock_expansion_audio(cycles);
                    rate.map_or(0, |rate| mapper.expansion_audio_sample(rate))
                },
            );
            settled += step;
        }
        apu.sample_buffer.clear();
    }
    let mut cycle = 0u64;
    let mut event = 0usize;
    for frame in 0..=stream.frames {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err("Cancelled".into());
        }
        while event < stream.events.len() && stream.events[event].frame == frame {
            let skip = if event == 0 { init.len() } else { 0 };
            for w in stream.events[event].writes.iter().skip(skip) {
                apu.write_vt_register(w.address, w.value);
                if w.address <= 0x4017 {
                    apu.write_register(w.address, w.value, frame as u64, cycle, cfg, &mut sink);
                } else {
                    mapper.write_prg(w.address, w.value, frame as u64, cycle, cfg, &mut sink);
                }
            }
            event += 1;
        }
        if frame == stream.frames {
            break;
        }
        let end = ((frame as f64 + 1.0) * CPU / stream.tick_rate).round() as u64;
        // Small CPU slices preserve oscillator/DMC state at audio sample boundaries.
        while cycle < end {
            if let Some(addr) = apu.pending_dmc_fetch_addr() {
                let byte = stream
                    .dpcm
                    .get(addr.wrapping_sub(0xc000) as usize)
                    .copied()
                    .unwrap_or(0x55);
                apu.complete_dmc_sample_fetch(byte, frame as u64, cycle, cfg, &mut sink);
            }
            let step = (end - cycle).min(16);
            apu.clock_cpu_cycles_with_timed_expansion(
                step,
                frame as u64,
                cycle,
                cfg,
                &mut sink,
                |cycles, rate| {
                    mapper.clock_expansion_audio(cycles);
                    rate.map_or(0, |rate| mapper.expansion_audio_sample(rate))
                },
            );
            cycle += step;
        }
    }
    Ok(apu.sample_buffer)
}
pub fn render(score: &Score, part: usize, cancel: Option<&AtomicBool>) -> Result<Vec<i16>, String> {
    if let Some(nsf) = &score.nsf {
        return crate::nsf_player::render(nsf, cancel);
    }
    render_stream(&compile(score, part)?, cancel)
}
pub fn write_wav(samples: &[i16], path: &Path) -> Result<(), String> {
    let mut wav = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .map_err(|e| e.to_string())?;
    for s in samples {
        wav.write_sample(*s).map_err(|e| e.to_string())?;
    }
    wav.finalize().map_err(|e| e.to_string())
}
pub fn render_wav(score: &Score, part: usize, path: &Path) -> Result<serde_json::Value, String> {
    let samples = render(score, part, None)?;
    write_wav(&samples, path)?;
    let mut report = serde_json::json!({"source":"KUROSAKI core","expansion":score.expansion,"samples":samples.len(),"sample_rate":44100,"seconds":samples.len()as f64/44100.0});
    if score.nsf.is_some() {
        report["source"] = "KUROSAKI NSF CPU/APU".into();
        report["nsf"] = crate::nsf::info(score)?;
        report["expansion"] = report["nsf"]["chips"].clone();
    }
    Ok(report)
}
pub fn export(score: &Score, path: &Path) -> Result<serde_json::Value, String> {
    score.validate()?;
    let streams: Vec<_> = (0..score.sections.len())
        .map(|p| compile(score, p))
        .collect::<Result<_, _>>()?;
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let mut header=String::from("#ifndef FCSEQ_SCORE_H\n#define FCSEQ_SCORE_H\ntypedef unsigned char fcseq_u8;\ntypedef unsigned short fcseq_u16;\n/* Each record: delay lo, delay hi, write count, then address lo/hi and value.\n   FF FF FF terminates. Call at FCSEQ_TICK_RATE_HZ. */\n");
    header += &format!("#define FCSEQ_TICK_RATE_HZ {:.9}\n", score.tick_rate());
    let mut source=String::from("/* Generated by FCSEQ. See fcseq_export.json for mapper and DPCM placement. */\n#include \"fcseq_score.h\"\n");
    header += &format!(
        "#define FCSEQ_EXPANSION {}\n",
        match score.expansion {
            Expansion::None => 0,
            Expansion::Vrc6 => 6,
            Expansion::Vrc7 => 7,
            Expansion::Fds => 20,
            Expansion::N163 => 19,
            Expansion::Mmc5 => 5,
            Expansion::Sunsoft5b => 69,
            Expansion::Epsm => 0x80,
            Expansion::Vt02 => 256,
        }
    );
    let mut sizes = vec![];
    for (i, stream) in streams.iter().enumerate() {
        let bytes = encode_stream(stream)?;
        sizes.push(bytes.len());
        header += &format!(
            "extern const fcseq_u8 fcseq_section_{i}[{}];\n",
            bytes.len()
        );
        source += &format!("const fcseq_u8 fcseq_section_{i}[{}]={{\n", bytes.len());
        for chunk in bytes.chunks(24) {
            source += &chunk
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(",");
            source += ",\n";
        }
        source += "};\n";
        std::fs::write(path.join(format!("section_{i}.bin")), bytes).map_err(|e| e.to_string())?;
    }
    header+="void fcseq_start(const fcseq_u8 *data);\nvoid fcseq_tick(void);\nvoid fcseq_stop(void);\n#endif\n";
    std::fs::write(path.join("fcseq_score.h"), header).map_err(|e| e.to_string())?;
    std::fs::write(path.join("score.c"), source).map_err(|e| e.to_string())?;
    std::fs::write(path.join("fcseq_player.c"), include_str!("player.c.txt"))
        .map_err(|e| e.to_string())?;
    if let Some(s) = streams.first() {
        std::fs::write(path.join("dpcm_bank.bin"), &s.dpcm).map_err(|e| e.to_string())?;
    }
    for (i, sample) in score.samples.iter().enumerate() {
        if !score.sections.iter().any(|s| {
            s.tracks[4]
                .notes
                .iter()
                .any(|n| n.sound(4).instrument as usize == i)
        }) {
            continue;
        }
        std::fs::write(path.join(format!("sample_{i:02}.dmc")), sample.encode()?)
            .map_err(|e| e.to_string())?;
    }
    let report = serde_json::json!({"format":"fcseq-export-2","expansion":score.expansion,"channels":score.expansion.channels(),"tick_rate_hz":score.tick_rate(),"frames_per_row":score.frames_per_row(),"ticks_per_row":score.ticks_per_row(),"average_ticks_per_row":score.average_ticks_per_row(),"requested_bpm":score.bpm,"actual_bpm":score.actual_bpm(),"duration_seconds":score.duration_seconds(),"timing":"accumulated-row-deadlines","sections":score.sections.iter().enumerate().map(|(i,s)|serde_json::json!({"name":s.name,"rows":s.rows,"bytes":sizes[i]})).collect::<Vec<_>>(),"dpcm_offsets":streams[0].sample_offsets,"dpcm_base":49152,"dpcm_placement":"Map dpcm_bank.bin at CPU $C000, keep mapped during playback; reserve vectors $FFFA..$FFFF. FDS requires matching writable sample memory or a compatible playback environment.","mapper":match score.expansion{Expansion::None=>"any compatible 2A03 board",Expansion::Vrc6=>"VRC6 mapper 24 (mapper 26 needs A0/A1 remapping)",Expansion::Vrc7=>"VRC7 mapper 85",Expansion::Fds=>"FDS 2C33",Expansion::N163=>"Namco 163 mapper 19; enable audio and keep driver/data mapped",Expansion::Mmc5=>"MMC5 mapper 5",Expansion::Sunsoft5b=>"SUNSOFT 5B mapper 69",Expansion::Epsm=>"EPSM YMF288 on CPU $401C..$401F",Expansion::Vt02=>"VT02+ second APU"},"driver":"Call fcseq_start(section) then fcseq_tick at tick_rate_hz (4000 Hz for PCM, otherwise NTSC frame rate); stream/driver must stay CPU-visible. Large songs require host bank planning. Export is data + driver, not a game ROM."});
    std::fs::write(
        path.join("fcseq_export.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        path.join("register_events.json"),
        serde_json::to_vec(&streams).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    Ok(report)
}
pub fn encode_stream(stream: &Stream) -> Result<Vec<u8>, String> {
    let mut out = vec![];
    let mut prev = 0u32;
    for e in &stream.events {
        let mut delay = e.frame - prev;
        while delay > 65534 {
            out.extend([254, 255, 0]);
            delay -= 65534;
        }
        for (chunk_i, chunk) in e.writes.chunks(254).enumerate() {
            let d = if chunk_i == 0 { delay } else { 0 };
            out.extend([d as u8, (d >> 8) as u8, chunk.len() as u8]);
            for w in chunk {
                out.extend([w.address as u8, (w.address >> 8) as u8, w.value]);
            }
        }
        if e.writes.is_empty() {
            out.extend([delay as u8, (delay >> 8) as u8, 0]);
        }
        prev = e.frame;
    }
    out.extend([255, 255, 255]);
    Ok(out)
}
