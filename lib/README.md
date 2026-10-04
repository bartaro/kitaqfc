# FC/NES standard library layout

[简体中文](README.zh-CN.md) · [简体中文 HTML](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html)





<!-- fc-current-20261004:en:start -->
### Current compiler and library distribution — 4 October 2026

fc.h includes the physics3d and audio_vblank declarations. Link the required implementation files separately. entity_update_all and entity_draw_all invoke the callback for active slots in ascending ID order; use a callback in common bank 0 or keep its PRG bank mapped throughout dispatch. Include zx0.h explicitly when using ZX0. Preserve the four-channel record order delay, CH1, CH2, CH3, CH4 for audio_vblank.

The current C# build was compared with the previous distribution using 29 inputs. Fifteen accepted inputs produced identical ROM bytes; the other fourteen inputs produced matching rejection diagnostics. Two additional ROMs ran for four frames in KUROSAKI: one checked entity allocation, callback dispatch and slot reuse; the other checked signed arithmetic, the KQBody3D layout and audio queue constants. The report records 18 expected RAM bytes and the actual values. These cases do not test audio playback or every library API; prior regression results are identified separately in BINARY_BUILD.json.

[Source fixtures and execution evidence](https://bartaro.github.io/kitaq-docs/en/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:en:end -->



Public library names intentionally follow the KITAQGB style: short, functional names
without a `kitaqfc_` prefix.

## Core

- `fc.h` - aggregate include
- `core.h` - basic types
- `intrinsics.h` - compiler-recognized intrinsic entry points
- `runtime.h` / `runtime.c` - NMI, OAM shadow, VRAM queue runtime support
- `system.h` / `system.c` - KITAQGB-style frame/wait wrappers
- `debug.h` / `debug.c` - small in-RAM trace/assert log

## Graphics

- `vram.h` / `vram.c` - KITAQGB-style queued VRAM helpers
- `sprite.h` / `sprite.c` - KITAQGB-style sprite allocation/metasprite helpers
- `ppu.h` / `ppu.c`
- `ppu_direct.h`
- `vram_queue.h`
- `palette.h` / `palette.c`
- `scroll.h` / `scroll.c`
- `tilemap.h` / `tilemap.c`
- `nametable_asset.h` / `nametable_asset.c`
- `attribute.h` / `attribute.c`
- `metasprite.h` / `metasprite.c`
- `oam.h`
- `oam_fair.h` / `oam_fair_impl.h` - priority-preserving 64-candidate OAM rotation; see [oam_fair.md](oam_fair.md)

## Game framework

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## Audio and devices

- `audio.h` / `audio.c`
- `audio_vblank.h` / `audio_vblank.c` — NMI music with a seven-record BGM queue, pause/resume, a separate seven-record SFX buffer and one-shot noise envelopes. [API and recorded examples](https://bartaro.github.io/kitaq-docs/en/fc-library.html#module-audio_vblank)
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c` - KITAQGB button-state compatibility wrapper
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## Mapper/FDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## Math

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c` - Box integration, gravity, AABB contacts, surface response and optional Q5.3 data types. Compile with `fixed.c`.
- `physics3d.h` / `physics3d.c` - Nonrotating 3D boxes, mass-weighted bounce and impact values. Compile with `fixed.c` and `physics2d.c`.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

Scene transitions, updates and drawing call their registered handlers synchronously. See the individual API entries for callback order and reentrancy constraints.

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

<!-- manual-language-links:start -->
| Language / 言語 | HTML |
| --- | --- |
| English | [fc-library](https://bartaro.github.io/kitaq-docs/en/fc-library.html) |
| 日本語 | [fc-library](https://bartaro.github.io/kitaq-docs/fc-library.html) |
| 한국어 | [fc-library](https://bartaro.github.io/kitaq-docs/ko/fc-library.html) |
| 简体中文 | [fc-library](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html) |
| 繁體中文 | [fc-library](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html) |
| Français | [fc-library](https://bartaro.github.io/kitaq-docs/fr/fc-library.html) |
| Español | [fc-library](https://bartaro.github.io/kitaq-docs/es/fc-library.html) |
| Deutsch | [fc-library](https://bartaro.github.io/kitaq-docs/de/fc-library.html) |
<!-- manual-language-links:end -->

<!-- rust-native-20261004:start -->
## Library use with the native compiler

lib/ contains C headers and sources compiled into the console ROM. The desktop compiler and helper tools are Rust; game sources, target libraries and their APIs remain C. SOURCE_MANIFEST.json and LIBRARY_MERGE.json record the selected library sources where provided.

Native builds and executable checks passed on Windows, Linux, macOS ARM and macOS Intel. KITAQGB passed 48 tests and 395 helper checks per platform; KITAQFC passed 55 tests and 401 helper checks. Rust 1.85 was also tested. PUBLIC_DISTRIBUTION.json records the installed binary hashes and validation provenance. The public GitHub Actions workflows rebuild and test this source independently.

Frozen reference outputs test ROM bytes, diagnostics and helper formats. Earlier C# emulator evidence remains historical evidence with its original source fingerprints. It does not automatically prove every Rust API, real hardware or complete FDS BIOS/game startup. The unavailable original PNG conversion script was reconstructed from its specification; byte parity with that missing script cannot be claimed.

[Native Rust compiler and helper tools](../tools/README.en.md)

<!-- rust-native-20261004:end -->
