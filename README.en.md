# KITAQFC

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

<!-- manual-language-links:start -->
| Language / 言語 | HTML |
| --- | --- |
| English | [kitaqfc](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) |
| 日本語 | [kitaqfc](https://bartaro.github.io/kitaq-docs/kitaqfc.html) |
| 한국어 | [kitaqfc](https://bartaro.github.io/kitaq-docs/ko/kitaqfc.html) |
| 简体中文 | [kitaqfc](https://bartaro.github.io/kitaq-docs/zh-CN/kitaqfc.html) |
| 繁體中文 | [kitaqfc](https://bartaro.github.io/kitaq-docs/zh-TW/kitaqfc.html) |
| Français | [kitaqfc](https://bartaro.github.io/kitaq-docs/fr/kitaqfc.html) |
| Español | [kitaqfc](https://bartaro.github.io/kitaq-docs/es/kitaqfc.html) |
| Deutsch | [kitaqfc](https://bartaro.github.io/kitaq-docs/de/kitaqfc.html) |
<!-- manual-language-links:end -->

<!-- rust-native-20261004:start -->
## Native Rust compiler and helper tools

Build the compiler and all helper tools on Windows, Linux, macOS ARM or macOS Intel with Rust 1.85 or later. The native executables run without .NET; production asset tools also run without Python or Pillow.

```sh
cargo test --locked --tests
cargo build --locked --release
```

Windows executables are at the repository root. Linux and macOS executables are under bin/ in the platform folders listed below. Keep lib/ and license notices with the tools. On Linux/macOS, run chmod +x on the downloaded executables and add their folder to PATH, or invoke them by their full path.

| OS | kitaqfc |
| --- | --- |
| Windows x64 | `./kitaqfc.exe` |
| Linux x64 | `bin/Linux-X64/kitaqfc` |
| macOS ARM64 | `bin/macOS-ARM64/kitaqfc` |
| macOS Intel | `bin/macOS-X64/kitaqfc` |

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
```

```sh
sh scripts/build.sh
```


Native builds and executable checks passed on Windows, Linux, macOS ARM and macOS Intel. KITAQGB passed 48 tests and 395 helper checks per platform; KITAQFC passed 55 tests and 401 helper checks. Rust 1.85 was also tested. PUBLIC_DISTRIBUTION.json records the installed binary hashes and validation provenance. The public GitHub Actions workflows rebuild and test this source independently.

Frozen reference outputs test ROM bytes, diagnostics and helper formats. Earlier C# emulator evidence remains historical evidence with its original source fingerprints. It does not automatically prove every Rust API, real hardware or complete FDS BIOS/game startup. The unavailable original PNG conversion script was reconstructed from its specification; byte parity with that missing script cannot be claimed.

[Native Rust compiler and helper tools](tools/README.en.md)

<!-- rust-native-20261004:end -->






<a name="english"></a>

## English

C compiler and support libraries for original NES/Famicom/FDS homebrew software, derived from KITAQGB and NORCAL.

Public preview: APIs and behavior may change.

[Bullet-pool example and API reference](https://bartaro.github.io/kitaq-docs/en/fc-library.html#module-danmaku) · [Wireframe drawing and projection](https://bartaro.github.io/kitaq-docs/en/fc-library.html#module-wire3d)

[ZX0 API and visual example](https://bartaro.github.io/kitaq-docs/en/fc-library.html#module-zx0) · [Host compression tool](tools/zx0/README.md#english)

`audio_vblank` plays four-channel music from a RAM queue during NMI. Link `lib/audio_vblank.c`, initialize it, start a song and refill from the foreground. Records use `delay, CH1, CH2, CH3, CH4`; the compiler connects the music tick to NMI. Pause/resume preserves both music and effect timelines. A bounded SFX stream can temporarily own selected channels, and noise supports a one-shot decay envelope. [API, complete example and captured audio](https://bartaro.github.io/kitaq-docs/en/fc-library.html#module-audio_vblank)

### Development philosophy

KITAQFC supports development that respects the NES/Famicom hardware and can be checked in small, reproducible steps. Build the ROM, exercise it in KUROSAKI, inspect the evidence with SARAKURA, and repeat after each correction. Keep measured results separate from behavior that has not yet been tested.



<!-- fc-current-20261004:en:start -->
### Current compiler and library distribution — 4 October 2026

Use kitaqfc.exe and lib from the same checkout. From the parent directory, run the Release build script below; the project copies the executable to the repository root. BINARY_BUILD.json records the compiler source hashes, executable hash and checks for that build. Long diagnostic lines are limited to 200 characters followed by an ellipsis.

The current C# build was compared with the previous distribution using 29 inputs. Fifteen accepted inputs produced identical ROM bytes; the other fourteen inputs produced matching rejection diagnostics. Two additional ROMs ran for four frames in KUROSAKI: one checked entity allocation, callback dispatch and slot reuse; the other checked signed arithmetic, the KQBody3D layout and audio queue constants. The report records 18 expected RAM bytes and the actual values. These cases do not test audio playback or every library API; prior regression results are identified separately in BINARY_BUILD.json.

```powershell
.\kitaqfc\scripts\build.ps1
.\kitaqfc\kitaqfc.exe --help
```

[Source fixtures and execution evidence](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html#fc-current-20261004-heading)
<!-- fc-current-20261004:en:end -->
<!-- development-prompt:en:start -->
### Game development prompt

Fill in the requirements, then give the complete prompt to your AI assistant. It covers implementation, emulator testing, SARAKURA analysis and retesting.

[Read the reference example in the HTML manual](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html#loop-prompts)

<details>
<summary>Show the complete prompt</summary>

#### Game development with KITAQFC, KUROSAKI and SARAKURA

Fill in the requirements and give this entire document to the AI assistant. Commands assume sibling repositories named `kitaqgb`, `kitaqfc`, `kokura`, `kurosaki`, `sarakura` and `kitaq-docs`, with your `game-gb` or `game-fc` project beside them. Run commands from their parent directory; adapt paths to the actual environment.

##### Requirements

- Game title: &lt;fill in&gt;
- Genre and core gameplay: &lt;fill in&gt;
- Controls, success and failure conditions: &lt;fill in&gt;
- Required screens, stages, enemies and items: &lt;fill in&gt;
- Visual style, music and sound effects: &lt;fill in; identify any supplied assets&gt;
- Saving, communication, peripherals and other requirements: &lt;fill in, or none&gt;
- Project directory: &lt;fill in&gt;
- Redistribution requirements: &lt;for example, original code and assets suitable for MIT publication&gt;

- Target: &lt;NES/Famicom cartridge or FDS&gt;
- Mapper, ROM size and mirroring: &lt;specify, or select from the requirements&gt;
- Video standard and performance: &lt;for example, NTSC and 60 gameplay updates per second&gt;

##### Task

Implement the game with KITAQFC and its libraries. Use KUROSAKI for execution and debugging, and SARAKURA to organize diagnostics and compare results before and after a fix.

Repeat this cycle until the acceptance criteria are met: make the specification concrete → implement a small change → build → apply inputs and observe → investigate the cause → fix → retest under the same conditions. A plan, a code listing or a successful compilation is not completion.

###### Establish the environment and acceptance criteria

1. Read workspace instructions, tool READMEs, HTML manuals, and the headers and implementations of the libraries you will use. Record executable paths and versions or SHA-256 hashes. Verify commands against actual `--help` output and APIs against source.
2. Define measurable acceptance criteria for inputs, images, audio, progression and update frequency. Examples: pressing and releasing START begins the game; a collision removes one life; pausing silences the intended audio and resuming restores playback.
3. Ask only about material ambiguities. Make ordinary reversible implementation decisions autonomously. Do not weaken requirements or acceptance criteria.
4. First run a small supplied sample through the compiler, emulator and SARAKURA. This checks the tool connection, not completion of the requested game.

###### Implement a small playable slice

- Start with NROM for a small game; choose MMC3 or another mapper when banking or size requires it. Verify required board features using `inspect-rom`, `mapper-info`, `audit-board` and the implementation, rather than relying on the mapper name alone.
- Plan PRG/CHR size, CHR-ROM or CHR-RAM, mirroring, fixed banks, interrupt vectors and save RAM. Check headers against actual placement after optimizations or banking changes. `--nes-local-ram` uses internal CPU RAM in `$0000–$07FF`; avoid overlaps with zero page, stack, OAM buffers and runtime/library storage.
- Use the KITAQFC C dialect, FC libraries and `void main(void)`. Do not assume GB APIs are compatible. Check implementation providers and include the required `.c` files; some header entries are declarations only.
- Account for PPU registers, NMI, OAM DMA, per-scanline sprite limits, scrolling, mirroring, attribute tables and APU/DMC behavior. Transfer-queue free space is not PPU VRAM capacity; budget the work done per NMI.
- Convert the supplied original `ascii.c` font to FC CHR for letters, digits and symbols. Verify CHR, palettes, nametables and attributes. For FDS, separately verify disk access, saving and BIOS requirements; do not assume cartridge boot conditions.

- First connect boot, title, a controllable player, success or failure, and restart. Then expand the game.
- Keep editable graphics, music and sound-effect sources and their generation steps. Verify that the build actually consumes their exports.
- Write source comments in English and progress reports in English. Keep SARAKURA’s standard reports in English.

###### Connect each build to its execution

Use a separate output directory for each iteration, such as `out/iter-001`. Record commands, exit codes, and hashes of source, assets, tools, ROM and metadata. Never run an older ROM after a failed build. Maps, source maps and debug information must come from the same build as the ROM.

The following is an NROM check with no input. Supply `main.c`, required library implementation units and the CHR file; select the appropriate mapper. Do not pass compiler build metadata to KUROSAKI’s `--kitaqfc-debug` without first checking its required format.

```powershell
$iteration = '.\game-fc\out\iter-001'
New-Item -ItemType Directory -Force $iteration | Out-Null

# Include all additional implementation units required by the game.
& '.\kitaqfc\kitaqfc.exe' '.\game-fc\src\main.c' `
  -I '.\kitaqfc\lib' -o "$iteration\game.nes" `
  --mapper=nrom '--nes-chr=.\game-fc\assets\game.chr' --no-disasm `
  "--kurosaki-metadata=$iteration\build.json"
if ($LASTEXITCODE -ne 0) { throw 'Build failed; inspect the build log.' }

& '.\kurosaki\kurosaki.exe' run "$iteration\game.nes" `
  --frames 300 --pad1 0 --png "$iteration\frame.png" `
  --json "$iteration\run.json" --emit-diagnostics "$iteration\events.jsonl"
if ($LASTEXITCODE -ne 0) { throw 'Emulator run failed; inspect the run log.' }

& '.\sarakura\sarakura.exe' fc analyze `
  --metadata "$iteration\build.json" --events "$iteration\events.jsonl" `
  --frames 300 --out "$iteration\analysis" --fail-on error
if ($LASTEXITCODE -ne 0) { throw 'Inspect the analysis report and fix the cause.' }
```


A 300-frame run with no input is only an initial check. Add ordered gameplay scenarios before claiming that the game works.

###### Reproduce ordered input

- `--pad1` and `--pad2` use raw NES bitmasks: A=1, B=2, SELECT=4, START=8, UP=16, DOWN=32, LEFT=64, RIGHT=128. Do not confuse these with library `BTN_*` values.
- `run --pad1` applies fixed input. For sequences of actions, create a replay that distinguishes presses, holds and releases. Inspect the `Replay` and `ReplayFrame` definitions. The public CLI’s `replay-record` records neutral input, not a person’s interactive play.
- Check ROM SHA-256, replay identity and frame range yourself. `replay-run --verify` compares an expected final hash only when one is provided; it does not comprehensively validate gameplay or ROM identity. Never overwrite expected results with observed results merely to make a test pass.

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


Prepare the replay for the ROM being tested. The public CLI’s `replay-run` has no `--emit-diagnostics` option. Do not invent that option or pass CPU trace data as diagnostic events. To analyze ordered-input scenarios with SARAKURA, create a project-local test harness using public `kurosaki-core` APIs: `RunOptions.replay_frames` and `diagnostic_events_from_trace_and_report`. Run the same ROM, replay and frame conditions; generate diagnostic JSONL from that execution’s trace and diagnostic report. Check trace configuration and retained ranges, and compare the harness’s images and observations with CLI replay results. Do not present no-input diagnostics as evidence for a gameplay scenario. If the required environment is unavailable, report this verification as incomplete.

###### Check images, audio, state and performance

- Save input scenarios with distinct presses, holds and releases. Exercise every specified path: boot, start, movement, actions, collisions, scrolling, stage changes, game over, restart, pause and saving or communication where applicable.
- Preserve PNGs at relevant frames, input data, execution reports, diagnostic JSONL, WAVs and any necessary state or memory observations. Check the reached frame count and stop reason. Actually open the images; one screenshot cannot establish motion or input response. Compare counters, positions and state changes with expected values. Check screen edges, tile/attribute boundaries and crowded sprite scenes.
- Check music, effects, simultaneous playback, dropouts, pause and resume. A generated WAV alone does not establish correct sound. If listening is unavailable, distinguish the waveform/numerical checks performed from unverified audible qualities.
- Measure heavy scenes, target CPU/update workload and transfers; on FC, include NMI work. Host emulator throughput is not game update frequency or proof of hardware speed. Continuing with `--allow-unimplemented`, where available, does not demonstrate support for the missing feature.

###### Analyze, repair and retest

- Feed SARAKURA the build metadata for the tested ROM and diagnostic JSONL from the tested execution. A CPU trace or ordinary run report is not a substitute. `--frames` specifies analysis conditions; SARAKURA does not execute the ROM or automatically edit the source.
- Read `report.html`, `ai_diagnostics.json`, `repair_prompt.md` and `retest_plan.json`. Compare diagnoses with reproduction steps, images, audio and source. Distinguish inferred source locations or causes from verified facts, and normal waiting loops from hangs. Assess warnings individually and record unsupported events or analysis limits. Do not hide warnings with filters or shorten tests to obtain a passing result.
- Reduce failures to minimal reproductions, fix their causes and rebuild. If the compiler or emulator is responsible, isolate its defect from game code and add regression verification for the tool fix.
- Retest with matching input, random seed, hardware/video mode, mapper, observed frames and diagnostic settings. Use new metadata for each new ROM; do not blindly reuse save states after code or RAM layout changes.

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


Use diagnostic differences alongside gameplay, graphics and audio acceptance checks. If the same failure repeats, revisit the evidence and hypothesis instead of continuing arbitrary changes.

###### Completion and deliverables

Rerun all required scenarios against the final ROM built from the delivered source and settings. Invincibility, automatic test input or another mapper alone does not verify normal play in the final build. Provide a requirement-to-test table, reasons for remaining warnings and explicit unverified or unsupported items. State “not tested on physical hardware” when applicable.

Deliver source, tool/library identities, editable assets, reproducible build and test scripts, the ROM, final verification evidence, and a README covering setup, controls and known limits. Include replay data and a test harness where needed. Publish or send files externally only within explicitly authorized scope. Delete unnecessary intermediate builds and temporary traces after verification, preserving source, assets, final deliverables and needed regression evidence.

If environment or permission constraints prevent a required check, report the exact reproduction steps and required action. Do not mark the work complete.

</details>
<!-- development-prompt:en:end -->

### Repository layout

```text
Cargo.toml / Cargo.lock
src/                     # Rust compiler and native helper sources
kitaqfc.exe             # Windows x64 compiler
kitaqfc-*.exe           # Windows native helper tools
bin/                     # Linux and macOS executables
lib/                     # C libraries for console ROMs
tests/                   # Frozen reference fixtures and Rust tests
scripts/build.ps1
scripts/build.sh
```

Build the compiler and all helper tools on Windows, Linux, macOS ARM or macOS Intel with Rust 1.85 or later. The native executables run without .NET; production asset tools also run without Python or Pillow.

```sh
cargo test --locked --tests
cargo build --locked --release
```

Windows executables are at the repository root. Linux and macOS executables are under bin/ in the platform folders listed below. Keep lib/ and license notices with the tools. On Linux/macOS, run chmod +x on the downloaded executables and add their folder to PATH, or invoke them by their full path.

| OS | kitaqfc |
| --- | --- |
| Windows x64 | `./kitaqfc.exe` |
| Linux x64 | `bin/Linux-X64/kitaqfc` |
| macOS ARM64 | `bin/macOS-ARM64/kitaqfc` |
| macOS Intel | `bin/macOS-X64/kitaqfc` |

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
```

```sh
sh scripts/build.sh
```

### Build and first use

Build the compiler and all helper tools on Windows, Linux, macOS ARM or macOS Intel with Rust 1.85 or later. The native executables run without .NET; production asset tools also run without Python or Pillow.

```sh
cargo test --locked --tests
cargo build --locked --release
```

Windows executables are at the repository root. Linux and macOS executables are under bin/ in the platform folders listed below. Keep lib/ and license notices with the tools. On Linux/macOS, run chmod +x on the downloaded executables and add their folder to PATH, or invoke them by their full path.

| OS | kitaqfc |
| --- | --- |
| Windows x64 | `./kitaqfc.exe` |
| Linux x64 | `bin/Linux-X64/kitaqfc` |
| macOS ARM64 | `bin/macOS-ARM64/kitaqfc` |
| macOS Intel | `bin/macOS-X64/kitaqfc` |

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
```

```sh
sh scripts/build.sh
```

### Manuals and licenses

- [Japanese HTML manuals](https://bartaro.github.io/kitaq-docs/kitaqfc.html) / [English manuals](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)
- [Offline manual source](https://github.com/bartaro/kitaq-docs)
- [License](LICENSE) / [Japanese reference translation](LICENSE.ja)

The project license does not replace third-party font, dependency, logo or trademark terms. Preserve the accompanying notices when redistributing.

---

