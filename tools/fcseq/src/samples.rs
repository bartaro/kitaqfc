//! Non-destructive PCM processing and hardware DPCM encoding.
use rodio::Source;
use serde::{Deserialize, Serialize};
use std::{io::BufReader, path::Path};
pub const DMC_PERIODS: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 85, 72, 54,
];
pub fn rate(index: u8) -> f64 {
    1_789_773.0 / DMC_PERIODS[index.min(15) as usize] as f64
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub name: String,
    pub pcm: Vec<i16>,
    pub sample_rate: u32,
    pub start: u32,
    pub end: u32,
    pub gain: f32,
    pub normalize: bool,
    pub reverse: bool,
    pub rate: u8,
    pub initial_level: u8,
}
impl Sample {
    pub fn pcm8_length(&self, rate: f64) -> usize {
        ((self.end - self.start) as f64 / self.sample_rate as f64 * rate).ceil() as usize
    }
    pub fn pcm8(&self, rate: f64) -> Result<Vec<u8>, String> {
        self.validate()?;
        if !rate.is_finite() || !(500.0..=32000.0).contains(&rate) {
            return Err("PCM resampling rate must be 500..32000 Hz".into());
        }
        let mut pcm: Vec<f64> = self.pcm[self.start as usize..self.end as usize]
            .iter()
            .map(|s| f64::from(*s) / 32768.0)
            .collect();
        if self.reverse {
            pcm.reverse()
        }
        let mean = pcm.iter().sum::<f64>() / pcm.len() as f64;
        for v in &mut pcm {
            *v -= mean;
        }
        let peak = pcm.iter().fold(0f64, |m, v| m.max(v.abs()));
        let gain = f64::from(self.gain)
            * if self.normalize && peak > 1e-8 {
                0.9 / peak
            } else {
                1.0
            };
        let ratio = self.sample_rate as f64 / rate;
        let cutoff = (1.0 / ratio).min(1.0) * 0.9;
        Ok((0..self.pcm8_length(rate))
            .map(|i| {
                let pos = i as f64 * ratio;
                let center = pos.floor() as i64;
                let mut sum = 0.0;
                let mut weight = 0.0;
                for tap in -16..=16 {
                    let idx = center + tap;
                    if idx < 0 || idx >= pcm.len() as i64 {
                        continue;
                    }
                    let x = (idx as f64 - pos) * cutoff;
                    let sinc = if x.abs() < 1e-8 {
                        1.0
                    } else {
                        (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x)
                    };
                    let w = sinc
                        * (0.5 + 0.5 * (std::f64::consts::PI * (idx as f64 - pos) / 17.0).cos())
                        * cutoff;
                    sum += pcm[idx as usize] * w;
                    weight += w;
                }
                (128.0
                    + if weight.abs() > 1e-8 {
                        sum / weight * gain * 127.0
                    } else {
                        0.0
                    })
                .round()
                .clamp(1.0, 255.0) as u8
            })
            .collect())
    }
    pub fn import(path: &Path) -> Result<Self, String> {
        let decoder = rodio::Decoder::new(BufReader::new(
            std::fs::File::open(path).map_err(|e| e.to_string())?,
        ))
        .map_err(|e| format!("Cannot decode audio: {e}"))?;
        let channels = decoder.channels() as usize;
        let sample_rate = decoder.sample_rate();
        if channels == 0 || sample_rate == 0 {
            return Err("Invalid audio format".into());
        }
        let raw: Vec<f32> = decoder
            .convert_samples::<f32>()
            .take(sample_rate as usize * channels * 60 + 1)
            .collect();
        if raw.len() > sample_rate as usize * channels * 60 {
            return Err("Import up to 60 seconds of audio; trim the source first".into());
        }
        let pcm: Vec<i16> = raw
            .chunks_exact(channels)
            .map(|s| ((s.iter().sum::<f32>() / channels as f32).clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();
        if pcm.is_empty() {
            return Err("Audio file contains no samples".into());
        }
        // Retain the complete source; initialize the editable region to hardware capacity.
        let end = pcm
            .len()
            .min((4081.0 * 8.0 / rate(15) * sample_rate as f64).floor() as usize)
            as u32;
        Ok(Self {
            name: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            pcm,
            sample_rate,
            start: 0,
            end,
            gain: 1.0,
            normalize: true,
            reverse: false,
            rate: 15,
            initial_level: 64,
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if !(1000..=384000).contains(&self.sample_rate)
            || self.pcm.len() > self.sample_rate as usize * 60
            || self.start >= self.end
            || self.end as usize > self.pcm.len()
            || !self.gain.is_finite()
            || !(0.0..=8.0).contains(&self.gain)
            || self.rate > 15
            || self.initial_level > 127
        {
            return Err(
                "Invalid sample region, gain (0..8), rate (0..15) or initial level (0..127)".into(),
            );
        }
        Ok(())
    }
    pub fn encoded_size(&self) -> usize {
        let bits = ((self.end - self.start) as f64 / self.sample_rate as f64 * rate(self.rate))
            .ceil() as usize;
        let bytes = bits.div_ceil(8).max(1);
        (bytes - 1).div_ceil(16) * 16 + 1
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let size = self.encoded_size();
        if size > 4081 {
            return Err(format!(
                "DPCM needs {size} bytes (maximum 4081). Shorten the region or lower the rate."
            ));
        }
        let mut pcm: Vec<f64> = self.pcm[self.start as usize..self.end as usize]
            .iter()
            .map(|v| *v as f64 / 32768.0)
            .collect();
        if self.reverse {
            pcm.reverse();
        }
        let mean = pcm.iter().sum::<f64>() / pcm.len() as f64;
        for v in &mut pcm {
            *v -= mean;
        }
        let peak = pcm.iter().fold(0.0_f64, |a, v| a.max(v.abs()));
        let gain = self.gain as f64
            * if self.normalize && peak > 0.000001 {
                0.9 / peak
            } else {
                1.0
            };
        let ratio = self.sample_rate as f64 / rate(self.rate);
        let cutoff = (1.0 / ratio).min(1.0) * 0.9;
        let mut output = vec![0u8; size];
        let mut level = self.initial_level as i32;
        for bit in 0..size * 8 {
            let pos = bit as f64 * ratio;
            let center = pos.floor() as i64;
            let mut sum = 0.0;
            let mut weight = 0.0;
            // Windowed-sinc low-pass prevents aliases when reducing source sample rate.
            if pos < (pcm.len() as f64) {
                for tap in -16..=16 {
                    let idx = center + tap;
                    if idx < 0 || idx >= pcm.len() as i64 {
                        continue;
                    }
                    let x = (idx as f64 - pos) * cutoff;
                    let sinc = if x.abs() < 1e-8 {
                        1.0
                    } else {
                        (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x)
                    };
                    let w = sinc
                        * (0.5 + 0.5 * (std::f64::consts::PI * (idx as f64 - pos) / 17.0).cos())
                        * cutoff;
                    sum += pcm[idx as usize] * w;
                    weight += w;
                }
            }
            let value = if weight.abs() > 1e-8 {
                sum / weight * gain
            } else {
                0.0
            };
            let target = 64.0 + value.clamp(-1.0, 1.0) * 60.0;
            if target >= level as f64 {
                output[bit / 8] |= 1 << (bit % 8);
                if level <= 125 {
                    level += 2;
                }
            } else if level >= 2 {
                level -= 2;
            }
        }
        Ok(output)
    }
}
