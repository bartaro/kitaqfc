# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA edits NES tiles, nametables, palettes and 8x8 metasprite frames for KITAQFC. The standalone CLI shares the desktop editor's core and does not require a GUI, GPU, PLITA installation or Python.

## Build and Run

Rust 1.92 or newer is required to build. On Windows use target/release/fchua.exe; on Linux and macOS use ./target/release/fchua. Run --help for syntax. Commands below use fchua as the executable name. Create parent output directories first.

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## Workflow

Start with a blank project or import PNG artwork. Inspect the report and PNG preview before exporting. --auto-palette selects four NES background palettes by reducing conversion error; without it, the current project's palettes are used. --project base.fch supplies an existing palette and pattern-table configuration. Transparent pixels are composited over the current universal background color. Imports replace the nametable and selected background table, so sprite tiles sharing that table also change.

```sh
fchua new --out blank.fch
fchua import-image artwork.png --out scene.fch --auto-palette
fchua inspect scene.fch
fchua validate scene.fch --strict
fchua preview scene.fch --out preview.png --view map --scale 2
fchua apply scene.fch --script examples/edits.json --out edited.fch
fchua export edited.fch --out assets --prefix scene
```

## Command Reference

```text
new --out FILE.fch [--demo] [--force]
inspect FILE.fch
validate FILE.fch [--strict]
import-image IMAGE.png --out FILE.fch [--project BASE.fch] [--auto-palette] [--force]
import-raw --out FILE.fch [--project BASE.fch] [--chr FILE.chr]
           [--palette FILE.pal] [--nametable FILE.nam] [--attributes FILE.atr] [--force]
apply INPUT.fch --script EDITS.json --out OUTPUT.fch [--dry-run] [--force]
preview FILE.fch --out IMAGE.png [--view map|tiles|sprite|tile] [--scale 1..8] [--force]
export FILE.fch --out DIRECTORY [--prefix C_IDENTIFIER] [--force]
migrate OLD.kfcproject --out FILE.fch [--force]
```

## Batch Editing

A script is a JSON array. Every operation is validated before the edited copy replaces the document. A failed batch writes nothing. --dry-run validates and reports without writing a project. Unknown operations and fields are errors. Tile IDs are absolute 0..511 for tile operations, but map and sprite IDs are local 0..255 within their selected pattern table.

```json
[
  {"op":"pixel","tile":1,"x":2,"y":3,"color":1},
  {"op":"fill","tile":2,"color":0},
  {"op":"checker","tile":3,"a":1,"b":2},
  {"op":"line","tile":4,"x0":0,"y0":0,"x1":7,"y1":7,"color":3},
  {"op":"rect","tile":5,"x0":1,"y0":1,"x1":6,"y1":6,"color":2},
  {"op":"mirror","tile":1,"horizontal":true,"vertical":false},
  {"op":"rotate","tile":1},
  {"op":"copy","source":1,"target":6},
  {"op":"palette","index":1,"value":48},
  {"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":0},
  {"op":"tables","background":0,"sprites":1},
  {"op":"sprite","frame":0,"index":0,"x":-8,"y":0,"tile":1,"attr":64},
  {"op":"clone_frame","frame":0},
  {"op":"remove_sprite","frame":1,"index":0},
  {"op":"shift_map","dx":2,"dy":0},
  {"op":"dedupe"}
]
```

`tile: 0..511; x/y (pixel): 0..7; color: 0..3; palette index: 0..31; value: 0..63`

`map: 32x30; palette: 0..3; tables: 0..1; frame: 0..255; index: 0..63`

`attr: palette bits 0..1, priority 0x20, flip H 0x40, flip V 0x80; reserved: 0x1c`

`shift_map: dx=-32..32, dy=-30..30; dx % 2 == 0, dy % 2 == 0`

## Desktop and Python

The local PLITA desktop uses Roboto, grouped menus with Lucide icons, hover/press/release feedback and tooltips. File provides New, Open, Save, Save As, PNG import and exports. Ctrl+O opens, Ctrl+S saves, Ctrl+Shift+S saves as; Ctrl+Z/Y undo/redo and Ctrl+C/V copy/paste a tile. Only content changes mark the project dirty; selecting a tile or tool does not. Closing, opening another project or creating a new one asks before discarding edits.

Map/Tile/Sprite/Check switch the center view. B/E/F/L/R/I select pencil/eraser/fill/line/rectangle/picker; 1..4 select a color. One drag is one undo step; Escape cancels it. Use Fit and zoom icons, wheel scrolling, scrollbar thumbs or tracks. In narrow windows, Tiles/Palette switch the sidebar. Click a palette swatch, then a NES color below it. Color zero edits the universal background. Sprite view places the selected tile on an 8-pixel grid; Picker selects a quad, Eraser removes it. Sprite menu adds/clones frames and flips or removes the selected quad. Check reports per-frame scanline warnings, not combined-scene or emulator validation.

The standalone package includes optional Python bindings (Python 3.10+), with no GUI, GPU or PLITA dependency. Install with pip from tools/fchua, or use the supplied Windows x64 wheel. Controller supports project IO, PNG conversion, transactional commands and raw/C exports. GUI_AVAILABLE is False in this build. The local PLITA build additionally provides run_editor and ButtonInteraction. Controller edits its own document, not a running GUI; exchange .fch files. Python saves replace existing files, unlike the CLI's --force guard.

```sh
# Standalone CLI package: build Python bindings from source (Rust required)
cd tools/fchua
python -m pip install .
# Windows x64, CPython 3.10+: prebuilt wheel, no Rust required
python -m pip install python-dist/fchua-0.1.0-cp310-abi3-win_amd64.whl
```

```sh
# Local PLITA workspace
cargo run -p kitaqfc_graphics_editor --bin fchua_gui
python -m pip install ./examples/kitaqfc_graphics_editor
```

```python
import fchua
editor = fchua.Controller()
editor.new_blank()
editor.apply_commands('[{"op":"fill","tile":1,"color":2},{"op":"map_rect","x":0,"y":0,"width":32,"height":30,"tile":1,"palette":0}]')
editor.save_project("scene.fch")
editor.preview_png("preview.png", "map", 2)
editor.export_raw_assets("assets", "scene")
print(fchua.GUI_AVAILABLE, editor.snapshot().unique_tiles)
```

## Hardware and Files

A .fch file is a versioned JSON project (format=fchua, version=1). It stores 8192 CHR bytes, 32 palette bytes, a 32x30 nametable, 64 attributes, both pattern-table selections and up to 256 metasprite frames. Legacy .kfcproject files can be migrated; hardware attributes take precedence over inconsistent legacy palette maps. PNG dimensions must be multiples of 8, from 8x8 to 256x240. One 16x16 block uses one four-color palette. One background table supports 256 tile IDs, with tile zero reserved as blank during image import; excessive unique tiles are rejected.

Export writes .chr (8192), .pal (32), .nam (960), .atr (64), metasprite JSON, all-frame C/H arrays, an asset-pack manifest and a .fch project. C arrays use KITAQFC's __prg_rom qualifier. Set PPU pattern-table registers to match the project; the pack does not configure game code. Raw import accepts whole CHR tiles up to 8192 bytes and exact lengths for the other three files. NES palette codes are 0..63. Metasprites have at most 64 quads per frame; coordinates are signed -128..127, but x=-1 is reserved for the 0xFF terminator. Only 8x8 sprite mode is supported. More than eight quads on one scanline produces a warning.

## Safety and Errors

Existing output needs --force. Input and output paths must differ even with --force. Project saves use a temporary file and atomic replacement. Export --out is a directory; --force permits replacement of its named asset files, not deletion of unrelated files. --prefix must be a C identifier. Successful commands print one JSON object; errors print JSON to stderr. Exit codes are 0 success, 1 warnings under validate --strict, and 2 invalid arguments, project, conversion or IO. Back up projects before replacing exports. No network or AI-generation service is called.

Original code: MIT. Dependencies keep their own licenses; see LICENSE and THIRD_PARTY_NOTICES.md.
