# KITAQGB → KITAQFC library inventory (2026-11-02)

The inventory compares **both** `kitaqgb_rust/lib` and the workspace's newer
shared `lib`. The target is the Rust compiler's `kitaqfc_rust/lib`, not the
historical C# library. [library_inventory.json](library_inventory.json) lists
every source/header, its SHA-256 and its FC destination/status.

Pinball computation is **deferred at the user's request**. GB-specific hardware
features, including GB communication, are **excluded at the user's request**.
No pinball or GB communication source/header is supplied in this FC update.

## Software libraries

| GB module/family | FC implementation | Scope |
|---|---|---|
| `rng`, `flags`, `rle` | Same names, plus individual headers | Random state, bounded flags/quests and RLE |
| `text`, `menu`, `script`, `rpg` | Same names + `rpg_runtime.c`; `adv.h` convenience include | Windows, choices, menus, waits and cooperative script execution |
| `map` | `map.c`/`map.h` | 32×30 tile view, 16×15 metatile view, collision/events and NES attribute packing |
| `save` | `save.c`/`save.h` | Four 512-byte PRG-RAM slots, 506-byte payload, checksum |
| `slg_board`, `slg_path`, `slg_unit` | Same names | Board, BFS paths, movement ranges, units |
| `camera` | Same names | Q8.8 camera, bounds, targets and NES scroll shadow |
| `sprite_order` | Same names | Four priorities, rotation and 64-entry NES OAM output |
| `math` | `math.c`/`math.h` | 256-entry sine table |
| `physics2d`, `physics2d_circle`, `physics3d` | Same names | Q8 scaling, surfaces, contacts, circles, AABBs and spheres; existing FC motion-channel API retained |
| `wire3d`, `wire3d_dmg*`, `wire3d_cgb*`, `dmg3d` | `wire3d.c` + `wire3d_model.c` | NES CHR-RAM renderer, camera, model rotation/scaling, indexed edges, faces, edge masks, scene occlusion and erasing |
| `wire3d_rig*` | `wire3d_rig.c` + `wire3d_rig_view.c` | Baked articulation, detach/dock, root rotation and scene drawing |
| `raster` | `raster.c` | MMC3 IRQ bands, HUD/parallax and quantized line-X effects |
| `zx0` | Existing `zx0.c`/`zx0.h` and `kitaqfc-zx0` host tool | Forward v2 ZX0, bounded decoding, KQA1 assets and rendering-off VRAM upload |

Existing FC equivalents also cover `asset`, `audio`, `audio_vblank`, `bank`,
`chain`, `danmaku`, `debug`, `entity`, `fixed`, `input`, `scene`, `scroll`,
`sprite`, `system` and `vram`. Scene/entity callbacks execute through the
compiler's indirect-call support; the older README restriction was stale.

## Hardware boundaries

These are native NES interfaces, not binary/GB-register compatibility. CGB tile
attributes and RGB15 palettes map to `tilemap`/`attribute`/`palette`; the NES has
different palette grouping, sprite ordering and pattern memory. GB serial
registers, link protocols/drivers and the DMG-07 adapter are excluded from this
port. Existing NES palette/attribute/audio interfaces remain the native APIs.

The GB renderer profiles (88/96/120-line layouts, CGB VRAM banks, auxiliary tile
transfer, per-edge color and their SM83 assembly) are replaced by the NES
renderer, not copied as incompatible register writes. Wire3D owns both CHR
tables, its nametable, PPU writes and ZP `$00..$0A`; it disables NMI and polls
VBlank for transfers. GB renderer-specific NMI audio glue is outside this port's
scope. Use the existing FC audio APIs with an application-owned
frame/audio schedule; mixing a second PPU/NMI owner requires integration.

## Building and memory ownership

Include only needed headers and link the corresponding `.c` files. `fc.h`
collects declarations; it does not link implementations. For RPG text/map calls,
enable NMI (`__ppu_ctrl_set(128)`) before the first call: the adapter submits a
bounded VRAM queue and waits for consumption. Load the desired font/CHR and
palettes separately. ASCII bytes are tile IDs, not a bundled Japanese font.

For example, build an ADV program with:

```powershell
.\kitaqfc.exe game.c lib/rpg_runtime.c lib/input.c lib/flags.c lib/text.c lib/menu.c lib/script.c -Ilib --mapper=mmc3 -o game.nes
```

SLG defaults to `RPG_PATH_CELLS=64`, replacing the GB's 1024-cell workspace.
Larger maps are rejected by path/range routines unless that limit is increased
consistently in every translation unit and sufficient RAM is reserved.

Save requires an enabled, writable battery-backed PRG-RAM mapping at
`$6000..$67FF`. The application/cartridge profile enables and persists it. This
API does not write FDS media or configure arbitrary mappers automatically.

Wire3D requires `--nes-chr-ram` and PRG RAM. Its base staging/dirty state occupies
`$6800..$70BF`; model/occlusion state extends this reservation through `$77FF`.
Large rig states should use separate cartridge RAM (the verification allocates
them at `$7900` and `$7B00`). Supported views are 64×48, 96×64 and 128×96; use
the same defines everywhere. Models and arrays must remain readable in the
current ROM mapping. Generic rotations have 16 steps; the small FC backend API
retains 32 steps. Near-plane crossing segments are rejected. Fixed ROM holds the
tables read directly by assembly, preventing automatic-bank-packing corruption.

Raster owns `__nes_irq`, uses MMC3 and requires BG table 0/sprite table 1 with
rendering/NMI enabled. It disables APU frame IRQ; custom IRQ/DMC users must merge
their interrupt handling. Its eight-band line-X effect is not the GB's 144-line
LY loop. IRQ latency and vertical splits still require target-console tuning.

Physics uses the portable C path and NES scale routine. GB-specific cached
fixed-bank/register-math configurations are outside this port; leave
`KQ2D_CACHE_SURFACE` and `KQ2D_REGISTER_MATH` undefined. Q8.8 camera positions
retain their signed 16-bit range; choose units/bounds accordingly.

## Verification

`verification/fds_library_20261101/compile_libraries.py` compiles every function
in independent module groups. `runtime_libraries.py` compiles original test ROMs
with the current native compiler, executes them in KUROSAKI and checks RAM,
nametable attributes, decoded CHR bytes and geometry CHR transfers. This is
emulator evidence, not physical-console, cable or performance certification.
