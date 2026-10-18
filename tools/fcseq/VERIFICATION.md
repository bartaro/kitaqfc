# FCSEQ 0.5.0 verification

Windows local release checks: 49 Rust tests and 14 Python tests passed.
The suite covers new expansion voices, NSF2 IRQ/background execution, quantized
waveform curves and single-track key editing. All 22 GUI draw-list scenes were
generated; Japanese keyboard, FDS curve and EPSM panels were visually inspected.

The 0.5.0 native distribution workflow is pending. Existing bin/ and downloads/
packages remain 0.4.2 until the verified 0.5.0 binaries are installed. Do not use
old binary receipts as verification of this source revision.

These checks are software/emulator evidence, not physical hardware measurements,
human mouse/keyboard testing or analog listening. See docs/AUDIO_FORMATS.md for
the supported modes and limitations, and LICENSE_AUDIT.json for dependency terms.
