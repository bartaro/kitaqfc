//! Hardware instrument parameters shared by GUI, Python and register export.
pub fn wave_preset(length: usize, max: u8, shape: u8) -> Vec<u8> {
    (0..length)
        .map(|i| {
            let x = i as f64 / length as f64;
            let y = match shape {
                0 => (x * std::f64::consts::TAU).sin() * 0.5 + 0.5,
                1 => {
                    if i < length / 2 {
                        1.0
                    } else {
                        0.0
                    }
                }
                2 => i as f64 / (length - 1).max(1) as f64,
                _ => 1.0 - (x * 2.0 - 1.0).abs(),
            };
            (y * max as f64).round().clamp(0.0, max as f64) as u8
        })
        .collect()
}
pub fn n163_frequency(note: u8, length: usize, channels: u8) -> u32 {
    let hz = 440.0 * 2f64.powf((note as f64 - 69.0) / 12.0);
    (hz * 15.0 * 65536.0 * length as f64 * channels as f64 / crate::services::CPU).round() as u32
}
#[derive(Clone, Copy)]
pub struct FmParameter {
    pub name: &'static str,
    pub byte: usize,
    pub shift: u8,
    pub max: u8,
}
// VRC7 custom instrument bit fields. Operator wave is sine / rectified sine.
// https://www.nesdev.org/wiki/VRC7_audio
pub const FM_PARAMETERS: &[FmParameter] = &[
    FmParameter {
        name: "MOD MULTIPLIER",
        byte: 0,
        shift: 0,
        max: 15,
    },
    FmParameter {
        name: "MOD ATTACK",
        byte: 4,
        shift: 4,
        max: 15,
    },
    FmParameter {
        name: "MOD DECAY",
        byte: 4,
        shift: 0,
        max: 15,
    },
    FmParameter {
        name: "MOD SUSTAIN LEVEL",
        byte: 6,
        shift: 4,
        max: 15,
    },
    FmParameter {
        name: "MOD RELEASE",
        byte: 6,
        shift: 0,
        max: 15,
    },
    FmParameter {
        name: "MOD LEVEL",
        byte: 2,
        shift: 0,
        max: 63,
    },
    FmParameter {
        name: "MOD KEY LEVEL",
        byte: 2,
        shift: 6,
        max: 3,
    },
    FmParameter {
        name: "MOD KEY RATE",
        byte: 0,
        shift: 4,
        max: 1,
    },
    FmParameter {
        name: "MOD SUSTAIN",
        byte: 0,
        shift: 5,
        max: 1,
    },
    FmParameter {
        name: "MOD VIBRATO",
        byte: 0,
        shift: 6,
        max: 1,
    },
    FmParameter {
        name: "MOD TREMOLO",
        byte: 0,
        shift: 7,
        max: 1,
    },
    FmParameter {
        name: "MOD WAVE",
        byte: 3,
        shift: 3,
        max: 1,
    },
    FmParameter {
        name: "CAR MULTIPLIER",
        byte: 1,
        shift: 0,
        max: 15,
    },
    FmParameter {
        name: "CAR ATTACK",
        byte: 5,
        shift: 4,
        max: 15,
    },
    FmParameter {
        name: "CAR DECAY",
        byte: 5,
        shift: 0,
        max: 15,
    },
    FmParameter {
        name: "CAR SUSTAIN LEVEL",
        byte: 7,
        shift: 4,
        max: 15,
    },
    FmParameter {
        name: "CAR RELEASE",
        byte: 7,
        shift: 0,
        max: 15,
    },
    FmParameter {
        name: "FEEDBACK",
        byte: 3,
        shift: 0,
        max: 7,
    },
    FmParameter {
        name: "CAR KEY LEVEL",
        byte: 3,
        shift: 6,
        max: 3,
    },
    FmParameter {
        name: "CAR KEY RATE",
        byte: 1,
        shift: 4,
        max: 1,
    },
    FmParameter {
        name: "CAR SUSTAIN",
        byte: 1,
        shift: 5,
        max: 1,
    },
    FmParameter {
        name: "CAR VIBRATO",
        byte: 1,
        shift: 6,
        max: 1,
    },
    FmParameter {
        name: "CAR TREMOLO",
        byte: 1,
        shift: 7,
        max: 1,
    },
    FmParameter {
        name: "CAR WAVE",
        byte: 3,
        shift: 4,
        max: 1,
    },
];
impl FmParameter {
    pub fn get(self, patch: &[u8; 8]) -> u8 {
        (patch[self.byte] >> self.shift) & self.max
    }
    pub fn set(self, patch: &mut [u8; 8], value: u8) -> Result<(), String> {
        if value > self.max {
            return Err(format!("{} must be 0..{}", self.name, self.max));
        }
        patch[self.byte] = (patch[self.byte] & !(self.max << self.shift)) | (value << self.shift);
        Ok(())
    }
}
