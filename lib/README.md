# FC/NES standard library layout

<!-- manual-language-links:start -->
| Language / 言語 | HTML |
| --- | --- |
| English | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/en/fc-library.html) |
| 日本語 | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/fc-library.html) |
| 한국어 | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/ko/fc-library.html) |
| 简体中文 | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html) |
| 繁體中文 | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html) |
| Español | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/es/fc-library.html) |
| Português (Brasil) | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/pt/fc-library.html) |
| Français | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/fr/fc-library.html) |
| Deutsch | [KITAQFC Library](https://bartaro.github.io/kitaq-docs/de/fc-library.html) |
<!-- manual-language-links:end -->

[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md)


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
- `physics2d.h` / `physics2d.c` - byte-sized Q5.3 types and tuning constants:
  whole-pixel position, 1/8-pixel fraction, unsigned speed, explicit direction
  and drag settings. Game code performs integration; the header supplies no
  physics-step routine. Keeping fields separate lets hot loops compute
  movement without aggregate/pointer ABI traffic.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

KITAQGB compatibility helpers intentionally keep short functional names where
the FC/NES hardware model can support them. Callback-style scene/entity helpers
currently keep state but do not indirect-call user function pointers.
