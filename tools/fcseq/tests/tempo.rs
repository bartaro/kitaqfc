use fcseq_native::{model::*, nsf, nsf_player, samples::Sample, services};

#[test]
fn cumulative_deadlines_bound_tempo_error_in_ntsc_and_pcm_clocks() {
    for pcm in [false, true] {
        let mut score = Score::empty();
        if pcm {
            score.set_expansion(Expansion::Mmc5).unwrap();
            score.sections[0].tracks[7].notes.push(Note::new(0, 4, 60));
        }
        for bpm in [30.0, 108.0, 120.5, 150.0, 300.0] {
            score.bpm = bpm;
            for row in 0..=4096 {
                let ideal = row as f64 * score.tick_rate() * 15.0 / bpm;
                assert!((score.row_tick(row) as f64 - ideal).abs() <= 0.500000001);
            }
        }
    }
    let mut score = Score::empty();
    score.bpm = 108.0;
    let steps: Vec<_> = (0..128)
        .map(|r| score.row_tick(r + 1) - score.row_tick(r))
        .collect();
    assert!(steps.contains(&8) && steps.contains(&9));
    assert!(steps.iter().all(|n| matches!(n, 8 | 9)));
    assert!((score.actual_bpm() - 108.0).abs() < 0.1);
}

#[test]
fn register_stream_c_delays_and_nsf_playback_use_accumulated_tempo() {
    let mut score = Score::empty();
    score.bpm = 108.0;
    for row in 0..64 {
        score.sections[0].tracks[0]
            .notes
            .push(Note::new(row, 1, 69));
    }
    let stream = services::compile(&score, 0).unwrap();
    let expected: Vec<u32> = (0..64)
        .map(|r| (r as f64 * FPS * 15.0 / 108.0).round() as u32)
        .collect();
    let actual: Vec<_> = stream
        .events
        .iter()
        .filter(|e| e.writes.iter().any(|w| w.address == 0x4003))
        .map(|e| e.frame)
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(stream.frames, 534);
    // Decode the delay records used by the C and NSF drivers independently.
    let bytes = services::encode_stream(&stream).unwrap();
    let (mut cursor, mut time) = (0, 0u32);
    let mut starts = vec![];
    loop {
        let delay = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        let count = bytes[cursor + 2] as usize;
        cursor += 3;
        if delay == 65535 {
            break;
        }
        time += u32::from(delay);
        let mut has_start = false;
        for _ in 0..count {
            let address = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
            has_start |= address == 0x4003;
            cursor += 3;
        }
        if has_start {
            starts.push(time);
        }
    }
    assert_eq!(starts, expected);
    assert_eq!(time, 534);
    let mut held = Score::empty();
    held.bpm = 108.0;
    held.sections[0].rows = 16;
    held.sections[0].tracks[0].notes.push(Note::new(0, 16, 69));
    let wav = services::render(&held, 0, None).unwrap();
    assert!((wav.len() as f64 / 44100.0 - 16.0 * 15.0 / 108.0).abs() <= 1.0 / FPS);
    let exported = nsf_player::render(
        &nsf::Imported {
            data: nsf::export_bytes(&held).unwrap(),
            song: 0,
            seconds: 3,
            duration_ms: None,
        },
        None,
    )
    .unwrap();
    let rms = |a: usize, b: usize| {
        (exported[a..b]
            .iter()
            .map(|&x| (x as f64).powi(2))
            .sum::<f64>()
            / (b - a) as f64)
            .sqrt()
    };
    assert!(
        rms(96000, 97000) > 100.0,
        "108 BPM must still sound at 2.2 seconds"
    );
    assert!(
        rms(104000, 105000) < 10.0,
        "NSF must stop at the corrected section boundary"
    );
}

#[test]
fn pcm_duration_and_shared_ssg_mixer_follow_fractional_note_edges() {
    let mut score = Score::empty();
    score.set_expansion(Expansion::Mmc5).unwrap();
    score.bpm = 108.0;
    score.sections[0].rows = 16;
    score.samples.push(Sample {
        name: "Tempo fixture".into(),
        pcm: vec![12000; 22050],
        sample_rate: 44100,
        start: 0,
        end: 22050,
        gain: 1.0,
        normalize: false,
        reverse: false,
        rate: 12,
        initial_level: 64,
    });
    let mut note = Note::new(3, 2, 60);
    note.sound = Some(Sound {
        instrument: 0,
        ..Sound::default()
    });
    score.sections[0].tracks[7].notes.push(note);
    let stream = services::compile(&score, 0).unwrap();
    let on = score.row_tick(3);
    let off = score.row_tick(5);
    assert_eq!((on, off), (1667, 2778));
    assert!(stream.events.iter().any(|e| e.frame == on
        && e.writes
            .iter()
            .any(|w| w.address == 0x5011 && w.value != 128)));
    assert!(stream.events.iter().any(|e| e.frame == off
        && e.writes
            .iter()
            .any(|w| w.address == 0x5011 && w.value == 128)));
    let mut ssg = Score::empty();
    ssg.set_expansion(Expansion::Sunsoft5b).unwrap();
    ssg.bpm = 108.0;
    ssg.sections[0].tracks[5].notes.push(Note::new(3, 2, 60));
    let stream = services::compile(&ssg, 0).unwrap();
    for (tick, value) in [(25, 0x3e), (42, 0x3f)] {
        assert!(stream.events.iter().any(|e| e.frame == tick
            && e.writes.windows(2).any(|w| w[0].address == 0xc000
                && w[0].value == 7
                && w[1].address == 0xe000
                && w[1].value == value)));
    }
}
