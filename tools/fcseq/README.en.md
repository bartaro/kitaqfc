# FCSEQ 0.5.0

[English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [Français](README.fr.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Português](README.pt-BR.md)

FCSEQ is a Famicom music editor built with plita. Start with an empty project, write notes, shape your instruments, and listen as you work. The desktop editor, command-line tool and Python bindings use the same project format and audio engine.

## Install and choose a language

Download the ZIP for your computer from [downloads](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads), then extract the whole archive. Open `fcseq.exe` on Windows or `./fcseq` on Linux and macOS. Packages are available for Windows and Linux x86_64, and for both Apple Silicon and Intel Macs. Keep the fonts and licence files beside the program. If necessary, run `chmod +x fcseq fcseq-cli` after extracting a Unix ZIP.

The interface supports English and Japanese. Choose **SOUND → 日本語 / English** to switch; the NSF player has the same switch on its toolbar. FCSEQ remembers your choice. On first launch it uses Japanese on a Japanese system and English otherwise. You can also set `FCSEQ_LANGUAGE=ja` or `en`. Noto Sans JP is included so Japanese text works on all supported systems.

Windows needs a Direct3D 12 graphics driver and the [Microsoft Visual C++ x64 runtime](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist). macOS uses Metal. Linux packages are built on Ubuntu 24.04 and use ALSA with X11 or Wayland and Vulkan/OpenGL; older distributions may need a source build. Live playback needs an audio device, but WAV export does not. The binaries are unsigned, and the Mac apps are not notarized.

## Write and edit notes

Choose a sound chip from **SOUND**. Double-click an empty part of a track to add a note. Drag the note to move it or change its pitch; drag its right edge to change its length. Each hardware channel is monophonic, so overlapping notes on one track are rejected.

For detailed editing, double-click the track name or press **E**. The Key Editor opens that track with a piano keyboard on the left. The vertical grid shows semitones and the horizontal grid shows time. Click a piano key to audition it. Scroll the wheel to change the visible pitch range, use the key-range −/+ buttons to zoom, and hold Shift while scrolling to move along the timeline. Use the track arrows to open another track. Press E again to return to all tracks. Noise, DPCM and rhythm tracks show their available sounds or rates instead of a chromatic instrument range.

Use Ctrl/Shift-click or drag a rectangle to select notes. Ctrl+A selects the current track; Ctrl+C/X/V copies, cuts or pastes at the cursor. Ctrl+D duplicates and Delete removes the selection. Ctrl+Z/Y undoes or redoes changes. Arrow keys move or transpose; Shift+Up/Down moves by an octave. G changes the grid, Q aligns notes to it, F fits the section, Home returns to the start, and +/− zooms horizontally.

Space starts or stops playback. **LOOP** repeats the current section; **AUDITION** plays the selection. Mute and solo settings also affect exports. Use **SONG** to add, rename or reorder sections. A project can hold 16 sections of up to 256 bars each. Click the tempo or bar count to edit it. Ctrl+N/O/S/Shift+S creates, opens, saves or saves under a new name. Recovery files are written to `out/recovery_<process-id>.fcq` beside the program; open one and use Save As to recover your work.

## Choose a sound chip

Every project includes the five 2A03 channels: two pulses, triangle, noise and DPCM. One expansion configuration can be active at a time.

| Expansion | Additional voices | Total tracks |
|---|---|---:|
| VRC6 | 2 pulse, 1 saw | 8 |
| VRC7 | 6 FM | 11 |
| FDS | 1 wavetable | 6 |
| N163 | Up to 8 wavetable | 13 |
| MMC5 | 2 pulse, 1 PCM | 8 |
| SUNSOFT 5B | 3 SSG, with shared noise and envelope | 8 |
| EPSM | 3 SSG, 6 FM, 6 rhythm | 20 |
| VT02 | 2 pulse, triangle, noise, PCM | 10 |

FCSEQ will not switch chips or reduce active channels if doing so would discard notes. Clear the affected tracks first, or start a new project.

## Shape instruments and waveforms

Select notes, then open **INSTRUMENT / WAVEFORM**. Pulse instruments offer duty-cycle settings, and noise instruments offer period and long/short mode. The VRC6 saw has an accumulator-rate control. These are hardware parameters; those channels do not have freely editable waveform memory.

FDS provides one shared 64-point, 6-bit waveform. N163 provides 4-bit waveforms; each note selects a waveform slot. Draw by dragging across the graph, or start from a sine, pulse, saw or triangle preset. You can invert or rotate the waveform. **CURVE / LINE** switches between a smooth curve and straight segments. Visible points are the values saved to the sound chip; the smooth curve is a display aid. The sine preset therefore looks smooth while its quantized sample values remain precise. Hover over a point to see its index and value.

N163 waveform lengths are multiples of four. All voices share `256 − 16 × active_channels` samples of wave memory, leaving 128 samples at eight voices. Shorten another waveform or reduce the channel count if you need more room. FDS modulation-table and hardware-envelope editing are not included.

VRC7 custom patch 0 exposes its operator and envelope parameters; presets 1–15 are synthetic approximations. EPSM has 16 editable FM patches, each with four operators, algorithm, feedback, envelope and modulation settings. Choose an operator to edit its parameters. Changes affect every note using that patch. EPSM FM and rhythm voices also have pan settings. The current preview and WAV output are mono; the exported registers retain the pan settings.

5B and EPSM SSG voices offer tone, noise and envelope controls. Their three SSG channels share the noise period and envelope generator, so a later setting also affects other notes using that generator. EPSM rhythm preview uses original synthesized percussion, not a Yamaha ROM dump; an EPSM device plays its own built-in drum sounds.

## Turn audio into an instrument

Open **SAMPLES** and import WAV, FLAC, Ogg Vorbis or MP3. You can also drop a file into the window. Adjust the start and end, gain, normalization and reverse settings. Original audio is kept, so these changes are reversible. Stereo files are mixed to mono, and source files may contain up to 60 seconds of audio.

Place a note on DPCM and choose its sample and playback rate. **AUDITION DPCM** previews the encoded result; **EXPORT .DMC** saves it. DPCM conversion removes DC and resamples with a low-pass filter. A sample must fit in 4,081 bytes; the shared bank holds 16,320 bytes, or 8,192 with FDS. Trim the region or lower the conversion rate if it is too large. The sample's conversion rate and a note's playback rate are separate settings.

MMC5 and VT02 PCM tracks use the same imported audio and processing controls. They stream 8-bit samples at 4 kHz; note pitch changes playback speed. PCM does not use the DPCM byte-size limit. VT02 DPCM and PCM share a hardware sample unit, so use only one of those sample modes within a section. Sample looping and automatic pitch correction are not provided.

## Save, exchange and export music

Save editable projects as **`.fcq`**. Older `.fcseq` projects can be renamed to `.fcq` before opening. Note projects retain the `fcseq-2` JSON format; older `fcseq-1` projects are also accepted. Playback projects that embed imported music use `fcseq-3`.

Open NSF, NSF2 or NSFe through FILE, drag and drop, or a launch argument. Select the subsong and playback duration, then press PLAY. Timing metadata is used when present; otherwise playback defaults to 30 seconds, adjustable from 1 to 600. Saving as FCQ embeds the original music and playback settings. Import retains executable music data; it does not transcribe arbitrary NSF files into editable notes. Re-export to the same container preserves the original bytes.

Export to `.nsf`, `.nsf2` or `.nsfe` by choosing that extension in the export dialog. Each section becomes a subsong. NSF and NSF2 exports also create an NSFe companion containing track durations and zero fade. Playback ends at the section boundary; the editor's loop switch is separate.

The player supports NTSC and dual-region files using NTSC timing, 4 KiB banking, all chips listed above, and NSF2 timer IRQs, background INIT and suppressed PLAY. PAL-only files are rejected. NSF2 timer ports overlap EPSM's ports, so files combining EPSM with the NSF2 timer-IRQ flag are rejected explicitly. See [audio-format details](docs/AUDIO_FORMATS.md) for the execution and hardware boundaries.

**Export C + DPCM** writes the register streams, C driver, header, sample bank and placement report. It is not a complete game ROM. Keep the driver and data CPU-visible and configure the target mapper. Call `fcseq_tick` at `FCSEQ_TICK_RATE_HZ`: normally the NTSC frame rate, or 4,000 Hz for a project using PCM. The exported JSON reports that rate. Define `FCSEQ_KITAQFC` for the KITAQFC compiler; `FCSEQ_WRITE(address, value)` can connect a custom register writer.

## CLI and Python

Run `fcseq-cli help` for commands. Append `.exe` on Windows. Section, channel, note, patch and sample indices start at 0; NSF song numbers start at 1.

```sh
./fcseq-cli new song.fcq epsm
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf2 song.fcq song.nsf2
./fcseq-cli import-nsf music.nsf2 imported.fcq 1 60
./fcseq-cli nsf-render music.nsfe preview.wav 1 10
./fcseq-cli set-epsm-parameter song.fcq edited.fcq 0 44 4
./fcseq-cli export-kitaqfc song.fcq export
```

Python 3.10 or later can load the bundled native extension when the package matches the operating system and architecture. The GUI and CLI do not require Python. [fcseq_native.pyi](fcseq_native.pyi) lists the API; `set_sound` accepts the same sound fields saved in FCQ, including SSG envelope and EPSM pan settings.

```python
import fcseq_native
c = fcseq_native.Controller()
c.set_expansion('epsm')
c.set_epsm_parameter(0, 44, 4)  # patch 0, algorithm 4
c.add_note(0, 8, 0, 8, 69)    # first EPSM FM track
c.set_sound(0, 8, 0, '{"instrument": 0}')
c.save_project('example.fcq')
c.export_nsf('example.nsf2')
c.render_wav('example.wav')
```

## Build and licences

Use Rust 1.93.1 and a C++14 compiler. Build with `cargo build --locked --release`; run `cargo test --locked --tests` to test. The command-line-only build uses `--no-default-features --bin fcseq-cli`. Build the Python extension with `--features python --lib`. Windows needs MSVC Build Tools, macOS needs Xcode Command Line Tools, and Linux needs the development libraries listed in the repository's `.github/workflows/fcseq.yml`.

FCSEQ's application code is MIT-licensed, Copyright (c) 2026 DAISUKE OBA. Libraries and fonts retain their own licences, including BSD-3-Clause ymfm, MPL-2.0 Symphonia and OFL fonts. Keep LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ and third_party_sources/ when redistributing. No game ROM, FDS BIOS or commercial instrument-ROM dump is included. Imported songs and samples retain their own rights. See [notices](THIRD_PARTY_NOTICES.md), [licence audit](LICENSE_AUDIT.json) and [verification](VERIFICATION.md).

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
