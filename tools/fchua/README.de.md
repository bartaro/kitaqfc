# FCHUA

[English](README.en.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português](README.pt.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

FCHUA bearbeitet NES-Tiles, Nametables, Paletten und 8x8-Metasprites für KITAQFC. Die eigenständige CLI verwendet denselben Kern wie die Oberfläche und benötigt weder GUI noch GPU, PLITA-Installation oder Python.

## Bauen und starten

Zum Bauen wird Rust ab 1.92 benötigt. Windows: target/release/fchua.exe; Linux und macOS: ./target/release/fchua. --help zeigt die Syntax. In den Beispielen heißt das Programm fchua. Übergeordnete Ausgabeordner müssen bereits existieren.

```sh
cd tools/fchua
cargo build --release --locked
cargo test --locked
./target/release/fchua --help
```

## Arbeitsablauf

Erstellen Sie ein leeres Projekt oder importieren Sie PNG. Prüfen Sie Bericht und Vorschau vor dem Export. --auto-palette wählt vier NES-Hintergrundpaletten zur Verringerung des Konvertierungsfehlers; sonst gelten die aktuellen Paletten. --project base.fch übernimmt Paletten und Mustertabellen. Transparente Pixel werden mit der aktuellen universellen Hintergrundfarbe verrechnet. Der Import ersetzt Karte und gewählte Hintergrundtabelle; Sprites mit derselben Tabelle ändern sich ebenfalls.

```sh
fchua new --out blank.fch
fchua import-image artwork.png --out scene.fch --auto-palette
fchua inspect scene.fch
fchua validate scene.fch --strict
fchua preview scene.fch --out preview.png --view map --scale 2
fchua apply scene.fch --script examples/edits.json --out edited.fch
fchua export edited.fch --out assets --prefix scene
```

## Befehlsreferenz

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

## Stapelbearbeitung

Ein Skript ist ein JSON-Array. Alle Operationen werden vor der Änderung geprüft. Bei Fehlern wird nichts geschrieben. --dry-run prüft ohne Speicherung. Unbekannte Operationen und Felder sind Fehler. Tile-Operationen verwenden absolute Nummern 0..511; Karte und Sprites verwenden 0..255 innerhalb ihrer jeweiligen Tabelle.

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

## Oberfläche und Python

Die lokale PLITA-Oberfläche nutzt Roboto, gruppierte Menüs mit Lucide-Symbolen, Hover-/Druck-/Loslasszustände und Tooltips. File bietet Neu, Öffnen, Speichern, Speichern unter, PNG-Import und Export. Ctrl+O/S/Shift+S öffnet/speichert/speichert unter; Ctrl+Z/Y macht rückgängig/wiederholt; Ctrl+C/V kopiert/fügt ein Tile ein. Reine Auswahländerungen gelten nicht als Bearbeitung. Beim Schließen, Neuanlegen oder Öffnen eines anderen Projekts wird vor dem Verwerfen von Änderungen gefragt.

Map/Tile/Sprite/Check wechseln die Hauptansicht. B/E/F/L/R/I wählen Stift/Radierer/Füllen/Linie/Rechteck/Pipette; 1..4 wählen Farben. Ein Ziehvorgang ist ein Rückgängig-Schritt, Escape bricht ihn ab. Fit, Zoom, Mausrad und Schieberegler samt Laufleisten stehen bereit. In schmalen Fenstern wechselt Tiles/Palette die Seitenleiste. Wählen Sie ein Farbfeld und danach eine NES-Farbe; Index null ändert den universellen Hintergrund. Sprite setzt Tiles auf ein 8-Pixel-Raster; Pipette wählt Teile, Radierer löscht. Das Menü ergänzt/dupliziert Frames und spiegelt/löscht das gewählte Teil. Check prüft einzelne Frames, nicht die gesamte Szene oder einen Emulator.

Das CLI-Paket enthält optionale Python-Bindings (Python 3.10+), ohne GUI, GPU oder PLITA. Bauen Sie mit pip in tools/fchua oder installieren Sie das mitgelieferte Windows-x64-Wheel. Controller unterstützt Projekte, PNG, transaktionale Befehle und RAW/C-Export; GUI_AVAILABLE=False. Der lokale PLITA-Build ergänzt run_editor und ButtonInteraction. Controller bearbeitet ein eigenes Dokument, keine laufende GUI; tauschen Sie .fch-Dateien aus. Python überschreibt vorhandene Dateien, anders als der --force-Schutz der CLI.

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

## Hardware und Dateien

.fch ist versioniertes JSON (format=fchua, version=1): 8192 CHR-Bytes, 32 Palettenbytes, Karte 32x30, 64 Attribute, zwei Tabellenauswahlen und bis zu 256 Frames. Alte .kfcproject-Dateien sind migrierbar; Hardwareattribute haben Vorrang vor widersprüchlichen Palettenkarten. PNG-Abmessungen müssen Vielfache von 8 zwischen 8x8 und 256x240 sein. Jeder 16x16-Block nutzt eine Vierfarbpalette. Eine Hintergrundtabelle hat 256 Tiles; beim Import bleibt Tile null leer reserviert. Überschreitungen werden abgelehnt.

Exportiert werden .chr (8192), .pal (32), .nam (960), .atr (64), Metasprite-JSON, C/H-Arrays aller Frames, Manifest und .fch. C verwendet __prg_rom. Die PPU-Tabellen müssen im Spiel passend eingestellt werden; dieser Code wird nicht erzeugt. Roh-CHR besteht aus ganzen 16-Byte-Tiles bis 8192 Bytes; die anderen drei Formate verlangen exakte Längen. NES-Farbcodes: 0..63. Maximal 64 Teile pro Frame, Koordinaten -128..127 außer x=-1, das für den Abschluss 0xFF reserviert ist. Nur 8x8-Sprites werden unterstützt; mehr als acht Teile pro Scanline erzeugen eine Warnung.

## Schutz und Fehler

Vorhandene Ausgaben erfordern --force; Eingabe und Ausgabe müssen trotzdem verschieden sein. Projekte werden über eine temporäre Datei atomar ersetzt. export --out bezeichnet ein Verzeichnis; --force ersetzt benannte Assets, löscht aber keine fremden Dateien. --prefix muss ein C-Bezeichner sein. Erfolg schreibt ein JSON-Objekt, Fehler schreiben JSON nach stderr. Rückgabecodes: 0 Erfolg, 1 Warnungen bei validate --strict, 2 Argument-/Projekt-/Konvertierungs-/E/A-Fehler. Vor Ersetzungen sichern. Es werden keine Netzwerk- oder KI-Generierungsdienste aufgerufen.

Originalcode: MIT. Abhängigkeiten behalten ihre Lizenzen; siehe LICENSE und THIRD_PARTY_NOTICES.md.
