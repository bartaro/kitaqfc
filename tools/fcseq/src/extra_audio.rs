//! Register compiler for MMC5, 5B, EPSM and VT02. No chip-specific GUI state.
use crate::{model::*, services::Write};
fn w(out: &mut Vec<Write>, address: u16, value: u8) {
    out.push(Write { address, value });
}
fn psg(out: &mut Vec<Write>, chip: Expansion, register: u8, value: u8) {
    let (a, d) = if chip == Expansion::Epsm {
        (0x401c, 0x401d)
    } else {
        (0xc000, 0xe000)
    };
    w(out, a, register);
    w(out, d, value);
}
fn opn(out: &mut Vec<Write>, bank: u8, register: u8, value: u8) {
    w(out, 0x401c + u16::from(bank) * 2, register);
    w(out, 0x401d + u16::from(bank) * 2, value);
}
pub fn init(score: &Score) -> Vec<Write> {
    let mut out = vec![];
    match score.expansion {
        Expansion::Mmc5 => {
            for (a, v) in [
                (0x5015, 3),
                (0x5010, 0),
                (0x5011, 128),
                (0x5000, 0x30),
                (0x5004, 0x30),
            ] {
                w(&mut out, a, v)
            }
        }
        Expansion::Sunsoft5b | Expansion::Epsm => {
            psg(&mut out, score.expansion, 7, 0x3f);
            for i in 8..=10 {
                psg(&mut out, score.expansion, i, 0)
            }
            if score.expansion == Expansion::Epsm {
                for (r, v) in [
                    (0x20, 2),
                    // Shared LFO at rate 4; zero AM/PM sensitivity stays unmodulated.
                    (0x22, 0x0c),
                    (0x27, 0),
                    (0x29, 0x80),
                    (0x11, 63),
                    (0x10, 0xbf),
                ] {
                    opn(&mut out, 0, r, v)
                }
                for c in 0..6 {
                    opn(&mut out, 0, 0x28, (c % 3) | if c >= 3 { 4 } else { 0 })
                }
            }
        }
        Expansion::Vt02 => {
            for (a, v) in [
                (0x4030, 8),
                (0x4031, 128),
                (0x4035, 15),
                (0x4020, 0x30),
                (0x4024, 0x30),
                (0x4028, 0x80),
                (0x402c, 0x30),
            ] {
                w(&mut out, a, v)
            }
        }
        _ => {}
    }
    out
}
pub fn base(chip: Expansion, ch: usize) -> u16 {
    if chip == Expansion::Mmc5 {
        0x5000 + (ch - 5) as u16 * 4
    } else {
        0x4020 + (ch - 5) as u16 * 4
    }
}
pub fn swap_vt_duty(v: u8) -> u8 {
    (v & 63)
        | (match v >> 6 {
            1 => 2,
            2 => 1,
            x => x,
        } << 6)
}
pub fn on(score: &Score, ch: usize, n: &Note, s: &Sound) -> Vec<Write> {
    let mut out = vec![];
    let kind = score.expansion.kind(ch);
    let hz = 440.0 * 2f64.powf((f64::from(n.note) - 69.0) / 12.0);
    match kind {
        Voice::MmcPulse | Voice::VtPulse | Voice::VtTriangle => {
            let b = base(score.expansion, ch);
            let timer = (crate::services::CPU
                / (if kind == Voice::VtTriangle {
                    32.0
                } else {
                    16.0
                })
                / hz
                - 1.0)
                .round()
                .clamp(0.0, 2047.0) as u16;
            let ctl = if kind == Voice::VtTriangle {
                0xff
            } else {
                (s.duty << 6) | 0x30 | s.volume
            };
            w(
                &mut out,
                b,
                if kind == Voice::VtPulse {
                    swap_vt_duty(ctl)
                } else {
                    ctl
                },
            );
            w(&mut out, b + 2, timer as u8);
            w(&mut out, b + 3, 8 | (timer >> 8) as u8);
        }
        Voice::VtNoise => {
            w(&mut out, 0x402c, 0x30 | s.volume);
            w(
                &mut out,
                0x402e,
                s.noise_period
                    .unwrap_or([13, 8, 2, 3][(n.note - 1) as usize])
                    | if s.noise_short { 128 } else { 0 },
            );
            w(&mut out, 0x402f, 8)
        }
        Voice::Psg => {
            let c = (ch - 5) as u8;
            let clock = if score.expansion == Expansion::Epsm {
                2_000_000.0
            } else {
                crate::services::CPU / 2.0
            };
            let period = (clock / (16.0 * hz)).round().clamp(1.0, 4095.0) as u16;
            for (r, v) in [
                (c * 2, period as u8),
                (c * 2 + 1, (period >> 8) as u8),
                (6, s.noise_period.unwrap_or(8)),
                (8 + c, if s.envelope { 16 } else { s.volume }),
            ] {
                psg(&mut out, score.expansion, r, v)
            }
            if s.envelope {
                for (r, v) in [
                    (11, s.envelope_period as u8),
                    (12, (s.envelope_period >> 8) as u8),
                    (13, s.envelope_shape),
                ] {
                    psg(&mut out, score.expansion, r, v)
                }
            }
            // Mixer register is resolved across all three tracks after event compilation.
        }
        Voice::EpsmFm => {
            let c = (ch - 8) as u8;
            let bank = c / 3;
            let channel = c % 3;
            let key = channel | if bank != 0 { 4 } else { 0 };
            let patch = &score.epsm_patches[s.instrument as usize];
            opn(&mut out, 0, 0x28, key);
            for op in 0..4 {
                let slot = [0, 8, 4, 12][op];
                for r in 0..7 {
                    let mut value = patch[op * 7 + r];
                    if r == 1 {
                        let carrier = match patch[28] & 7 {
                            0..=3 => op == 3,
                            4 => op == 1 || op == 3,
                            5 | 6 => op >= 1,
                            _ => true,
                        };
                        if carrier {
                            value = value.saturating_add((15 - s.volume) * 4).min(127)
                        }
                    }
                    opn(&mut out, bank, 0x30 + r as u8 * 16 + slot + channel, value)
                }
            }
            opn(&mut out, bank, 0xb0 + channel, patch[28]);
            opn(
                &mut out,
                bank,
                0xb4 + channel,
                (s.pan << 6) | (patch[29] & 0x37),
            );
            let (num, block) = epsm_pitch(n.note);
            opn(
                &mut out,
                bank,
                0xa4 + channel,
                ((num >> 8) as u8) | (block << 3),
            );
            opn(&mut out, bank, 0xa0 + channel, num as u8);
            opn(&mut out, 0, 0x28, 0xf0 | key);
        }
        Voice::Rhythm => {
            let c = (ch - 14) as u8;
            opn(&mut out, 0, 0x18 + c, (s.pan << 6) | (s.volume * 2));
            opn(&mut out, 0, 0x10, 1 << c)
        }
        _ => {}
    }
    out
}
pub fn epsm_pitch(note: u8) -> (u16, u8) {
    let hz = 440.0 * 2f64.powf((f64::from(note) - 69.0) / 12.0);
    for block in 0..8 {
        // OPN phase increment is FNUM * 2^(BLOCK-1) at clock/144.
        let n = (hz * 144.0 * 2097152.0 / (8_000_000.0 * 2f64.powi(block))).round();
        if n <= 2047.0 {
            return (n as u16, block as u8);
        }
    }
    (2047, 7)
}
pub fn off(score: &Score, ch: usize) -> Vec<Write> {
    let mut out = vec![];
    match score.expansion.kind(ch) {
        Voice::MmcPulse | Voice::VtPulse => w(&mut out, base(score.expansion, ch), 0x30),
        Voice::VtTriangle => w(&mut out, 0x4028, 0x80),
        Voice::VtNoise => w(&mut out, 0x402c, 0x30),
        Voice::Psg => psg(&mut out, score.expansion, 8 + (ch - 5) as u8, 0),
        Voice::EpsmFm => {
            let c = (ch - 8) as u8;
            opn(&mut out, 0, 0x28, c % 3 | if c >= 3 { 4 } else { 0 })
        }
        Voice::Rhythm => opn(&mut out, 0, 0x10, 0x80 | (1 << (ch - 14))),
        Voice::Pcm => w(
            &mut out,
            if score.expansion == Expansion::Mmc5 {
                0x5011
            } else {
                0x4031
            },
            128,
        ),
        _ => {}
    }
    out
}
pub fn volume(score: &Score, ch: usize, s: &Sound, v: u8) -> Vec<Write> {
    let mut out = vec![];
    match score.expansion.kind(ch) {
        Voice::MmcPulse | Voice::VtPulse => {
            let ctl = s.duty << 6 | 0x30 | v;
            w(
                &mut out,
                base(score.expansion, ch),
                if score.expansion == Expansion::Vt02 {
                    swap_vt_duty(ctl)
                } else {
                    ctl
                },
            )
        }
        Voice::VtNoise => w(&mut out, 0x402c, 0x30 | v),
        Voice::Psg if !s.envelope => psg(&mut out, score.expansion, 8 + (ch - 5) as u8, v),
        Voice::Rhythm => opn(&mut out, 0, 0x18 + (ch - 14) as u8, (s.pan << 6) | (v * 2)),
        Voice::EpsmFm => {
            let c = (ch - 8) as u8;
            let patch = &score.epsm_patches[s.instrument as usize];
            for op in 0..4 {
                let carrier = match patch[28] & 7 {
                    0..=3 => op == 3,
                    4 => op == 1 || op == 3,
                    5 | 6 => op >= 1,
                    _ => true,
                };
                if carrier {
                    opn(
                        &mut out,
                        c / 3,
                        0x40 + [0, 8, 4, 12][op] + c % 3,
                        patch[op * 7 + 1].saturating_add((15 - v) * 4).min(127),
                    );
                }
            }
        }
        _ => {}
    }
    out
}
