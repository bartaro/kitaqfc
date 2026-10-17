# FCHUA 0.1.0 Verification

## GUI Publication: 2026-10-18

The public package now includes the plita-based GUI source, the five required
plita foundation crates, Roboto/Roboto Mono fonts, and a Windows x64 GUI ZIP.
The GUI is optional. Default CLI and headless Python dependency trees contain
no plita, wgpu, winit or rfd. All 57 original locked registry package versions
and checksums were retained when adding the GUI dependency graph.

- The GUI-only Windows release compiled successfully with the static C runtime.
- With `gui,python`, 19 shared-core/UI tests and 5 CLI integration tests passed.
  Coverage includes button hover/release, menu hit blocking, undo, scrollbars,
  palette clipping, portable font lookup, project IO and transactional editing.
- DrawList software snapshots were generated at 900x600, 1024x600, 1360x820 and
  1920x1080, with menu-open variants. The 1360x820 and 900x600 menu images were
  visually inspected. These are CPU renders, not native window captures.
- The GUI-enabled Python extension loaded in CPython. Controller edits and
  project save/load passed; ButtonInteraction hover/press/release passed.
  The headless CLI strictly validated the Python-saved project with no warnings.
- All 3 existing installed-headless-wheel Python tests passed.
- All 4 distribution tests passed, checking nine manuals and their links,
  curated crate boundaries, dependency notices, fonts, ZIP integrity and hashes.
- GUI dependency notices cover 144 Windows normal/build registry packages.
  Missing crate-archive license texts were obtained from the exact upstream
  commits; URLs and hashes are recorded in license-overrides/SOURCES.json.
- The CLI was rebuilt without GUI/Python features; its help uses `plita`.
  PE import inspection confirms both release executables import no Python DLL
  or dynamic VC/UCRT runtime DLL. GUI subsystem=2; CLI subsystem=3.

The ZIP contains the GUI-only binary, not the `gui,python` test-build executable.
GUI executable SHA256:
`19d56467c11e905d9da02ed26a377eec12411bc1de90fddb9ec2b3f8d2600862`.
The authoritative per-file digest is in the ZIP's SHA256SUMS.
The source package's SHA256SUMS covers the complete ZIP,
the standalone CLI and the unchanged headless Python wheel.

Native-window clicking, file dialogs and a native GPU-present smoke test remain
unverified. The desktop tool returned `Computer Use app approval timed out`
when asked to launch the unpacked FCHUA.exe; a subsequent window inventory
contained no FCHUA window. Automated input-state tests and software snapshots
are not a substitute for that interactive check. All nine manuals disclose it.

Reproduction from tools/fchua:

```powershell
$env:RUSTFLAGS = '-C target-feature=+crt-static'
cargo build --release --locked --features gui --bin fchua_gui --target x86_64-pc-windows-msvc
python scripts/collect_notices.py --gui
python scripts/package_gui.py target/x86_64-pc-windows-msvc/release/fchua_gui.exe
$env:PLITA_UI_FONT = (Resolve-Path -LiteralPath 'assets/fonts/Roboto[wdth,wght].ttf').Path
$env:FCHUA_UI_SNAPSHOTS = Join-Path $env:TEMP 'fchua-snapshots'
cargo test --release --locked --features gui,python --target x86_64-pc-windows-msvc
python tests/distribution.py
```

Package the GUI-only executable before building/testing `gui,python`: Cargo may
replace the binary in the target directory when the feature set changes.

## Previous CLI/Python Baseline: 2026-10-11

The following checks apply to the earlier Windows x64 CLI/Python release.
Python support is an optional Cargo feature; the CLI executable does not
require Python.

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

The previous local plita desktop build passed 18 core/UI tests and 5 CLI tests.
Draw-list snapshots were inspected at 900x600, 1024x600, 1360x820 and 1920x1080.
Tests cover shared button release behavior, menu hit blocking, scrolling,
selection without dirtying the document, and one undo step per drawing stroke.

Native-window click/dialog automation was unavailable in that baseline. Software rendering and
event tests are not a substitute for an end-to-end native-window interaction
test. Hardware/console and full-game emulator validation are outside these checks.
