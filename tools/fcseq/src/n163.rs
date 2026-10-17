//! FCSEQ uses the same cycle-clocked N163 implementation as KUROSAKI.
use kurosaki_core::{
    mapper::{Mapper, MapperDebugState},
    namco163::Namco163Mapper,
    Mirroring, TraceConfig, TraceSink,
};

pub struct N163(Namco163Mapper);
impl Default for N163 {
    fn default() -> Self {
        let mut chip = Namco163Mapper::new(vec![0; 32768], vec![], Mirroring::Horizontal, false);
        chip.use_averaged_audio();
        Self(chip)
    }
}
impl Mapper for N163 {
    fn mapper_id(&self) -> u16 {
        19
    }
    fn mapper_name(&self) -> &'static str {
        "N163 audio"
    }
    fn read_prg(
        &mut self,
        addr: u16,
        frame: u64,
        cycle: u64,
        cfg: TraceConfig,
        sink: &mut TraceSink,
    ) -> u8 {
        self.0.read_prg(addr, frame, cycle, cfg, sink)
    }
    fn write_prg(
        &mut self,
        addr: u16,
        value: u8,
        frame: u64,
        cycle: u64,
        cfg: TraceConfig,
        sink: &mut TraceSink,
    ) {
        self.0.write_prg(addr, value, frame, cycle, cfg, sink)
    }
    fn read_chr(&mut self, _: u16) -> u8 {
        0
    }
    fn write_chr(&mut self, _: u16, _: u8) {}
    fn clock_expansion_audio(&mut self, cycles: u64) {
        self.0.clock_expansion_audio(cycles);
    }
    fn expansion_audio_sample(&mut self, rate: u32) -> i16 {
        self.0.expansion_audio_sample(rate)
    }
    fn debug_state(&self) -> MapperDebugState {
        self.0.debug_state()
    }
    fn snapshot_bytes(&self) -> Vec<u8> {
        self.0.snapshot_bytes()
    }
}
