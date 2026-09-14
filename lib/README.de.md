# Aufbau der FC-/NES-Standardbibliothek

[English](README.md) | **Deutsch**

**[Deutsches Bibliothekshandbuch öffnen](https://bartaro.github.io/kitaq-docs/de/fc-library.html)**

Die öffentlichen Bibliotheksnamen folgen dem Stil von KITAQGB: kurze, funktionsbezogene Namen ohne Präfix `kitaqfc_`.

## Grundfunktionen

- `fc.h`: Sammelheader.
- `core.h`: grundlegende Datentypen.
- `intrinsics.h`: vom Compiler erkannte Intrinsics.
- `runtime.h` / `runtime.c`: Laufzeitunterstützung für NMI, OAM-Schattenpuffer und VRAM-Warteschlange.
- `system.h` / `system.c`: Bild- und Wartefunktionen im KITAQGB-Stil.
- `debug.h` / `debug.c`: kleines RAM-Protokoll für Ablaufmarkierungen und Assertions.

## Grafik

- `vram.h` / `vram.c`: VRAM-Warteschlangenfunktionen im KITAQGB-Stil.
- `sprite.h` / `sprite.c`: Sprite-Reservierung und Metasprite-Hilfen im KITAQGB-Stil.
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
- `oam_fair.h` / `oam_fair_impl.h`: wechselnde OAM-Reihenfolge für bis zu 64 Kandidaten unter Beibehaltung der Prioritäten; siehe [oam_fair.md](oam_fair.md).

## Spielgerüst

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## Audio und Geräte

- `audio.h` / `audio.c`
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c`: KITAQGB-kompatible Tastenstatusfunktionen.
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## Mapper und FDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## Mathematik

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c`: bytegroße Q5.3-Typen und Einstellkonstanten für ganzzahlige Pixelpositionen, Achtelpixel-Anteile, vorzeichenlose Geschwindigkeit, separate Richtung und Bremswirkung. Die Bewegungsintegration übernimmt der Spielcode; der Header enthält keine Physik-Schrittfunktion. Die getrennten Felder erlauben Berechnungen in zeitkritischen Schleifen ohne zusätzlichen ABI-Aufwand für Strukturen und Zeiger.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

KITAQGB-Kompatibilitätsfunktionen behalten ihre kurzen Namen, soweit das FC-/NES-Hardwaremodell die jeweilige Funktion zulässt. Die callbackartigen Szenen- und Objektfunktionen speichern derzeit Zustände, rufen aber keine benutzerdefinierten Funktionszeiger indirekt auf.
