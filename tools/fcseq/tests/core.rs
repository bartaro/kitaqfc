use fcseq_native::{
    editing::{edit_notes, Clipboard, Edit},
    model::*,
    samples::Sample,
    services,
};
fn tone() -> Sample {
    Sample {
        name: "Test sine".into(),
        pcm: (0..4410)
            .map(|i| ((i as f64 * std::f64::consts::TAU * 330.0 / 44100.0).sin() * 16000.0) as i16)
            .collect(),
        sample_rate: 44100,
        start: 0,
        end: 4410,
        gain: 1.0,
        normalize: true,
        reverse: false,
        rate: 15,
        initial_level: 64,
    }
}
fn score(chip: Expansion) -> Score {
    let mut s = Score::empty();
    s.set_expansion(chip).unwrap();
    s.sections[0].rows = 16;
    s
}
#[test]
fn legacy_four_channel_projects_migrate_without_note_changes() {
    let text = r#"{"format":"fcseq-1","name":"Legacy","bpm":150,"sections":[{"name":"A","rows":256,"tracks":[{"notes":[{"row":252,"length":4,"note":69}]},{"notes":[]},{"notes":[]},{"notes":[]}]}]}"#;
    let s = Score::from_json(text).unwrap();
    assert_eq!(s.sections[0].tracks.len(), 5);
    assert_eq!(s.sections[0].tracks[0].notes[0], Note::new(252, 4, 69));
    assert_eq!(s.format, "fcseq-2");
    let again = Score::from_json(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(s, again);
}
#[test]
fn group_edit_clipboard_and_failures_are_atomic() {
    let mut s = score(Expansion::Vrc6);
    s.sections[0].tracks[0].notes = vec![Note::new(0, 2, 60), Note::new(4, 2, 64)];
    let ids = vec![(0, 0), (0, 1)];
    edit_notes(&mut s, 0, &ids, Edit::Move(2)).unwrap();
    let before = s.clone();
    assert!(edit_notes(&mut s, 0, &[(0, 0)], Edit::Move(4)).is_err());
    assert_eq!(s, before);
    let copy = Clipboard::copy(&s, 0, &ids);
    let pasted = copy.paste(&mut s, 0, 10).unwrap();
    assert_eq!(pasted.len(), 2);
    assert_eq!(s.sections[0].tracks[0].notes[3].row, 14);
    let before = s.clone();
    assert!(edit_notes(&mut s, 0, &ids, Edit::Transpose(60)).is_err());
    assert_eq!(s, before);
}
#[test]
fn last_bar_max_sections_and_chip_switch_protection() {
    let mut s = score(Expansion::Vrc7);
    s.sections[0].rows = 4096;
    s.sections[0].tracks[10].notes.push(Note::new(4095, 1, 72));
    s.validate().unwrap();
    let before = s.clone();
    assert!(s.set_expansion(Expansion::Fds).is_err());
    assert_eq!(s, before);
    s.sections = vec![s.sections[0].clone(); 16];
    s.validate().unwrap();
    s.sections.push(s.sections[0].clone());
    assert!(s.validate().is_err());
}
#[test]
fn every_audio_voice_produces_non_silent_pcm_and_mute_is_silent() {
    for chip in [
        Expansion::None,
        Expansion::Vrc6,
        Expansion::Vrc7,
        Expansion::Fds,
    ] {
        for ch in 0..chip.channels() {
            let mut s = score(chip);
            s.samples.push(tone());
            let note = if ch == 3 {
                2
            } else if ch == 4 {
                15
            } else {
                69
            };
            s.sections[0].tracks[ch].notes.push(Note::new(0, 8, note));
            let pcm = services::render(&s, 0, None).unwrap();
            let peak = pcm.iter().map(|v| v.unsigned_abs()).max().unwrap();
            assert!(peak > 80, "{chip:?} ch {ch} was silent: {peak}");
            assert!((pcm.len() as f64 / 44100.0 - 16.0 * 6.0 / FPS).abs() < 0.001);
            s.sections[0].tracks[ch].mute = true;
            let quiet = services::render(&s, 0, None).unwrap();
            assert!(
                quiet.iter().all(|v| v.unsigned_abs() < 5),
                "{chip:?} ch {ch} mute leaked"
            );
        }
    }
}
#[test]
fn volume_patch_wave_and_sample_processing_change_pcm() {
    for chip in [
        Expansion::None,
        Expansion::Vrc6,
        Expansion::Vrc7,
        Expansion::Fds,
    ] {
        let ch = if chip == Expansion::None { 0 } else { 5 };
        let mut s = score(chip);
        let mut n = Note::new(0, 8, 69);
        n.sound = Some(Sound::default());
        s.sections[0].tracks[ch].notes.push(n);
        let a = services::render(&s, 0, None).unwrap();
        s.sections[0].tracks[ch].notes[0]
            .sound
            .as_mut()
            .unwrap()
            .volume = 3;
        let b = services::render(&s, 0, None).unwrap();
        assert_ne!(a, b, "Volume ignored: {chip:?}");
        if chip == Expansion::Fds {
            s.fds_wave = (0..64).map(|i| if i < 32 { 63 } else { 0 }).collect();
            assert_ne!(b, services::render(&s, 0, None).unwrap());
        }
        if chip == Expansion::Vrc7 {
            s.sections[0].tracks[5].notes[0]
                .sound
                .as_mut()
                .unwrap()
                .instrument = 8;
            assert_ne!(b, services::render(&s, 0, None).unwrap());
        }
    }
    let mut s = tone();
    let bytes = s.encode().unwrap();
    assert_eq!(bytes.len() % 16, 1);
    assert!(bytes.len() <= 4081);
    s.gain = 0.2;
    assert_ne!(bytes, s.encode().unwrap());
    s.gain = 1.0;
    s.reverse = true;
    assert_ne!(bytes, s.encode().unwrap());
    s.end = 4409;
    s.start = 2205;
    assert!(s.encode().unwrap().len() < bytes.len());
}
#[test]
fn dpcm_limits_bad_regions_and_missing_sample_are_reported() {
    let mut s = tone();
    s.pcm.resize(44100 * 3, 0);
    s.end = s.pcm.len() as u32;
    assert!(s.encode().unwrap_err().contains("4081"));
    s.start = s.end;
    assert!(s.validate().is_err());
    let mut p = score(Expansion::None);
    p.sections[0].tracks[4].notes.push(Note::new(0, 2, 15));
    assert!(p.validate().unwrap_err().contains("missing sample"));
}
#[test]
fn exported_stream_preserves_exact_event_order_and_timing() {
    let mut s = score(Expansion::Vrc7);
    s.samples.push(tone());
    s.sections[0].tracks[0].notes = vec![Note::new(4, 4, 72), Note::new(0, 4, 60)];
    s.sections[0].tracks[5].notes.push(Note::new(0, 8, 69));
    s.sections[0].tracks[4].notes.push(Note::new(0, 4, 15));
    let stream = services::compile(&s, 0).unwrap();
    let bytes = services::encode_stream(&stream).unwrap();
    let mut p = 0;
    let mut frame = 0;
    let mut events = vec![];
    loop {
        let delay = u16::from_le_bytes([bytes[p], bytes[p + 1]]);
        let count = bytes[p + 2];
        p += 3;
        if delay == 65535 && count == 255 {
            break;
        }
        frame += delay as u32;
        let mut writes = vec![];
        for _ in 0..count {
            writes.push(services::Write {
                address: u16::from_le_bytes([bytes[p], bytes[p + 1]]),
                value: bytes[p + 2],
            });
            p += 3;
        }
        events.push(services::Frame { frame, writes });
    }
    assert_eq!(events, stream.events);
    let mut replay = stream.clone();
    replay.events = events;
    assert_eq!(
        services::render_stream(&stream, None).unwrap(),
        services::render_stream(&replay, None).unwrap()
    );
    let writes = &stream.events.iter().find(|e| e.frame == 24).unwrap().writes;
    let pulse: Vec<_> = writes
        .iter()
        .filter(|w| w.address == 0x4000)
        .map(|w| w.value)
        .collect();
    assert_eq!(pulse, vec![0x30, 0xbc]);
}
#[test]
fn solo_and_cancellation_are_honored() {
    let mut s = score(Expansion::Vrc6);
    s.sections[0].tracks[0].notes.push(Note::new(0, 8, 69));
    s.sections[0].tracks[5].notes.push(Note::new(0, 8, 72));
    s.sections[0].tracks[5].solo = true;
    let a = services::render(&s, 0, None).unwrap();
    s.sections[0].tracks[0].notes.clear();
    assert_eq!(a, services::render(&s, 0, None).unwrap());
    let cancel = std::sync::atomic::AtomicBool::new(true);
    assert_eq!(
        services::render(&s, 0, Some(&cancel)).unwrap_err(),
        "Cancelled"
    );
}

#[test]
fn fds_pitch_register_uses_complete_64_sample_periods() {
    let mut s = score(Expansion::Fds);
    s.sections[0].tracks[5].notes.push(Note::new(0, 8, 69));
    let stream = services::compile(&s, 0).unwrap();
    let writes = &stream.events[0].writes;
    let low = writes
        .iter()
        .rev()
        .find(|w| w.address == 0x4082)
        .unwrap()
        .value;
    let high = writes
        .iter()
        .rev()
        .find(|w| w.address == 0x4083)
        .unwrap()
        .value
        & 15;
    let frequency = u16::from(low) | (u16::from(high) << 8);
    assert!((services::CPU * frequency as f64 / (65536.0 * 64.0) - 440.0).abs() < 0.25);
    s.sections[0].tracks[5].notes[0].note = 93;
    assert!(s.validate().is_err());
}

#[test]
fn very_long_silent_sections_encode_without_delay_overflow() {
    let mut s = score(Expansion::None);
    s.bpm = 30.0;
    s.sections[0].rows = 4096;
    let stream = services::compile(&s, 0).unwrap();
    let data = services::encode_stream(&stream).unwrap();
    let mut p = 0;
    let mut total = 0u32;
    loop {
        let delay = u16::from_le_bytes([data[p], data[p + 1]]);
        let count = data[p + 2];
        p += 3;
        if delay == 65535 {
            assert_eq!(count, 255);
            break;
        }
        total += delay as u32;
        p += count as usize * 3;
    }
    assert_eq!(total, 4096 * s.frames_per_row());
    assert!(total > 65535);
}
