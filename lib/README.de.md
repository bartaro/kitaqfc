# Aufbau der FC-/NES-Standardbibliothek

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | **Deutsch** | [Português (Brasil)](README.pt-BR.md)
<!-- readme-language-links:end -->

<!-- fc-current-20261004:de:start -->
### Aktuelle Compiler- und Bibliotheksausgabe — 4. Oktober 2026

fc.h enthält die Deklarationen von physics3d und audio_vblank. Binden Sie die benötigten Implementierungsdateien gesondert ein. entity_update_all und entity_draw_all rufen den Callback für aktive Plätze nach aufsteigender ID auf. Legen Sie den Callback in die gemeinsame Bank 0 oder halten Sie seine PRG-Bank während der Aufrufe eingeblendet. Für ZX0 muss zx0.h ausdrücklich eingebunden werden. Die Reihenfolge der audio_vblank-Datensätze lautet delay, CH1, CH2, CH3, CH4.

Der aktuelle C#-Build wurde mit der vorherigen Ausgabe anhand von 29 Eingaben verglichen. Fünfzehn akzeptierte Eingaben erzeugten identische ROM-Bytes; die übrigen vierzehn Eingaben erzeugten identische Ablehnungsdiagnosen. Zwei weitere ROMs liefen je vier Frames in KUROSAKI: Entitätsvergabe, Callbacks und Wiederverwendung von Plätzen; vorzeichenbehaftete Berechnungen, KQBody3D-Layout und Audiowarteschlangenkonstanten. Der Bericht enthält 18 erwartete und beobachtete RAM-Bytes. Audiowiedergabe und sämtliche APIs werden damit nicht geprüft; frühere Regressionen sind in BINARY_BUILD.json gesondert gekennzeichnet.

[Quellbeispiele und Ausführungsnachweise](https://bartaro.github.io/kitaq-docs/de/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:de:end -->


**[KITAQFC · HTML-Handbuch](https://bartaro.github.io/kitaq-docs/de/fc-library.html)**

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
- `audio_vblank.h` / `audio_vblank.c` — NMI-Musik mit einer BGM-Warteschlange für sieben Datensätze, Pause und Fortsetzung, einem separaten SFX-Puffer für sieben Datensätze und einmal ausgelösten Rauschhüllkurven. [HTML-Handbuch](https://bartaro.github.io/kitaq-docs/de/fc-library.html#module-audio_vblank)
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
- `physics2d.h` / `physics2d.c` — Integration rechteckiger Körper, Schwerkraft, AABB-Kontakte, Oberflächenreaktion und optionale Q5.3-Datentypen. Zusammen mit `fixed.c` kompilieren.
- `physics3d.h` / `physics3d.c` — Nicht rotierende 3D-Boxen, massengewichteter Rückprall und Aufprallwerte. Zusammen mit `fixed.c` und `physics2d.c` kompilieren.
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

Szenenwechsel, Aktualisierung und Darstellung rufen ihre registrierten Handler synchron auf. Die einzelnen API-Einträge erläutern die Callback-Reihenfolge und die Grenzen der Wiedereintrittsfähigkeit.
