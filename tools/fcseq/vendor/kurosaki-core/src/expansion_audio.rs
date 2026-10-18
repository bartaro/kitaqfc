//! Audio-only expansion devices used by the sequencer and NSF host.
//! Register references are listed in FCSEQ's docs/AUDIO_FORMATS.md.
use crate::{
    apu::ApuState,
    mapper::{Mapper, MapperDebugState, NromMapper},
    Mirroring, TraceConfig, TraceSink,
};
use std::{ffi::c_void, ptr::NonNull};

extern "C" {
    fn fcseq_ym_new(epsm: u8) -> *mut c_void;
    fn fcseq_ym_delete(p: *mut c_void);
    fn fcseq_ym_write(p: *mut c_void, port: u8, value: u8);
    fn fcseq_ym_read(p: *mut c_void, port: u8) -> u8;
    fn fcseq_ym_clock(p: *mut c_void, cycles: u32);
    fn fcseq_ym_sample(p: *mut c_void) -> i32;
    fn fcseq_ym_irq(p: *mut c_void) -> u8;
    fn fcseq_ym_rom(p: *mut c_void, data: *const u8);
}
struct Yamaha(NonNull<c_void>);
// Each allocation belongs to exactly one mapper; no shared C++ state exists.
unsafe impl Send for Yamaha {}
impl Yamaha {
    fn new(epsm: bool) -> Self {
        Self(NonNull::new(unsafe { fcseq_ym_new(epsm as u8) }).expect("Yamaha allocation"))
    }
    fn write(&mut self, p: u8, v: u8) {
        unsafe { fcseq_ym_write(self.0.as_ptr(), p, v) }
    }
    fn read(&mut self, p: u8) -> u8 {
        unsafe { fcseq_ym_read(self.0.as_ptr(), p) }
    }
    fn clock(&mut self, c: u64) {
        for step in 0..c / u32::MAX as u64 {
            let _ = step;
            unsafe { fcseq_ym_clock(self.0.as_ptr(), u32::MAX) }
        }
        unsafe { fcseq_ym_clock(self.0.as_ptr(), (c % u32::MAX as u64) as u32) }
    }
    fn sample(&self) -> i32 {
        unsafe { fcseq_ym_sample(self.0.as_ptr()) }
    }
    fn irq(&self) -> bool {
        unsafe { fcseq_ym_irq(self.0.as_ptr()) != 0 }
    }
}
impl Drop for Yamaha {
    fn drop(&mut self) {
        unsafe { fcseq_ym_delete(self.0.as_ptr()) }
    }
}

#[derive(Default)]
struct Pulse {
    ctl: u8,
    timer: u16,
    phase: u8,
    clock: u64,
    length: u8,
    env: u8,
    divider: u8,
    start: bool,
}
impl Pulse {
    fn write(&mut self, reg: u16, v: u8, enabled: bool) {
        match reg {
            0 => self.ctl = v,
            2 => self.timer = (self.timer & 0x700) | u16::from(v),
            3 => {
                self.timer = (self.timer & 255) | (u16::from(v & 7) << 8);
                if enabled {
                    self.length = [
                        10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24,
                        18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
                    ][(v >> 3) as usize];
                }
                self.start = true;
            }
            _ => {}
        }
    }
    fn clock(&mut self, c: u64) {
        self.clock += c;
        let period = 2 * (u64::from(self.timer) + 1);
        self.phase = (u64::from(self.phase) + self.clock / period) as u8 & 7;
        self.clock %= period;
    }
    fn envelope(&mut self) {
        if self.start {
            self.start = false;
            self.env = 15;
            self.divider = self.ctl & 15;
        } else if self.divider == 0 {
            self.divider = self.ctl & 15;
            if self.env > 0 {
                self.env -= 1
            } else if self.ctl & 32 != 0 {
                self.env = 15
            }
        } else {
            self.divider -= 1;
        }
        if self.ctl & 32 == 0 {
            self.length = self.length.saturating_sub(1)
        }
    }
    fn sample(&self) -> i32 {
        if self.length == 0 {
            return 0;
        }
        let mask = [0x02u8, 0x06, 0x1e, 0xf9][(self.ctl >> 6) as usize];
        if mask & (1 << self.phase) == 0 {
            0
        } else {
            if self.ctl & 16 != 0 {
                i32::from(self.ctl & 15)
            } else {
                i32::from(self.env)
            }
        }
    }
}

pub struct ExpansionAudio {
    bit: u8,
    ym: Option<Yamaha>,
    apu: Option<ApuState>,
    pulses: [Pulse; 2],
    enable: u8,
    pcm: u8,
    pcm_mode: u8,
    pcm_irq: bool,
    frame_clock: u64,
    cycle: u64,
    control: u8,
    last: i16,
    ssg_select: u8,
    multiply: [u8; 2],
}
impl ExpansionAudio {
    pub fn new(bit: u8) -> Self {
        assert!([8, 32, 64, 128].contains(&bit));
        let mut apu = if bit == 64 {
            Some(ApuState::default())
        } else {
            None
        };
        if let Some(a) = &mut apu {
            a.write_register(
                0x4017,
                0x40,
                0,
                0,
                TraceConfig::none(),
                &mut TraceSink::default(),
            );
        }
        Self {
            bit,
            ym: if bit == 32 || bit == 128 {
                Some(Yamaha::new(bit == 128))
            } else {
                None
            },
            apu,
            pulses: Default::default(),
            enable: 0,
            pcm: 0,
            pcm_mode: 0,
            pcm_irq: false,
            frame_clock: 0,
            cycle: 0,
            control: 0,
            last: 0,
            ssg_select: 0,
            multiply: [0; 2],
        }
    }
    pub fn set_epsm_rhythm_rom(&mut self, rom: &[u8]) -> Result<(), String> {
        if self.bit != 128 || rom.len() != 8192 {
            return Err("EPSM rhythm ROM requires exactly 8192 bytes".into());
        }
        unsafe { fcseq_ym_rom(self.ym.as_ref().unwrap().0.as_ptr(), rom.as_ptr()) };
        Ok(())
    }
    pub fn pcm_read(&mut self, addr: u16, value: u8) {
        if self.bit == 8 && self.pcm_mode & 1 != 0 && (0x8000..=0xbfff).contains(&addr) {
            self.pcm_value(value)
        }
    }
    fn pcm_value(&mut self, v: u8) {
        if v != 0 {
            self.pcm = v;
            self.pcm_irq = false
        } else if self.pcm_mode & 0x80 != 0 {
            self.pcm_irq = true
        }
    }
}
impl Mapper for ExpansionAudio {
    fn observe_cpu_read(&mut self, addr: u16, value: u8) {
        self.pcm_read(addr, value)
    }
    fn mapper_id(&self) -> u16 {
        match self.bit {
            8 => 5,
            32 => 69,
            64 => 256,
            _ => 0,
        }
    }
    fn mapper_name(&self) -> &'static str {
        match self.bit {
            8 => "MMC5 audio",
            32 => "SUNSOFT 5B",
            64 => "VT02 audio",
            _ => "EPSM YMF288",
        }
    }
    fn read_prg(
        &mut self,
        a: u16,
        _f: u64,
        _c: u64,
        _cfg: TraceConfig,
        _sink: &mut TraceSink,
    ) -> u8 {
        match self.bit {
            8 => match a {
                0x5010 => {
                    let v = if self.pcm_irq { 0x80 } else { 0 };
                    self.pcm_irq = false;
                    v
                }
                0x5015 => {
                    (self.pulses[0].length > 0) as u8 | (((self.pulses[1].length > 0) as u8) << 1)
                }
                0x5205 => (u16::from(self.multiply[0]) * u16::from(self.multiply[1])) as u8,
                0x5206 => ((u16::from(self.multiply[0]) * u16::from(self.multiply[1])) >> 8) as u8,
                _ => 0,
            },
            32 => self.ym.as_mut().unwrap().read(1),
            128 => self.ym.as_mut().unwrap().read((a & 3) as u8),
            64 if a == 0x4035 => self.apu.as_mut().unwrap().read_register(0x4015),
            _ => 0,
        }
    }
    fn write_prg(&mut self, a: u16, v: u8, f: u64, c: u64, cfg: TraceConfig, sink: &mut TraceSink) {
        match self.bit {
            8 => match a {
                0x5000..=0x5007 => {
                    let ch = ((a - 0x5000) / 4) as usize;
                    self.pulses[ch].write(a & 3, v, self.enable & (1 << ch) != 0)
                }
                0x5010 => {
                    self.pcm_mode = v;
                    if v & 128 == 0 {
                        self.pcm_irq = false
                    }
                }
                0x5011 if self.pcm_mode & 1 == 0 => self.pcm_value(v),
                0x5015 => {
                    self.enable = v;
                    for i in 0..2 {
                        if v & (1 << i) == 0 {
                            self.pulses[i].length = 0;
                        }
                    }
                }
                0x5205..=0x5206 => self.multiply[(a - 0x5205) as usize] = v,
                _ => {}
            },
            32 => {
                if a < 0xe000 {
                    self.ssg_select = v;
                    self.ym.as_mut().unwrap().write(0, v & 15)
                } else if self.ssg_select & 0xf0 == 0 {
                    self.ym.as_mut().unwrap().write(1, v)
                }
            }
            128 => self.ym.as_mut().unwrap().write((a & 3) as u8, v),
            64 => match a {
                0x4020..=0x402f => {
                    let mut value = v;
                    if a == 0x4020 || a == 0x4024 {
                        let duty = v >> 6;
                        value = (v & 63)
                            | (match duty {
                                1 => 2,
                                2 => 1,
                                _ => duty,
                            } << 6)
                    }
                    self.apu
                        .as_mut()
                        .unwrap()
                        .write_register(a - 32, value, f, c, cfg, sink)
                }
                0x4030 => self.control = v,
                0x4031 => self.pcm = v,
                0x4035 => {
                    self.apu
                        .as_mut()
                        .unwrap()
                        .write_register(0x4015, v & 15, f, c, cfg, sink)
                }
                _ => {}
            },
            _ => {}
        }
    }
    fn read_chr(&mut self, _: u16) -> u8 {
        0
    }
    fn write_chr(&mut self, _: u16, _: u8) {}
    fn irq_pending(&self) -> bool {
        self.pcm_irq || self.ym.as_ref().is_some_and(Yamaha::irq)
    }
    fn clock_expansion_audio(&mut self, c: u64) {
        self.cycle += c;
        if let Some(ym) = &mut self.ym {
            ym.clock(c)
        }
        if self.bit == 8 {
            for p in &mut self.pulses {
                p.clock(c)
            }
            self.frame_clock += c;
            while self.frame_clock >= 7457 {
                self.frame_clock -= 7457;
                for p in &mut self.pulses {
                    p.envelope()
                }
            }
        }
        if let Some(a) = &mut self.apu {
            a.clock_cpu_cycles_with_timed_expansion(
                c,
                0,
                self.cycle,
                TraceConfig::none(),
                &mut TraceSink::default(),
                |_, _| 0,
            );
            if let Some(s) = a.sample_buffer.last() {
                self.last = *s;
            }
            a.sample_buffer.clear();
        }
    }
    fn expansion_audio_sample(&mut self, rate: u32) -> i16 {
        let value = match self.bit {
            8 => {
                -(self.pulses.iter().map(Pulse::sample).sum::<i32>() * 340
                    + (i32::from(self.pcm) - 128) * 70)
            }
            64 => {
                self.apu.as_mut().unwrap().sample_rate = rate;
                if self.control & 8 != 0 {
                    i32::from(self.last) / if self.control & 128 != 0 { 2 } else { 1 }
                } else {
                    0
                }
            }
            _ => self.ym.as_ref().unwrap().sample(),
        };
        value.clamp(-32768, 32767) as i16
    }
    fn debug_state(&self) -> MapperDebugState {
        let mut s = NromMapper::new(vec![0; 32768], vec![], Mirroring::Horizontal).debug_state();
        s.name = self.mapper_name().into();
        s
    }
    fn snapshot_bytes(&self) -> Vec<u8> {
        vec![]
    }
}
