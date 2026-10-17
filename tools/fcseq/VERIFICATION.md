# FCSEQ 0.4.1 verification

All four native builds and verification jobs passed: [GitHub Actions run](https://github.com/bartaro/kitaqfc/actions/runs/39060150415).
Binary source commit: `bd3e8f41ceac0d472804745e9e8775741c345afe`. Toolchain: Rust 1.93.1, CPython 3.10 abi3.
GUI and CLI are built without the Python feature; the optional extension is built separately.
The following counts come from execution on the stated native operating system.

| Platform | Rust tests | Python tests | Audio configurations / combined voices | Native GUI startup |
|---|---:|---:|---|---|
| windows-x86_64 | 37 | 13 | 5 / 43 | passed |
| linux-x86_64 | 37 | 13 | 5 / 43 | passed |
| macos-arm64 | 37 | 13 | 5 / 43 | passed |
| macos-x86_64 | 37 | 13 | 5 / 43 | passed |

Rust coverage includes editor selection/history, model validation, waveform/FM
parameters, every exported voice, NSF/NSFe timing, malformed input and bounded
CPU execution. The native package smoke exercises five complete audio setups
(2A03, VRC6, VRC7, FDS and N163), including imported/converted DPCM samples,
non-silent WAV rendering, NSF/NSFe export and byte-preserving NSF import/export.
Python tests exercise the actual abi3 module from each native package.
Fourteen GUI draw-list scenarios and native window/GPU creation with 16 event
ticks and automatic exit also passed. Linux window testing runs under Xvfb.

Receipts are in `verification/<platform>/`; binary hashes, sizes and file formats
are in NATIVE_BINARIES.json. SHA256SUMS includes each executable/extension and
complete ZIP. Source hashes are in SOURCE_SNAPSHOT.json. Windows binaries were
identified as PE x86_64, Linux as ELF x86_64, and macOS as native Mach-O ARM64/x86_64.

These are emulator/software checks, not physical Famicom/audio-hardware proof.
CI does not certify analog listening, physical mouse/keyboard interaction or a
live audio output device. WAV rendering and GUI startup are tested independently
of physical audio playback. macOS packages are not notarized; Windows binaries
are not publisher-signed. Linux binaries target Ubuntu 24.04's runtime baseline.

License audit: 279 Cargo dependencies across the four targets with Python enabled,
no missing notices, four unmodified MPL-2.0 Symphonia source archives, preserved
font/icon attribution and the matching Rust standard-library license inventory.
FCSEQ application copyright is (c) 2026 DAISUKE OBA under MIT. Third-party licenses
remain applicable; this audit is not a legal warranty.

The original local Windows release build ran out of disk space and was excluded
from passing evidence. Earlier CI failures in the macOS Python linker setup and
the smoke-test Noise fixture were corrected before the successful run above.
