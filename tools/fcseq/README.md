# FCSEQ 0.4.1 — user manual

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

By **DAISUKE OBA**. A general-purpose Famicom music sequencer built with **plita**. GUI, CLI and Python bindings share the same project model, audio renderer and exporters. The GUI starts with an empty project. No game ROM or BIOS is required. The interface uses English labels; this manual is available in nine languages.

## Download and launch

Download the complete ZIP for your platform from [downloads/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads): Windows x86_64, Linux x86_64, macOS Apple Silicon (ARM64), or macOS Intel (x86_64). Extract the whole archive, retaining assets, notices and source archives. Windows: open `fcseq.exe`; Unix: run `./fcseq`. Use `fcseq-cli` for terminal commands (append `.exe` on Windows). Individual executable files are also in [bin/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/bin), but redistribution requires the accompanying notices.

Windows 10/11 uses Direct3D 12; macOS uses Metal; Linux uses Vulkan/OpenGL with X11/Wayland and ALSA. Linux binaries are built on Ubuntu 24.04; older distributions may need a source build. On Ubuntu install runtime libraries with `sudo apt install libasound2t64 libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`. A compatible graphics driver is required for the GUI and an audio device for live playback. WAV export needs no audio device. Binaries are unsigned; macOS builds are not notarized. If blocked after downloading, review the source and hashes, then use the operating system's normal approval mechanism. Unix ZIP extraction may require `chmod +x fcseq fcseq-cli`.

Windows also requires the [Microsoft Visual C++ x64 Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) (the executables import `VCRUNTIME140.dll`). If it is missing, install the current supported x64 package from Microsoft. GUI/CLI do not link to Python; Python is needed only for the optional bindings and wrapper.

## Create and edit a song

Choose the chip in SOUND. Base 2A03 has Pulse 1/2, Triangle, Noise and DPCM (5 voices). VRC6 adds two pulse voices and saw (8 total); VRC7 adds six FM voices (11); FDS adds one wavetable voice (6); N163 adds up to eight wavetable voices (13). Only one expansion configuration is active per project. Changing chips or reducing channels refuses operations that would discard notes.

Double-click a lane to add a note; drag its body to move/change pitch and its right edge to resize. Ctrl/Shift-click or drag a rectangle to select notes. Ctrl+A selects the current lane. Ctrl+C/X/V copies/cuts/pastes at the cursor; Ctrl+D duplicates; Delete removes; Ctrl+Z/Y undoes/redoes. Arrows move or transpose; Shift+Up/Down changes an octave. G changes grid, Q quantizes, F fits, Home returns to the start, +/- zooms; Shift+wheel scrolls time. UP/DOWN scrolls channels. Space starts/stops; LOOP repeats; AUDITION previews selected notes. MUTE/SOLO also affect exports. SONG manages up to 16 sections of 256 bars. Click names, BPM or bar count to edit. Ctrl+N/O/S/Shift+S creates/opens/saves/saves as. Invalid overlapping or out-of-range edits are rejected atomically. Recovery files are in `out/recovery_<process-id>.fcq` beside the executable; reopen and Save As to recover.

## Instruments and waveforms

Open SOUND > Instrument / waveform editor or INSTRUMENT / WAVEFORM. Select notes on that channel, edit parameters and use AUDITION/UNDO/REDO. Pulse: 4 duty values; VRC6 pulse: 8. Triangle has a fixed 32-step waveform. Noise has 16 periods and long/short mode. Pulse/noise volume is 0–15; decay 0 holds volume. VRC6 saw uses a 6-bit accumulator increment (0–63), whose overflow changes timbre. These chips have no freely writable waveform RAM.

FDS has one shared 64-sample, 6-bit editable wave. N163 has 4-bit waves, initially four of 32 samples; notes choose a wave slot. Lengths are multiples of 4. Shared capacity is `256 - 16 × active_channels` samples (128 at eight voices); shorten another wave or reduce channels before exceeding capacity. Paint by dragging; use sine/pulse/saw/triangle presets, invert and rotate. Pitch, RAM and disabled-channel limits are validated. VRC7 custom patch 0 exposes 24 parameters (operator multipliers, ADSR, feedback, level, key scaling, tremolo/vibrato, sine/rectified sine) or eight raw bytes. Presets 1–15 are synthetic approximations, not instrument ROM dumps. FDS modulation-table/envelope editing is not provided. Emulation is not a guarantee of analog hardware equivalence.

## DPCM samples

SAMPLES or SOUND > Import audio file accepts WAV, FLAC, Ogg Vorbis and MP3; files can be dropped. Set START/END (ms), GAIN, NORMALIZE, REVERSE and RATE 0–15, then AUDITION DPCM. EXPORT .DMC saves a sample. Place notes on the DPCM lane; choose SAMPLE and note playback RATE in the inspector. Original PCM is retained for reversible processing, up to 60 seconds, with stereo mixed to mono. Conversion removes DC and resamples with low-pass filtering. Each sample is `16n+1` bytes, at most 4081; at most 64 samples and 16320 bank bytes (8192 for FDS). Oversize edits are rejected. Conversion rate and note playback rate are different settings. Sample looping and automatic pitch correction are not implemented.

## Files, NSF/NSFe and C export

Save editable projects as `.fcq`. Rename old `.fcseq` files to `.fcq`; note projects retain the `fcseq-2` JSON format and legacy `fcseq-1` imports are supported. Imported NSF playback documents use `fcseq-3`.

FILE > Import NSF, OPEN, dropping a file or a launch argument opens NSF/NSFe. Choose a song with PREV/NEXT or its number, PLAY/STOP, LOOP and duration (1–600 seconds). Timing metadata is respected; without it the default is 30 seconds. Audio is rendered before playback and can be cancelled. Save as FCQ embeds the original music, selection and duration; this is playback import, not automatic note transcription. Re-export preserves the original container bytes.

Export NSF converts each edited section into a subsong with its chip settings and DPCM data. NSF export also creates an NSFe companion with song durations and zero fade; use NSFe when a player ignores NSF timing metadata. Playback stops at section end; editor LOOP is separate. Supported input: NSF version 1 and NSFe, NTSC or dual-region with NTSC, unbanked or 4 KiB banked, 2A03/DPCM, VRC6, VRC7, FDS and N163. PAL-only, NSF2 execution extensions, MMC5, Sunsoft 5B and VT02 are rejected. Runaway INIT/PLAY code is bounded.

Export C + DPCM writes `score.c`, `fcseq_score.h`, `fcseq_player.c`, section binaries, `.dmc`, `dpcm_bank.bin` and `fcseq_export.json`. It does not generate a complete game ROM: arrange mapper support and CPU-visible code/data/DPCM addresses yourself. VRC6 assumes mapper 24 (mapper 26 requires address-line translation), VRC7 mapper 85, N163 mapper 19, or FDS. Define `FCSEQ_KITAQFC` before including the driver in KITAQFC; `FCSEQ_WRITE(address, value)` can connect a custom register writer.

## CLI and Python

The commands below run from the extracted package. Sections/channels/notes/sample indices start at 0; NSF song numbers start at 1. Python 3.10+ uses the bundled abi3 extension from the same folder; match OS/architecture. The CLI and GUI run without Python. `fcseq_native.pyi` lists the complete API. CLI note operations include move, transpose, resize, quantize, duplicate and delete; waveform editing is available through GUI/Python.

```sh
./fcseq-cli new song.fcq n163
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf song.fcq song.nsf
./fcseq-cli export-nsfe song.fcq song.nsfe
./fcseq-cli import-nsf song.nsf imported.fcq 1 60
./fcseq-cli nsf-info song.nsfe
./fcseq-cli nsf-render song.nsfe preview.wav 1 10
./fcseq-cli play-nsf song.nsfe 1 30
./fcseq-cli import-audio song.fcq voice.wav with_voice.fcq
./fcseq-cli export-dpcm with_voice.fcq 0 voice.dmc
./fcseq-cli edit song.fcq edited.fcq 0 0 0 transpose 12
./fcseq-cli export-kitaqfc song.fcq export
python fcseq.py nsf-render song.nsf preview.wav --song 1 --seconds 10
```

```python
import fcseq_native
c = fcseq_native.Controller()
c.set_expansion('n163')
c.set_n163_channels(8)
c.set_n163_wave(0, [i // 2 for i in range(32)])
c.add_note(0, 5, 0, 8, 69)
c.save_project('example.fcq')
c.export_nsf('example.nsf')
c.render_wav('example.wav')
```

## Build and verification

From this source folder, use Rust 1.93.1 and the commands below. All local library sources are in vendor/. Linux needs the development packages listed in `.github/workflows/fcseq.yml` (at the repository root); macOS needs Xcode Command Line Tools, Windows needs MSVC Build Tools. Python is optional. The verification scripts exercise native CLI/Python, WAV, NSF/NSFe round trips and GUI startup; details and limits are in VERIFICATION.md. Interactive mouse/keyboard and physical hardware listening are not certified.

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python --lib
python -c "import shutil,sys; from pathlib import Path; n={'win32':'fcseq_native.dll','darwin':'libfcseq_native.dylib'}.get(sys.platform,'libfcseq_native.so'); shutil.copy2(Path('target/release')/n,'fcseq_native.pyd' if sys.platform=='win32' else 'fcseq_native.so')"
```

## License

FCSEQ is MIT, Copyright (c) 2026 DAISUKE OBA. Dependencies retain their licenses. Keep LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ and third_party_sources/ when redistributing. MPL-2.0 Symphonia source is included; font/icon notices are retained. Imported music and samples have separate rights.

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
