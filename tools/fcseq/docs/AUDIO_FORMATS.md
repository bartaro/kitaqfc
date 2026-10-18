# Audio formats and implementation boundaries

FCSEQ 0.5.0 uses the same score model, register compiler and KUROSAKI-based renderer
for the editor, CLI and Python bindings. Editable scores use `.fcq`. Imported NSF
programs remain executable music data; importing them does not recover source notes.

## Expansion audio

| Configuration | Added channels | Engine and controls |
|---|---:|---|
| VRC6 | 3 | Two pulse generators and one saw accumulator |
| VRC7 | 6 | FM operators; synthetic preset approximations and custom patch 0 |
| FDS | 1 | Shared 64 × 6-bit waveform; modulation-table/envelope editor not included |
| N163 | 1–8 | Shared 4-bit wave RAM and per-note waveform selection |
| MMC5 | 3 | Two pulses without sweep, envelopes/length counters, direct/read-mode PCM |
| SUNSOFT 5B | 3 | YM2149 SSG tone/noise/envelope engine |
| EPSM | 15 | YMF288: three SSG, six four-operator FM and six rhythm channels |
| VT02 | 5 | Second APU, physical duty encoding and 8-bit PCM mode |

MMC5 memory-mapped status, PCM read-mode/IRQ flag, multiplier and ExRAM access
are handled by the NSF host. The score editor emits direct PCM writes, rather
than relying on read-triggered playback. Imported NSF code can use the read mode.

5B and EPSM use the BSD-3-Clause ymfm implementation pinned in
`vendor/kurosaki-core/vendor/ymfm/UPSTREAM.json`. FCSEQ's C++ adapter supplies clock,
timer and memory callbacks. 5B uses half the NTSC CPU clock; EPSM uses 8 MHz.
SSG channels share a noise period and envelope generator. Register writes from a
later note update those shared generators. EPSM patch slots contain the four
operators plus algorithm/feedback/modulation fields. Preview and WAV are mono;
pan bits remain in the exported stream. Modulation uses the chip's shared LFO
at rate index 4. AM/PM sensitivity is editable per patch; the global LFO rate
does not currently have an editor control. Zero sensitivity disables modulation
for that patch.

EPSM percussion is synthesized from original formulas in `ymfm_bridge.cpp`, then
encoded into an ADPCM bank. No commercial Yamaha ROM data is distributed. These
sounds are approximations: physical EPSM devices use their internal rhythm ROM.
The engine's optional ROM setter is an internal integration interface, not a GUI
sample-import option.

VT02 implements the second APU registers, reversed physical pulse-duty encoding,
the $4030 mixer/sample mode controls and raw 8-bit DMA from CPU-visible memory.
The editor emits 4 kHz PCM DAC writes, with the same import, trim, gain, normalize
and reverse controls as DPCM. MMC5 PCM uses this stream rate too. Pitch alters the
resampling speed. DPCM and PCM share the VT02 sample unit and cannot both be used
within one section. Hardware-specific analog mixing and half-volume response are
approximations; these are software models, not measurements of physical chips.

The FDS and N163 graph can connect quantized values with straight segments or a
smooth monotone cubic curve. Interpolation affects only the view: samples saved
in FCQ, sent to the engine and exported to NSF remain exact integer values. The
curve passes through the sample points and does not overshoot adjacent values.

## NSF, NSF2 and NSFe

The reader accepts NSF versions 1 and 2 and NSFe, including its NSF2 feature chunk.
It supports unbanked programs, 4 KiB bank switching and the eight expansion flag
bits. NTSC is the execution clock; dual-region music runs as NTSC, and PAL-only
files are rejected. FDS BIOS is neither bundled nor required for FCSEQ's exports.
Imported code requiring a BIOS or unsupported mapper behavior is not guaranteed
to run. Multiple expansion flags can be played by the host; editable score projects
select one expansion configuration at a time.

NSF2 timer IRQ registers, background/non-returning INIT and suppressed PLAY are
implemented. The timer period is the reload value plus one CPU cycle. Background
INIT receives the NSF2 handshake values; the host interrupts it with a PLAY wrapper
and restores the interrupted CPU state. Ordinary INIT and PLAY calls are bounded
so runaway code produces an error. Tests cover IRQ-driven output, no-PLAY music,
background execution and deterministic rendering across sample chunks.

EPSM occupies $401C–$401F, overlapping the NSF2 timer. A file requesting both EPSM
and NSF2 timer IRQ is rejected instead of silently routing writes to the wrong
device. EPSM timer callbacks can interrupt actively executing background code;
ordinary idle playback does not provide an additional autonomous EPSM IRQ runner.

FCSEQ exports NSF2 with a version-2 header and no advanced execution flags; it
uses an ordinary INIT/PLAY driver. Files that need advanced NSF2 execution can
be imported and played, but the editor does not author arbitrary IRQ programs.
NSF/NSF2 export also writes an NSFe companion with section durations and zero
fade. Each section is a subsong, ending at its section boundary. Re-exporting
an imported project to the same container preserves its original bytes.

PCM scores use a 250 μs PLAY interval (4 kHz); other scores use the NTSC interval.
The C driver reports `FCSEQ_TICK_RATE_HZ` and `ticks_per_row` for the same timing.
The host must supply that call rate and keep code/data accessible to the CPU.
High write density or unusually large imported programs can exceed the bounded
CPU execution budget; export is not a promise that every arrangement fits every
physical cartridge or third-party player.

## Public references

Implementation was checked against [NSF](https://www.nesdev.org/wiki/NSF),
[NSF2](https://www.nesdev.org/wiki/NSF2), [NSFe](https://www.nesdev.org/wiki/NSFe),
[MMC5 audio](https://www.nesdev.org/wiki/MMC5_audio),
[Sunsoft 5B audio](https://www.nesdev.org/wiki/Sunsoft_5B_audio) and
[VT02+ sound](https://www.nesdev.org/wiki/VT02%2B_Sound) descriptions, and the
[EPSM hardware project](https://github.com/Perkka2/EPSM).
The [ymfm source and licence](https://github.com/aaronsgiles/ymfm) are retained
with attribution; the referenced documentation is not bundled as source code.
