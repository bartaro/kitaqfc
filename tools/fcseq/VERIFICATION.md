# FCSEQ 0.4.1 verification

The source package includes a native build/verification workflow at
`.github/workflows/fcseq.yml` in the repository root. Binary publication follows
successful native tests; receipts will be recorded here with each distribution.

The test scope includes the shared Rust model/editor, Python bindings, five
audio configurations (43 combined voices including DPCM), non-silent WAV output,
NSF/NSFe export, embedded import and byte-preserving re-export, fourteen GUI
draw-list scenarios, and actual native window/GPU startup and shutdown.
Audio device playback and physical Famicom/expansion hardware are not part of
CI verification. Automated startup does not certify human mouse/keyboard use.

`LICENSE_AUDIT.json` covers 279 dependencies across four native targets with
Python enabled. Required notices and four unmodified Symphonia source archives
are supplied. This is a source/license audit, not a legal warranty.

The initial local Windows release build was interrupted by disk exhaustion
before completion; it is not counted as a successful native verification.
