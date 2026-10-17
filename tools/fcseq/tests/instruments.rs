use fcseq_native::{editing, instruments::*, model::*, n163::N163, nsf, nsf_player, services};
use kurosaki_core::{mapper::Mapper, TraceConfig, TraceSink};
fn wr(chip: &mut N163, a: u16, v: u8) {
    chip.write_prg(a, v, 0, 0, TraceConfig::none(), &mut TraceSink::default());
}
fn rd(chip: &mut N163, a: u16) -> u8 {
    chip.read_prg(a, 0, 0, TraceConfig::none(), &mut TraceSink::default())
}
fn score(count: u8, length: usize) -> Score {
    let mut s = Score::empty();
    s.set_expansion(Expansion::N163).unwrap();
    s.n163_channels = count;
    s.n163_waves = vec![wave_preset(length, 15, 0)];
    s.sections[0].rows = 16;
    let mut n = Note::new(0, 8, 69);
    n.sound = Some(Sound {
        instrument: 0,
        ..Sound::default()
    });
    s.sections[0].tracks[5].notes.push(n);
    s
}
fn frequency(pcm: &[i16]) -> f64 {
    let p = &pcm[4410..17640];
    let mean = p.iter().map(|&v| v as f64).sum::<f64>() / p.len() as f64;
    let window: Vec<f64> = p
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            (v as f64 - mean)
                * (0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / (p.len() - 1) as f64).cos())
        })
        .collect();
    // Goertzel spectrum avoids false autocorrelation peaks from the multiplex DAC.
    (200..900)
        .max_by(|&a, &b| {
            let power = |hz| {
                let c = 2.0 * (std::f64::consts::TAU * hz as f64 / 44100.0).cos();
                let (mut q1, mut q2) = (0.0, 0.0);
                for &v in &window {
                    let q = v + c * q1 - q2;
                    q2 = q1;
                    q1 = q;
                }
                q1 * q1 + q2 * q2 - c * q1 * q2
            };
            power(a).partial_cmp(&power(b)).unwrap()
        })
        .unwrap() as f64
}
#[test]
fn n163_ram_read_increment_wrap_and_audio_disable() {
    let mut chip = N163::default();
    wr(&mut chip, 0xf800, 0xff);
    wr(&mut chip, 0x4800, 0x71);
    wr(&mut chip, 0x4800, 0x98);
    wr(&mut chip, 0xf800, 0xff);
    assert_eq!(rd(&mut chip, 0x4800), 0x71);
    assert_eq!(rd(&mut chip, 0x4800), 0x98);
    let stream = services::compile(&score(1, 32), 0).unwrap();
    for e in stream.events.iter().filter(|e| e.frame == 0) {
        for w in &e.writes {
            wr(&mut chip, w.address, w.value);
        }
    }
    assert!((0..1000).any(|_| {
        chip.clock_expansion_audio(40);
        chip.expansion_audio_sample(44100).abs() > 50
    }));
    wr(&mut chip, 0xe000, 64);
    assert!((0..1000).all(|_| {
        chip.clock_expansion_audio(40);
        chip.expansion_audio_sample(44100) == 0
    }));
    wr(&mut chip, 0xe000, 0);
    assert!((0..1000).any(|_| {
        chip.clock_expansion_audio(40);
        chip.expansion_audio_sample(44100).abs() > 50
    }));
}
#[test]
fn n163_pitch_scales_with_active_channels_and_wave_length() {
    for count in [1, 2, 4, 8] {
        for len in [4, 16, 32, 64] {
            let s = score(count, len);
            let pcm = services::render(&s, 0, None).unwrap();
            assert!(
                (frequency(&pcm) - 440.0).abs() < 6.0,
                "{count}/{len}: {}",
                frequency(&pcm)
            );
            let data = nsf::export_bytes(&s).unwrap();
            assert_eq!(data[123], 16);
            let pcm = nsf_player::render(
                &nsf::Imported {
                    data,
                    song: 0,
                    duration_ms: None,
                    seconds: 1,
                },
                None,
            )
            .unwrap();
            assert!(
                (frequency(&pcm) - 440.0).abs() < 6.0,
                "NSF {count}/{len}: {}",
                frequency(&pcm)
            );
        }
    }
}

#[test]
fn n163_preview_and_nsf_share_kurosaki_timing_and_mixer_gain() {
    let s = score(4, 32);
    let data = nsf::export_bytes(&s).unwrap();
    let actual = nsf_player::render(
        &nsf::Imported {
            data: data.clone(),
            song: 0,
            duration_ms: None,
            seconds: 1,
        },
        None,
    )
    .unwrap();
    let file = kurosaki_core::nsf::NsfFile::from_bytes(&data).unwrap();
    let mut player =
        kurosaki_core::nsf::NsfPlayer::new(&file, Some(1), 44100, TraceConfig::none()).unwrap();
    let reference = player.render_samples(44100).unwrap();
    assert_eq!(
        actual, reference,
        "FCSEQ NSF and KUROSAKI must produce identical PCM"
    );
    let preview = services::render(&s, 0, None).unwrap();
    let rms = |pcm: &[i16]| {
        (pcm[6615..15435]
            .iter()
            .map(|&v| (v as f64).powi(2))
            .sum::<f64>()
            / 8820.0)
            .sqrt()
    };
    let difference_db = 20.0 * (rms(&preview) / rms(&reference)).log10();
    assert!(
        difference_db.abs() < 0.2,
        "preview/NSF gain differs by {difference_db} dB"
    );
}
#[test]
fn n163_wave_ram_pitch_and_channel_limits_are_atomic() {
    let mut s = score(8, 32);
    let before = s.clone();
    assert!(editing::transaction(&mut s, |s| {
        s.n163_waves[0] = vec![0; 132];
        Ok(())
    })
    .is_err());
    assert_eq!(s, before);
    assert!(editing::transaction(&mut s, |s| {
        s.n163_waves[0][0] = 16;
        Ok(())
    })
    .is_err());
    assert_eq!(s, before);
    assert!(editing::transaction(&mut s, |s| {
        s.sections[0].tracks[5].notes[0].note = 95;
        Ok(())
    })
    .is_err());
    assert_eq!(s, before);
    s.n163_channels = 1;
    s.sections[0].tracks[12].notes = s.sections[0].tracks[5].notes.clone();
    assert!(s.validate().is_err());
    s.sections[0].tracks[12].notes.clear();
    s.n163_waves[0] = vec![0; 240];
    s.sections[0].tracks[5].notes[0].note = 60;
    s.validate().unwrap();
}
#[test]
fn waveform_edit_and_all_fm_fields_survive_fcq_and_change_sound() {
    let mut s = score(8, 32);
    let old = services::render(&s, 0, None).unwrap();
    s.n163_waves[0] = wave_preset(32, 15, 2);
    let restored = Score::from_json(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(s, restored);
    assert_ne!(old, services::render(&restored, 0, None).unwrap());
    let mut patch = [0; 8];
    for p in FM_PARAMETERS {
        p.set(&mut patch, p.max).unwrap();
        assert_eq!(p.get(&patch), p.max);
    }
    assert_eq!(patch, [255, 255, 255, 223, 255, 255, 255, 255]);
    for p in FM_PARAMETERS {
        p.set(&mut patch, 0).unwrap();
    }
    assert_eq!(patch, [0; 8]);
    let mut fm = Score::empty();
    fm.set_expansion(Expansion::Vrc7).unwrap();
    fm.sections[0].rows = 16;
    let mut n = Note::new(0, 8, 60);
    n.sound = Some(Sound {
        instrument: 0,
        ..Sound::default()
    });
    fm.sections[0].tracks[5].notes.push(n);
    let a = services::render(&fm, 0, None).unwrap();
    FM_PARAMETERS[12].set(&mut fm.vrc7_patch, 4).unwrap();
    assert_ne!(a, services::render(&fm, 0, None).unwrap());
    let a = services::render(&fm, 0, None).unwrap();
    FM_PARAMETERS[18].set(&mut fm.vrc7_patch, 3).unwrap();
    let b = services::render(&fm, 0, None).unwrap();
    assert_ne!(
        a, b,
        "Carrier key level must affect preview as well as export"
    );
}
#[test]
fn vrc6_full_saw_rate_is_exported_and_audible() {
    let mut s = Score::empty();
    s.set_expansion(Expansion::Vrc6).unwrap();
    s.sections[0].rows = 16;
    let mut n = Note::new(0, 8, 60);
    n.sound = Some(Sound {
        saw_rate: Some(63),
        ..Sound::default()
    });
    s.sections[0].tracks[7].notes.push(n);
    assert!(services::compile(&s, 0).unwrap().events[0]
        .writes
        .iter()
        .any(|w| w.address == 0xb000 && w.value == 63));
    let a = services::render(&s, 0, None).unwrap();
    s.sections[0].tracks[7].notes[0]
        .sound
        .as_mut()
        .unwrap()
        .saw_rate = Some(16);
    assert_ne!(a, services::render(&s, 0, None).unwrap());
}
#[test]
fn independent_unbanked_n163_nsf_executes_ram_ports_and_sounds() {
    // Independent 6502 INIT: fill 16 bytes of wave RAM, configure a single
    // 32-sample voice at A4, leave PLAY as RTS. No FCSEQ stream or compiler.
    let mut code = vec![];
    let store = |code: &mut Vec<u8>, addr: u16, v: u8| {
        code.extend([0xa9, v, 0x8d, addr as u8, (addr >> 8) as u8])
    };
    store(&mut code, 0xf800, 0x80);
    for i in 0..16 {
        store(&mut code, 0x4800, if i < 8 { 0xff } else { 0 });
    }
    store(&mut code, 0xf800, 0xf8);
    for v in [0x35, 0, 0x1e, 0, 0xe0, 0, 0, 15] {
        store(&mut code, 0x4800, v);
    }
    code.push(0x60);
    let play = 0x8000 + code.len() as u16;
    code.push(0x60);
    let mut data = vec![0; 128];
    data[..5].copy_from_slice(b"NESM\x1a");
    data[5] = 1;
    data[6] = 1;
    data[7] = 1;
    data[8..10].copy_from_slice(&0x8000u16.to_le_bytes());
    data[10..12].copy_from_slice(&0x8000u16.to_le_bytes());
    data[12..14].copy_from_slice(&play.to_le_bytes());
    data[123] = 16;
    data.extend(code);
    let pcm = nsf_player::render(
        &nsf::Imported {
            data,
            song: 0,
            duration_ms: None,
            seconds: 1,
        },
        None,
    )
    .unwrap();
    assert!((frequency(&pcm) - 440.0).abs() < 6.0);
}
