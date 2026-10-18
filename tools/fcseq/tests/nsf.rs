use fcseq_native::{model::*, nsf, nsf_player, samples::Sample, services};
use std::sync::atomic::AtomicBool;

fn sine() -> Sample {
    Sample {
        name: "Synthetic".into(),
        pcm: (0..4410)
            .map(|i| ((i as f64 * 0.05).sin() * 24000.0) as i16)
            .collect(),
        sample_rate: 44100,
        start: 0,
        end: 4410,
        gain: 1.0,
        normalize: true,
        reverse: false,
        rate: 12,
        initial_level: 64,
    }
}
fn source(chip: Expansion) -> Score {
    let mut s = Score::empty();
    s.set_expansion(chip).unwrap();
    s.sections[0].rows = 16;
    s.samples.push(sine());
    for ch in 0..chip.channels() {
        s.sections[0].tracks[ch].notes.push(Note::new(
            0,
            4,
            if ch == 3 {
                2
            } else if ch == 4 {
                12
            } else {
                69
            },
        ));
    }
    s
}
fn render(data: Vec<u8>, song: u8, seconds: u32) -> Vec<i16> {
    nsf_player::render(
        &nsf::Imported {
            data,
            song,
            seconds,
            duration_ms: None,
        },
        None,
    )
    .unwrap()
}
fn peak(pcm: &[i16]) -> i32 {
    pcm.iter().map(|s| (*s as i32).abs()).max().unwrap_or(0)
}
fn rms(pcm: &[i16]) -> f64 {
    (pcm.iter().map(|s| (*s as f64).powi(2)).sum::<f64>() / pcm.len() as f64).sqrt()
}

#[test]
fn no_dpcm_preview_and_nsf_match_triangle_noise_and_expansion_balance() {
    // N163-only tests cannot detect the DMC DAC's effect on the stock TND
    // mixer. Exercise triangle/noise alone and with an expansion voice.
    for (chip, voices) in [
        (Expansion::None, vec![2]),
        (Expansion::None, vec![3]),
        (Expansion::N163, vec![5]),
        (Expansion::N163, vec![2, 3, 5]),
    ] {
        let mut s = Score::empty();
        s.set_expansion(chip).unwrap();
        s.sections[0].rows = 32;
        s.n163_channels = 4;
        for ch in voices {
            // Different pitches prevent oscillator phase differences between
            // event playback and the 6502 driver from cancelling a unison.
            let pitch = match ch {
                3 => 2,
                5 => 74,
                _ => 57,
            };
            let mut note = Note::new(0, 20, pitch);
            note.sound = Some(Sound {
                instrument: 0,
                noise_period: if ch == 3 { Some(8) } else { None },
                ..Sound::default()
            });
            s.sections[0].tracks[ch].notes.push(note);
        }
        let preview = services::render(&s, 0, None).unwrap();
        let exported = render(nsf::export_bytes(&s).unwrap(), 0, 2);
        let window = 13230..48510;
        assert!(rms(&preview[window.clone()]) > 100.0);
        assert!(rms(&exported[window.clone()]) > 100.0);
        let delta_db = 20.0 * (rms(&preview[window.clone()]) / rms(&exported[window])).log10();
        assert!(
            delta_db.abs() < 0.3,
            "{chip:?}: preview/NSF differ by {delta_db} dB"
        );
    }
}

#[test]
fn dac_setup_settles_before_capture_and_sample_levels_still_override_it() {
    let mut s = Score::empty();
    s.sections[0].rows = 16;
    let quiet = services::render(&s, 0, None).unwrap();
    assert!(
        peak(&quiet) < 5,
        "DAC initialization leaked into silent preview"
    );
    s.sections[0].tracks[2].notes.push(Note::new(8, 4, 57));
    let delayed = services::render(&s, 0, None).unwrap();
    assert!(
        peak(&delayed[..30000]) < 5,
        "DAC initialization leaked before first note"
    );
    s.samples.push(Sample {
        initial_level: 24,
        ..sine()
    });
    s.sections[0].tracks[4].notes.push(Note::new(8, 2, 12));
    let stream = services::compile(&s, 0).unwrap();
    assert!(stream.events[0].writes[..stream.init_writes]
        .iter()
        .any(|w| w.address == 0x4011 && w.value == services::INITIAL_DMC_DAC));
    assert!(stream
        .events
        .iter()
        .find(|e| e.frame == 48)
        .unwrap()
        .writes
        .iter()
        .any(|w| w.address == 0x4011 && w.value == 24));
}

#[test]
fn mixed_dpcm_n163_nsf_uses_the_same_pcm_as_kurosaki() {
    let data = nsf::export_bytes(&source(Expansion::N163)).unwrap();
    let actual = render(data.clone(), 0, 1);
    let file = kurosaki_core::nsf::NsfFile::from_bytes(&data).unwrap();
    let mut player = kurosaki_core::nsf::NsfPlayer::new(
        &file,
        Some(1),
        44100,
        kurosaki_core::TraceConfig::none(),
    )
    .unwrap();
    assert_eq!(actual, player.render_samples(44100).unwrap());
}

#[test]
fn exported_nsf_replays_every_voice_and_stops_cleanly() {
    for chip in [
        Expansion::None,
        Expansion::Vrc6,
        Expansion::Vrc7,
        Expansion::Fds,
        Expansion::N163,
    ] {
        for ch in 0..chip.channels() {
            let mut s = source(chip);
            for i in 0..chip.channels() {
                s.sections[0].tracks[i].mute = i != ch;
            }
            let data = nsf::export_bytes(&s).unwrap();
            let h = nsf::Header::parse(&data).unwrap();
            assert!(h.banked());
            assert_eq!(h.songs, 1);
            let pcm = render(data, 0, 1);
            assert_eq!(pcm.len(), 44100, "{chip:?}/{ch}");
            assert!(peak(&pcm[..15000]) > 80, "{chip:?}/{ch} silent");
            assert!(
                rms(&pcm[30000..]) < 2.0,
                "{chip:?}/{ch} did not stop: {}",
                rms(&pcm[30000..])
            );
        }
    }
}
#[test]
fn subsongs_use_independent_pointers_and_initialization() {
    let mut s = source(Expansion::None);
    s.sections.push(Section::empty("Silence"));
    s.sections[1].rows = 16;
    let data = nsf::export_bytes(&s).unwrap();
    assert_eq!(nsf::Header::parse(&data).unwrap().songs, 2);
    assert!(peak(&render(data.clone(), 0, 1)) > 80);
    assert!(peak(&render(data, 1, 1)) < 2);
}
#[test]
fn four_kib_stream_boundaries_and_delay_carries_are_playable() {
    let mut s = Score::empty();
    s.set_expansion(Expansion::Vrc6).unwrap();
    s.bpm = 300.0;
    s.sections[0].rows = 256;
    for ch in [0, 1, 5, 6, 7] {
        for row in 0..256 {
            s.sections[0].tracks[ch]
                .notes
                .push(Note::new(row, 1, 60 + (row % 12) as u8));
        }
    }
    assert!(
        services::encode_stream(&services::compile(&s, 0).unwrap())
            .unwrap()
            .len()
            > 4096
    );
    let pcm = render(nsf::export_bytes(&s).unwrap(), 0, 14);
    assert_eq!(pcm.len(), 14 * 44100);
    assert!(
        peak(&pcm[9 * 44100..11 * 44100]) > 80,
        "Bank transition lost the stream"
    );
    assert!(rms(&pcm[13 * 44100..]) < 2.0);
    let mut delayed = Score::empty();
    delayed.sections[0].rows = 64;
    delayed.sections[0].tracks[0]
        .notes
        .push(Note::new(48, 4, 69)); // 288-frame delay crosses low-byte borrow
    let pcm = render(nsf::export_bytes(&delayed).unwrap(), 0, 6);
    assert!(peak(&pcm[..4 * 44100]) < 2);
    assert!(peak(&pcm[5 * 44100..]) > 80);
}
#[test]
fn arbitrary_unbanked_nsf_executes_init_and_multiple_songs() {
    // Independent hand-written NSF, no FCSEQ driver/data layout.
    let mut d = vec![0; 128];
    d[..5].copy_from_slice(b"NESM\x1a");
    d[5] = 1;
    d[6] = 2;
    d[7] = 2;
    d[8..10].copy_from_slice(&0x8000u16.to_le_bytes());
    d[10..12].copy_from_slice(&0x8000u16.to_le_bytes());
    // INIT chooses the timer from the song index; PLAY is an RTS.
    let code = [
        0x48, 0xa9, 1, 0x8d, 0x15, 0x40, 0xa9, 0xbf, 0x8d, 0, 0x40, 0xa9, 8, 0x8d, 1, 0x40, 0x68,
        0x0a, 0x09, 0x40, 0x8d, 2, 0x40, 0xa9, 1, 0x8d, 3, 0x40, 0x60, 0x60,
    ];
    d[12..14].copy_from_slice(&(0x8000u16 + code.len() as u16 - 1).to_le_bytes());
    d.extend(code);
    let imported = nsf::import_bytes(d.clone()).unwrap();
    assert_eq!(imported.nsf.as_ref().unwrap().song, 1);
    let a = render(d.clone(), 0, 1);
    let b = render(d, 1, 1);
    assert!(peak(&a) > 80);
    assert_ne!(a, b);
}
#[test]
fn fcq_roundtrip_embeds_nsf_and_export_preserves_original_bytes() {
    let data = nsf::export_bytes(&source(Expansion::Fds)).unwrap();
    let mut s = nsf::import_bytes(data.clone()).unwrap();
    s.nsf.as_mut().unwrap().seconds = 1;
    let text = serde_json::to_string(&s).unwrap();
    let restored = Score::from_json(&text).unwrap();
    assert_eq!(nsf::export_bytes(&restored).unwrap(), data);
    assert_eq!(
        services::render(&s, 0, None).unwrap(),
        services::render(&restored, 0, None).unwrap()
    );
    assert!(services::compile(&s, 0).is_err());
    s.sections[0].tracks[0].notes.push(Note::new(0, 1, 60));
    assert!(s.validate().is_err());
}
#[test]
fn malformed_files_cancellation_and_runaway_code_return_errors() {
    for data in [vec![], vec![0; 128], b"NSFEjunk".to_vec()] {
        assert!(nsf::Header::parse(&data).is_err());
    }
    let mut data = nsf::export_bytes(&source(Expansion::None)).unwrap();
    for (offset, value) in [
        (5, 2),
        (6, 0),
        (7, 0),
        (7, 2),
        (122, 1),
        (123, 128),
        (123, 8),
        (123, 32),
    ] {
        let old = data[offset];
        data[offset] = value;
        assert!(nsf::Header::parse(&data).is_err(), "{offset}");
        data[offset] = old;
    }
    let cancelled = AtomicBool::new(true);
    let asset = nsf::Imported {
        data: data.clone(),
        song: 0,
        seconds: 1,
        duration_ms: None,
    };
    assert_eq!(
        nsf_player::render(&asset, Some(&cancelled)).unwrap_err(),
        "Cancelled"
    );
    data[128..131].copy_from_slice(&[0x4c, 0, 0x80]); // INIT never returns
    let asset = nsf::Imported {
        data,
        song: 0,
        seconds: 1,
        duration_ms: None,
    };
    assert!(nsf_player::render(&asset, None)
        .unwrap_err()
        .contains("did not return within"));
}

#[test]
fn exported_track_lengths_and_nsfe_playback_are_exact() {
    let mut score = source(Expansion::N163);
    score.sections[0].rows = 896; // 56 bars, about 90 seconds
    let data = nsf::export_bytes(&score).unwrap();
    let file = kurosaki_core::nsf::NsfFile::from_bytes(&data).unwrap();
    assert_eq!(file.header.duration_ms(1), Some(89451));
    assert_eq!(data[5], 1); // Optional metadata does not require NSF2.
    let nsfe = file.to_nsfe_bytes().unwrap();
    let restored = kurosaki_core::nsf::NsfFile::from_bytes(&nsfe).unwrap();
    assert_eq!(restored.header.duration_ms(1), Some(89451));
    assert_eq!(restored.header.expansion_bits, 16);
    let a = render(data, 0, 1);
    let b = render(nsfe.clone(), 0, 1);
    assert_eq!(a, b);
    assert_eq!(
        nsf::import_bytes(nsfe).unwrap().nsf.unwrap().duration_ms,
        Some(89451)
    );
}

#[test]
fn n163_wave_upload_runs_in_init_not_the_first_play_budget() {
    let score = source(Expansion::N163);
    let data = nsf::export_bytes(&score).unwrap();
    let file = kurosaki_core::nsf::NsfFile::from_bytes(&data).unwrap();
    let mut player = kurosaki_core::nsf::NsfPlayer::new(
        &file,
        None,
        44100,
        kurosaki_core::TraceConfig {
            apu: true,
            mapper: true,
            ..kurosaki_core::TraceConfig::none()
        },
    )
    .unwrap();
    assert!(
        player
            .trace
            .events
            .iter()
            .filter(|e| e.addr == Some(0x4800))
            .count()
            >= 192
    );
    let initial = player.trace.events.len();
    player.render_samples(700).unwrap();
    let writes: Vec<_> = player.trace.events[initial..]
        .iter()
        .filter(|e| {
            e.frame == 0
                && matches!(
                    e.kind.as_str(),
                    "apu.reg_write" | "mapper.expansion_audio_write" | "mapper.n163_write"
                )
        })
        .collect();
    assert!(!writes.is_empty());
    assert!(writes.last().unwrap().cpu_cycle - writes[0].cpu_cycle < 29780);
    assert!(writes.iter().filter(|e| e.addr == Some(0x4800)).count() < 100);
}
