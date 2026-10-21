# FCSEQ 0.5.1 verification

All four native builds and verification jobs passed: [GitHub Actions run](https://github.com/bartaro/kitaqfc/actions/runs/39518117575).
Binary source commit: `e07b06d07f4017bbf5fb234c691d3dd4f4b71571`. Toolchain: Rust 1.93.1, CPython 3.10 abi3.
GUI and CLI are built without the Python feature; the optional extension is built separately.
The following counts come from execution on the stated native operating system.

| Platform | Rust tests | Python tests | Audio configurations / combined voices | Native GUI startup |
|---|---:|---:|---|---|
| windows-x86_64 | 51 | 15 | 9 / 88 | passed |
| linux-x86_64 | 51 | 15 | 9 / 88 | passed |
| macos-arm64 | 51 | 15 | 9 / 88 | passed |
| macos-x86_64 | 51 | 15 | 9 / 88 | passed |

Rust coverage includes native plita named-key input, section-name cancellation,
compact keyboard/grid hit testing, editor selection/history, single-track piano-key editing,
Japanese text input, waveform curves without overshoot, instrument parameters,
every expansion voice, NSF/NSF2/NSFe execution and bounded malformed input.
5B and EPSM pitch are checked against A440. NSF2 tests exercise timer IRQ,
background INIT, suppressed PLAY and deterministic chunked rendering.

Each native package exercises nine complete configurations (2A03, VRC6, VRC7,
FDS, N163, MMC5, SUNSOFT 5B, EPSM and VT02), imported audio, WAV rendering,
NSF/NSF2 export and byte-preserving import/export. VT02's combined fixture uses
PCM because DPCM and PCM share a sample unit. DPCM conversion and playback are
covered by the common 2A03 tests; VT02 raw DMA has a separate CPU-RAM fixture.
Python tests load the actual abi3 module built on each platform.

Twenty-two GUI draw-list scenarios include Japanese keyboard, waveform and
EPSM panels. Native window/GPU startup runs for 16 event ticks and exits
automatically; Linux uses Xvfb. Local Japanese draw-list renders were inspected.
See verification/<platform>/ for receipts, NATIVE_BINARIES.json for binary hashes,
SHA256SUMS for distribution hashes and SOURCE_SNAPSHOT.json for source hashes.

These are emulator/software checks, not physical Famicom or analog-listening
evidence. Human mouse/keyboard interaction and live audio devices are not certified
by CI. macOS packages are not notarized and Windows binaries are not signed.
Linux builds use Ubuntu 24.04's runtime baseline. AUDIO_FORMATS documents the
NTSC, EPSM timer-port, synthesized-rhythm and mono-output boundaries.

The licence inventory retains notices for 279 Cargo dependencies, BSD-3-Clause
ymfm, OFL fonts including Noto Sans JP, four unmodified MPL-2.0 Symphonia source
archives and the matching Rust standard-library copyright inventory.
FCSEQ application code remains MIT, Copyright (c) 2026 DAISUKE OBA. Third-party
terms remain applicable. This inventory is not a legal warranty.
