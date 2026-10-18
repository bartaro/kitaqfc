use fcseq_native::{model::*, nsf, samples::Sample, services};
use kurosaki_core::{
    nsf::{NsfFile, NsfPlayer},
    TraceConfig,
};
fn sample() -> Sample {
    Sample {
        name: "Original sine".into(),
        pcm: (0..11025)
            .map(|i| ((i as f64 * std::f64::consts::TAU * 440.0 / 44100.0).sin() * 20000.0) as i16)
            .collect(),
        sample_rate: 44100,
        start: 0,
        end: 11025,
        gain: 1.0,
        normalize: false,
        reverse: false,
        rate: 10,
        initial_level: 64,
    }
}
fn score(chip: Expansion, ch: usize) -> Score {
    let mut s = Score::empty();
    s.set_expansion(chip).unwrap();
    s.sections[0].rows = 16;
    let kind = chip.kind(ch);
    let pitch = match kind {
        Voice::VtNoise => 2,
        Voice::Pcm | Voice::Rhythm => 60,
        _ => 69,
    };
    let mut n = Note::new(0, 8, pitch);
    n.sound = Some(Sound {
        instrument: 0,
        ..Sound::default()
    });
    if kind == Voice::Pcm {
        s.samples.push(sample());
    }
    s.sections[0].tracks[ch].notes.push(n);
    s
}
fn rms(pcm: &[i16]) -> f64 {
    (pcm.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / pcm.len() as f64).sqrt()
}
#[test]
fn every_new_expansion_voice_reaches_preview_and_nsf() {
    for chip in [
        Expansion::Mmc5,
        Expansion::Sunsoft5b,
        Expansion::Epsm,
        Expansion::Vt02,
    ] {
        for ch in 5..chip.channels() {
            let s = score(chip, ch);
            let preview = services::render(&s, 0, None).unwrap();
            let bytes = nsf::export_bytes(&s).unwrap();
            let f = NsfFile::from_bytes(&bytes).unwrap();
            assert_eq!(f.header.expansion_bits, chip.nsf_bits());
            let mut p = NsfPlayer::new(&f, None, 44100, TraceConfig::none()).unwrap();
            let rendered = p.render_samples(44100).unwrap();
            let a = rms(&preview[500..10000]);
            let b = rms(&rendered[500..10000]);
            assert!(a > 12.0 && b > 12.0, "{chip:?}/{ch}: {a}, {b}");
            assert!(
                (20.0 * (a / b).log10()).abs() < 3.0,
                "{chip:?}/{ch}: preview={a} NSF={b}"
            );
            let mut muted = s.clone();
            muted.sections[0].tracks[ch].mute = true;
            let silence = services::render(&muted, 0, None).unwrap();
            assert!(rms(&silence[500..10000]) < 5.0, "{chip:?}/{ch} mute");
        }
    }
}
#[test]
fn ssg_noise_envelope_fm_parameters_and_pcm_processing_are_shared() {
    for chip in [Expansion::Sunsoft5b, Expansion::Epsm] {
        let mut s = score(chip, 5);
        let base = services::render(&s, 0, None).unwrap();
        let sound = s.sections[0].tracks[5].notes[0].sound.as_mut().unwrap();
        sound.tone = false;
        sound.noise = true;
        sound.envelope = true;
        sound.envelope_period = 900;
        sound.envelope_shape = 10;
        let changed = services::render(&s, 0, None).unwrap();
        assert!(base != changed, "{chip:?}: unchanged SSG sound");
        assert!(rms(&changed[1000..10000]) > 10.0);
        assert_eq!(
            s,
            Score::from_json(&serde_json::to_string(&s).unwrap()).unwrap()
        );
    }
    let mut s = score(Expansion::Epsm, 8);
    let base = services::render(&s, 0, None).unwrap();
    fcseq_native::instruments::set_epsm_parameter(&mut s.epsm_patches[0], 0, 5).unwrap();
    assert!(
        base != services::render(&s, 0, None).unwrap(),
        "unchanged EPSM FM sound"
    );
    let mut s = score(Expansion::Mmc5, 7);
    let base = services::render(&s, 0, None).unwrap();
    s.samples[0].gain = 0.0;
    let quiet = services::render(&s, 0, None).unwrap();
    assert!(rms(&base[1000..10000]) > 100.0);
    assert!(rms(&quiet[1000..10000]) < 5.0);
}
fn file(flags: u8, init: &[u8], play: &[u8], irq: &[u8]) -> Vec<u8> {
    let mut b = vec![0; 128 + 768];
    b[..8].copy_from_slice(b"NESM\x1a\x02\x01\x01");
    for (at, addr) in [(8, 0x8000u16), (10, 0x8000), (12, 0x8100), (110, 16639)] {
        b[at..at + 2].copy_from_slice(&addr.to_le_bytes());
    }
    b[124] = flags;
    b[128..128 + init.len()].copy_from_slice(init);
    b[384..384 + play.len()].copy_from_slice(play);
    b[640..640 + irq.len()].copy_from_slice(irq);
    b
}
#[test]
fn nsf2_timer_irq_suppressed_play_and_chunked_execution() {
    let init = [
        0xa9, 0x00, 0x8d, 0xfe, 0xff, 0xa9, 0x82, 0x8d, 0xff, 0xff, 0xa9, 0xff, 0x8d, 0x1b, 0x40,
        0xa9, 0x03, 0x8d, 0x1c, 0x40, 0xa9, 1, 0x8d, 0x1d, 0x40, 0x58, 0x60,
    ];
    let irq = [
        0x48, 0xad, 0x1d, 0x40, 0xe6, 0x00, 0xa5, 0x00, 0x29, 0x7f, 0x8d, 0x11, 0x40, 0x68, 0x40,
    ];
    let bytes = file(0x50, &init, &[0x02], &irq);
    let f = NsfFile::from_bytes(&bytes).unwrap();
    let mut p = NsfPlayer::new(&f, None, 44100, TraceConfig::none()).unwrap();
    let a = p.render_samples(22050).unwrap();
    assert!(rms(&a) > 100.0);
    assert_eq!(p.play_calls, 0);
    let mut p = NsfPlayer::new(&f, None, 44100, TraceConfig::none()).unwrap();
    let mut b = vec![];
    for _ in 0..5 {
        b.extend(p.render_samples(4410).unwrap());
    }
    assert_eq!(a, b);
    let nsfe = f.to_nsfe_bytes().unwrap();
    let f2 = NsfFile::from_bytes(&nsfe).unwrap();
    assert_eq!(f2.header.nsf2_flags, 0x50);
    assert_eq!(f2.to_nsf_bytes().unwrap()[5], 2);
}
#[test]
fn nsf2_background_init_is_interrupted_by_rts_play() {
    let init = [
        0x84, 0x00, 0xc0, 0x80, 0xf0, 0x06, 0xe6, 0x01, 0x4c, 0x06, 0x80, 0xea, 0x60,
    ];
    let play = [0xe6, 0x02, 0xa5, 0x02, 0x29, 0x7f, 0x8d, 0x11, 0x40, 0x60];
    let f = NsfFile::from_bytes(&file(0x20, &init, &play, &[])).unwrap();
    let mut p = NsfPlayer::new(&f, None, 44100, TraceConfig::none()).unwrap();
    let out = p.render_samples(22050).unwrap();
    assert!(rms(&out) > 5.0);
    assert!((28..=31).contains(&p.play_calls));
    let f = NsfFile::from_bytes(&file(0x60, &[0x60], &[0x02], &[])).unwrap();
    let mut p = NsfPlayer::new(&f, None, 44100, TraceConfig::none()).unwrap();
    assert_eq!(p.render_samples(4410).unwrap().len(), 4410);
    assert_eq!(p.play_calls, 0);
}

#[test]
fn nsf2_regular_play_matches_nsf1_and_irq_deadlines_do_not_repeat() {
    let s = score(Expansion::Epsm, 8);
    let a = NsfFile::from_bytes(&nsf::export_bytes(&s).unwrap()).unwrap();
    let b = NsfFile::from_bytes(&a.to_nsf2_bytes().unwrap()).unwrap();
    let mut p = NsfPlayer::new(&a, None, 44100, TraceConfig::none()).unwrap();
    let mut q = NsfPlayer::new(&b, None, 44100, TraceConfig::none()).unwrap();
    assert_eq!(p.render_samples(44100).unwrap(), q.render_samples(44100).unwrap());
    let f = NsfFile::from_bytes(&file(0x10, &[0x60], &[0x60], &[0x40])).unwrap();
    let mut p = NsfPlayer::new(&f, None, 44100, TraceConfig::none()).unwrap();
    p.render_samples(44100).unwrap();
    assert!((60..=61).contains(&p.play_calls), "repeated deadline: {}", p.play_calls);
}

#[test]
fn psg_pitch_is_a440_and_vt_raw_dma_reads_cpu_ram() {
    for (chip, ch) in [(Expansion::Sunsoft5b, 5), (Expansion::Epsm, 5), (Expansion::Epsm, 8)] {
        let mut s = score(chip, ch);
        s.sections[0].tracks[ch].notes[0]
            .sound
            .as_mut()
            .unwrap()
            .volume = 15;
        let pcm = services::render(&s, 0, None).unwrap();
        let window = &pcm[4410..13230];
        let mean = window.iter().map(|v| f64::from(*v)).sum::<f64>() / window.len() as f64;
        let edges = window
            .windows(2)
            .filter(|w| f64::from(w[0]) < mean && f64::from(w[1]) >= mean)
            .count();
        let frequency = edges as f64 * 44100.0 / window.len() as f64;
        assert!(
            (frequency - 440.0).abs() < 8.0,
            "{chip:?} frequency {frequency}"
        );
    }
    // Loop 16 bytes from internal CPU RAM through VT02's raw-DMA mode.
    let mut init = vec![];
    for i in 0..16u8 {
        init.extend([0xa9, if i & 1 == 0 { 32 } else { 224 }, 0x85, i]);
    }
    for (a, v) in [
        (0x4030u16, 0x3f),
        (0x4010, 0x4f),
        (0x4012, 0),
        (0x4013, 0),
        (0x4015, 0x10),
    ] {
        init.extend([0xa9, v, 0x8d, a as u8, (a >> 8) as u8]);
    }
    init.push(0x60);
    let mut bytes = file(0x40, &init, &[0x02], &[]);
    bytes[123] = 64;
    let f = NsfFile::from_bytes(&bytes).unwrap();
    let mut p = NsfPlayer::new(&f, None, 44100, TraceConfig::none()).unwrap();
    let raw = p.render_samples(4410).unwrap();
    assert!(rms(&raw) > 1000.0);
    // Removing second-APU output also mutes raw PCM.
    let at = bytes
        .windows(5)
        .position(|w| w == [0xa9, 0x3f, 0x8d, 0x30, 0x40])
        .unwrap();
    bytes[at + 1] = 0x37;
    let f = NsfFile::from_bytes(&bytes).unwrap();
    let mut p = NsfPlayer::new(&f, None, 44100, TraceConfig::none()).unwrap();
    assert!(rms(&p.render_samples(4410).unwrap()) < 5.0);
}
