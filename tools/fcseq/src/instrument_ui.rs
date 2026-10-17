use super::*;
use fcseq_native::instruments::{wave_preset, FM_PARAMETERS};

impl Studio {
    pub(super) fn instrument_kind(&self) -> Voice {
        self.score.expansion.kind(self.instrument_track)
    }
    pub(super) fn instrument_selection(&self) -> Selection {
        self.selection
            .iter()
            .copied()
            .filter(|(ch, _)| *ch == self.instrument_track)
            .collect()
    }
    pub(super) fn instrument_sound(&self) -> Sound {
        self.instrument_selection()
            .first()
            .map(|&(ch, i)| self.score.sections[self.part].tracks[ch].notes[i].sound(ch))
            .unwrap_or_else(Sound::default)
    }
    pub(super) fn wave_editable(&self) -> bool {
        matches!(self.instrument_kind(), Voice::Fds | Voice::N163)
    }
    pub(super) fn instrument_wave(&self) -> Vec<u8> {
        let s = self.instrument_sound();
        match self.instrument_kind() {
            Voice::Fds => self.score.fds_wave.clone(),
            Voice::N163 => self.score.n163_waves[self.wave_slot].clone(),
            Voice::Pulse | Voice::VrcPulse => {
                let duty = if self.instrument_kind() == Voice::Pulse {
                    [8, 16, 32, 48][s.duty.min(3) as usize]
                } else {
                    (s.duty + 1) as usize * 4
                };
                (0..64).map(|i| if i < duty { 63 } else { 0 }).collect()
            }
            Voice::Triangle => (0..32)
                .map(|i| if i < 16 { (15 - i) * 4 } else { (i - 16) * 4 })
                .collect(),
            Voice::Noise => {
                let mut lfsr = 1u16;
                (0..128)
                    .map(|_| {
                        let bit = (lfsr ^ (lfsr >> if s.noise_short { 6 } else { 1 })) & 1;
                        lfsr = (lfsr >> 1) | (bit << 14);
                        if lfsr & 1 == 0 {
                            63
                        } else {
                            0
                        }
                    })
                    .collect()
            }
            // Seven accumulator additions, then a reset. The 8-bit accumulator
            // wraps at high rates, which changes the timbre of the VRC6 saw.
            Voice::Saw => (0..14)
                .map(|i| {
                    (((i / 2) as u8).wrapping_mul(s.saw_rate.unwrap_or(s.volume * 2)) >> 3) * 2
                })
                .collect(),
            _ => vec![],
        }
    }
    pub(super) fn instrument_buttons(&self) -> Vec<Button> {
        let m = self.modal_rect();
        let mut b = vec![];
        let mut add =
            |a, x, y, w, label: String| b.push(Button::new(a, m.x + x, m.y + y, w, label, None));
        add(Action::InstrumentTrack(-1), 28.0, 62.0, 32.0, "<".into());
        add(Action::InstrumentTrack(1), 282.0, 62.0, 32.0, ">".into());
        add(
            Action::InstrumentPlay,
            328.0,
            62.0,
            140.0,
            "AUDITION C4".into(),
        );
        add(Action::Stop, 478.0, 62.0, 70.0, "STOP".into());
        add(Action::Undo, 558.0, 62.0, 70.0, "UNDO".into());
        add(Action::Redo, 638.0, 62.0, 70.0, "REDO".into());
        if self.instrument_kind() == Voice::Fm {
            for (i, p) in FM_PARAMETERS.iter().enumerate() {
                let x = 28.0 + (i / 12) as f32 * 414.0;
                let y = 116.0 + (i % 12) as f32 * 34.0;
                add(Action::FmParam(i, -1), x, y, 28.0, "-".into());
                add(Action::FmParam(i, 1), x + 366.0, y, 28.0, "+".into());
                let _ = p;
            }
            add(
                Action::Input(Field::Patch),
                28.0,
                538.0,
                264.0,
                "ADVANCED: 8 HEX BYTES".into(),
            );
        } else {
            if self.wave_editable() {
                for (i, label) in ["SINE", "PULSE", "SAW", "TRIANGLE"].iter().enumerate() {
                    add(
                        Action::WavePreset(i as u8),
                        28.0 + i as f32 * 112.0,
                        318.0,
                        102.0,
                        (*label).into(),
                    );
                }
                add(Action::WaveInvert, 480.0, 318.0, 100.0, "INVERT".into());
                add(
                    Action::WaveRotate(-1),
                    590.0,
                    318.0,
                    100.0,
                    "SHIFT <".into(),
                );
                add(Action::WaveRotate(1), 700.0, 318.0, 100.0, "SHIFT >".into());
            }
            if self.instrument_kind() == Voice::N163 {
                add(Action::WaveStep(-1), 28.0, 368.0, 30.0, "<".into());
                add(Action::WaveStep(1), 190.0, 368.0, 30.0, ">".into());
                add(Action::WaveLength(-4), 252.0, 368.0, 30.0, "-".into());
                add(Action::WaveLength(4), 426.0, 368.0, 30.0, "+".into());
                add(Action::N163Channels(-1), 494.0, 368.0, 30.0, "-".into());
                add(Action::N163Channels(1), 744.0, 368.0, 30.0, "+".into());
            }
            for (i, (id, _, _)) in self.timbre_parameters().iter().enumerate() {
                let x = 28.0 + (i % 2) as f32 * 414.0;
                let y = 454.0 + (i / 2) as f32 * 40.0;
                add(Action::Timbre(*id, -1), x, y, 30.0, "-".into());
                add(Action::Timbre(*id, 1), x + 350.0, y, 30.0, "+".into());
            }
            if self.instrument_kind() == Voice::Dpcm {
                add(
                    Action::Samples,
                    28.0,
                    318.0,
                    280.0,
                    "OPEN DPCM SAMPLE STUDIO".into(),
                );
            }
        }
        b
    }
    pub(super) fn timbre_parameters(&self) -> Vec<(u8, &'static str, u8)> {
        let s = self.instrument_sound();
        let mut p = vec![];
        if !matches!(
            self.instrument_kind(),
            Voice::Triangle | Voice::Dpcm | Voice::Fm
        ) {
            if self.instrument_kind() != Voice::Saw {
                p.push((0, "VOLUME", s.volume));
            }
            p.push((1, "DECAY (FRAMES)", s.decay));
        }
        match self.instrument_kind() {
            Voice::Pulse | Voice::VrcPulse => p.push((2, "DUTY", s.duty)),
            Voice::Saw => p.push((
                3,
                "SAW ACCUMULATOR RATE",
                s.saw_rate.unwrap_or(s.volume * 2),
            )),
            Voice::N163 => p.push((4, "NOTE WAVE (1-BASED)", s.instrument + 1)),
            Voice::Dpcm => p.push((4, "SAMPLE (1-BASED)", s.instrument + 1)),
            Voice::Noise => {
                p.push((5, "NOISE PERIOD", s.noise_period.unwrap_or(8)));
                p.push((6, "SHORT MODE", s.noise_short as u8));
            }
            _ => {}
        }
        p
    }
    pub(super) fn draw_instrument(&self, d: &mut DrawList) {
        let m = self.modal_rect();
        draw_text(
            d,
            m.x + 72.0,
            m.y + 72.0,
            &self.score.expansion.channel_name(self.instrument_track),
            0.85,
            ACCENT,
        );
        if self.instrument_kind() == Voice::Fm {
            for (i, p) in FM_PARAMETERS.iter().enumerate() {
                let x = m.x + 68.0 + (i / 12) as f32 * 414.0;
                let y = m.y + 126.0 + (i % 12) as f32 * 34.0;
                let v = p.get(&self.score.vrc7_patch);
                let value = if p.name.ends_with("WAVE") {
                    if v == 0 {
                        "SINE".into()
                    } else {
                        "HALF SINE".into()
                    }
                } else {
                    v.to_string()
                };
                draw_text(d, x, y, &format!("{}  {}", p.name, value), 0.76, TEXT);
            }
            draw_text(
                d,
                m.x + 312.0,
                m.y + 548.0,
                &self
                    .score
                    .vrc7_patch
                    .iter()
                    .map(|v| format!("{v:02X}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                0.9,
                ACCENT,
            );
            draw_text(
                d,
                m.x + 28.0,
                m.y + 585.0,
                "Shared custom patch: assign patch 0 to notes. Audition always uses patch 0.",
                0.73,
                MUTED,
            );
        } else {
            let r = self.wave_rect();
            d.fill_rect(r, BG);
            let wave = self.instrument_wave();
            let max = if self.instrument_kind() == Voice::N163 {
                15.0
            } else {
                63.0
            };
            for (i, v) in wave.iter().enumerate() {
                let h = *v as f32 / max * r.height;
                d.fill_rect(
                    Rect::new(
                        r.x + i as f32 * r.width / wave.len() as f32,
                        r.bottom() - h,
                        (r.width / wave.len() as f32 - 1.0).max(1.0),
                        h,
                    ),
                    ACCENT,
                );
            }
            let help = match self.instrument_kind() {
                Voice::Fds => {
                    "Draw the shared 64-sample FDS waveform (6-bit). Drag across the graph."
                }
                Voice::N163 => "Draw 4-bit samples. Select this wave on notes using NOTE WAVE.",
                Voice::Pulse => "Fixed pulse waveform: duty 0..3 selects 12.5%, 25%, 50%, 75%.",
                Voice::VrcPulse => {
                    "Fixed VRC6 pulse: duty 0..7 selects 6.25%..50%. Adjust while viewing above."
                }
                Voice::Saw => {
                    "Fixed VRC6 saw: rate 0..63 controls accumulator steps; high rates wrap."
                }
                Voice::Triangle => {
                    "Fixed 32-step triangle. Edit note pitch / length in the sequencer."
                }
                Voice::Noise => {
                    "Noise generator: period 0..15 and long / short mode. No editable wave RAM."
                }
                _ => "DPCM: import, trim, gain, normalize and reverse audio in the sample studio.",
            };
            draw_text(d, m.x + 28.0, m.y + 418.0, help, 0.76, TEXT);
            if self.instrument_kind() == Voice::N163 {
                for (x, t) in [
                    (
                        68.0,
                        format!(
                            "WAVE {} / {}",
                            self.wave_slot + 1,
                            self.score.n163_waves.len()
                        ),
                    ),
                    (292.0, format!("LENGTH {}", wave.len())),
                    (534.0, format!("CHANNELS {}", self.score.n163_channels)),
                ] {
                    draw_text(d, m.x + x, m.y + 378.0, &t, 0.8, ACCENT);
                }
                draw_text(d,m.x+28.0,m.y+565.0,&format!("Wave RAM: {} / {} samples. Lengths are multiples of 4; channels share RAM.",self.score.n163_waves.iter().map(Vec::len).sum::<usize>(),self.score.n163_wave_capacity()),0.73,MUTED);
            }
            for (i, (_, label, v)) in self.timbre_parameters().iter().enumerate() {
                draw_text(
                    d,
                    m.x + 70.0 + (i % 2) as f32 * 414.0,
                    m.y + 464.0 + (i / 2) as f32 * 40.0,
                    &format!("{label}  {v}"),
                    0.78,
                    TEXT,
                );
            }
            draw_text(
                d,
                m.x + 28.0,
                m.y + 590.0,
                if self.instrument_selection().is_empty() {
                    "Select notes on this channel to edit note parameters. Wave edits are shared."
                } else {
                    "Note parameters apply to selected notes on this channel. Wave edits are shared."
                },
                0.73,
                MUTED,
            );
        }
    }
    pub(super) fn instrument_action(&mut self, a: Action) {
        let slot = self.wave_slot;
        let n163 = self.instrument_kind() == Voice::N163;
        match a {
            Action::InstrumentTrack(d) => {
                self.instrument_track = (self.instrument_track as i32 + d as i32)
                    .rem_euclid(self.score.expansion.channels() as i32)
                    as usize;
            }
            Action::WaveStep(d) => {
                self.wave_slot =
                    (slot as i32 + d as i32).rem_euclid(self.score.n163_waves.len() as i32) as usize
            }
            Action::N163Channels(d) => self.change(|s| {
                s.n163_channels = (s.n163_channels as i16 + d as i16).clamp(1, 8) as u8;
                Ok(())
            }),
            Action::WaveLength(d) => self.change(|s| {
                let old = &s.n163_waves[slot];
                let len = (old.len() as i16 + d).clamp(4, 240) as usize;
                s.n163_waves[slot] = (0..len).map(|i| old[i * old.len() / len]).collect();
                Ok(())
            }),
            Action::WavePreset(_) | Action::WaveInvert | Action::WaveRotate(_) => {
                self.change(|s| {
                    let wave = if n163 {
                        &mut s.n163_waves[slot]
                    } else {
                        &mut s.fds_wave
                    };
                    let max = if n163 { 15 } else { 63 };
                    match a {
                        Action::WavePreset(p) => *wave = wave_preset(wave.len(), max, p),
                        Action::WaveInvert => wave.iter_mut().for_each(|v| *v = max - *v),
                        Action::WaveRotate(d) => {
                            if d < 0 {
                                wave.rotate_left(1)
                            } else {
                                wave.rotate_right(1)
                            }
                        }
                        _ => {}
                    };
                    Ok(())
                })
            }
            Action::FmParam(i, d) => self.change(|s| {
                let p = FM_PARAMETERS[i];
                let value = (p.get(&s.vrc7_patch) as i16 + d).clamp(0, p.max as i16) as u8;
                p.set(&mut s.vrc7_patch, value)
            }),
            Action::Timbre(id, d) => {
                let selection = self.instrument_selection();
                let part = self.part;
                let edit = match id {
                    0 => Edit::Volume(d),
                    1 => Edit::Decay(d),
                    2 => Edit::Duty(d),
                    3 => Edit::SawRate(d),
                    4 => Edit::Instrument(d),
                    5 => Edit::NoisePeriod(d),
                    _ => Edit::NoiseMode,
                };
                self.change(|s| editing::edit_notes(s, part, &selection, edit).map(|_| ()));
            }
            Action::InstrumentPlay => {
                if self.instrument_kind() == Voice::Dpcm {
                    self.play_sample();
                    return;
                }
                let mut score = self.score.clone();
                let ch = self.instrument_track;
                score.sections = vec![Section::with_channels(
                    "Audition",
                    score.expansion.channels(),
                )];
                score.sections[0].rows = 16;
                let mut note = Note::new(0, 8, if ch == 3 { 2 } else { 60 });
                let mut sound = self.instrument_sound();
                if n163 {
                    sound.instrument = slot as u8;
                } else if self.instrument_kind() == Voice::Fm {
                    sound.instrument = 0;
                }
                note.sound = Some(sound);
                score.sections[0].tracks[ch].notes.push(note);
                self.launch_render(score, 0, false);
            }
            _ => {}
        }
    }
    pub(super) fn paint_instrument_wave(&mut self, p: Point) {
        let r = self.wave_rect();
        let n163 = self.instrument_kind() == Voice::N163;
        let wave = if n163 {
            &mut self.score.n163_waves[self.wave_slot]
        } else {
            &mut self.score.fds_wave
        };
        let max = if n163 { 15.0 } else { 63.0 };
        let i = ((p.x - r.x) / r.width * wave.len() as f32).clamp(0.0, (wave.len() - 1) as f32)
            as usize;
        let v = ((r.bottom() - p.y) / r.height * max)
            .round()
            .clamp(0.0, max) as u8;
        if let Some((last, value)) = self.wave_last {
            for j in last.min(i)..=last.max(i) {
                wave[j] = if last == i {
                    v
                } else {
                    (value as f32
                        + (v as f32 - value as f32) * (j as f32 - last as f32)
                            / (i as f32 - last as f32))
                        .round() as u8
                };
            }
        } else {
            wave[i] = v;
        }
        self.wave_last = Some((i, v));
    }
}
