# Native Hilfsprogramme

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md)

Alle Hilfsprogramme sind in Rust implementiert und benötigen zur Ausführung weder .NET noch Python/Pillow. Bauen Sie alle Programme mit `cargo build --locked --release`. Unter Windows erhalten die Befehle die Endung `.exe`.

```text
kitaqfc-zx0 input.bin output.zx0
kitaqfc-zx0 output.zx0 restored.bin --decompress
kitaqfc-zx0 input.bin asset.h --header=level_data
kitaqfc-zx0 input.bin output.kqa --format=auto
kitaqfc-asset-pack assets/example_manifest.json
kitaqfc-png-index-build assets/example_png_manifest.json
kitaqfc-asset-pipeline assets/example_pipeline_manifest.json
kitaqfc-rights-name-guard --root . --denylist deny_terms.local.txt
```

ZX0 akzeptiert 1 bis 65535 Bytes und bewahrt die Ausgabe des C#-Encoders. Unterstützt werden `raw`, Anzahl/Wert-`rle` und der automatische neun Byte lange `KQA1`-Container. Bei Gleichstand gilt raw, RLE, ZX0. `--decompress` dekodiert einen vorwärts gelesenen ZX0-v2-Strom mit Ausgabelimit. Rückwärtsströme und v1 sind nicht unterstützt. Das Format stammt von Einar Saukas; diese KITAQ-Implementierung steht unter MIT.

Vorhandene JSON-Manifeste erzeugen 8 KiB CHR, C-Arrays/Header und JSON-Berichte. Metasprites unterstützen data, frames oder JSON-Dateien. Indizierte PNGs erzeugen CHR, Nametable, Attribute, Palette und Metasprite-JSON. Hintergründe: 256×240; Tiles: 8×8; gleiche Palettengruppen pro Tile und 16×16-Hintergrundquadrant. Indizes 0–15, Bittiefen 1/2/4/8, Adam7. Das fehlende PNG-Skript wurde anhand der lokalen Spezifikation und Beispielmanifeste rekonstruiert; ein Bytevergleich mit dem nicht verfügbaren Original ist unmöglich. RGB wird auf eine definierte ungefähre NES-Palette abgebildet. Die Pipeline führt beide Schritte intern aus. Die Namensprüfung liest eine externe Liste, meldet nur Datei/Zeile und keine geschützten Begriffe; generierte Ordner sind ausgeschlossen.
