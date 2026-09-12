# FC/NES standard library layout

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
- `physics2d.h` / `physics2d.c` - byte-safe Q5.3 motion channels for smooth
  2D games: whole-pixel position, 1/8-pixel fraction, unsigned speed,
  explicit direction, and drag tuning constants. The header-driven surface
  lets hot loops expand integration locally without aggregate/pointer ABI
  traffic.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

KITAQGB compatibility helpers intentionally keep short functional names where
the FC/NES hardware model can support them. Callback-style scene/entity helpers
currently keep state but do not indirect-call user function pointers.
