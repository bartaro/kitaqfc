# FCHUA 0.1.0 Verification

Verified on Windows x64 on 2026-10-11. The standalone distribution contains no
PLITA, window-system or GPU dependency. Python support is an optional Cargo
feature; the CLI executable does not require Python.

## Automated Checks

- 14 shared-core tests and 5 CLI integration tests passed with the `python` feature.
- The release wheel was installed into an isolated directory without dependencies.
- All 3 tests in `tests/python_api.py` passed against that installed wheel.
- CLI and both headless/desktop Python builds produced identical `.fch` documents,
  PNG previews, batch-edit results and raw asset bytes from the same 64x64 fixture.
- Invalid command batches left the original document unchanged.
- Multiple metasprite frames were present in the generated C output.
- The existing native `kitaqfc-asset-pack` accepted the generated manifest;
  its 8192-byte CHR output matched the editor export byte for byte.
- Dependency notices cover 52 normal/build packages, including optional PyO3.
  The wheel includes those license files and passed ZIP integrity checks.
- The nine manuals contain matching workflows and valid local language links.
- `SHA256SUMS` identifies the supplied Windows CLI and CPython abi3 wheel.

```sh
cargo fmt -- --check
cargo test --locked --features python
cargo build --release --locked --bin fchua
python -m pip install python-dist/fchua-0.1.0-cp310-abi3-win_amd64.whl
python tests/python_api.py
```

The supplied CLI and wheel use the static Windows C runtime. The wheel targets
CPython 3.10+ on Windows x64. Linux/macOS source builds are supported by the
package structure but were not executed for this release; no binaries for those
platforms are claimed here.

## Local Desktop Checks

The separate local PLITA desktop build passed 18 core/UI tests and 5 CLI tests.
Draw-list snapshots were inspected at 900x600, 1024x600, 1360x820 and 1920x1080.
Tests cover shared button release behavior, menu hit blocking, scrolling,
selection without dirtying the document, and one undo step per drawing stroke.

Native-window click/dialog automation was unavailable. Software rendering and
event tests are not a substitute for an end-to-end native-window interaction
test. Hardware/console and full-game emulator validation are outside these checks.
