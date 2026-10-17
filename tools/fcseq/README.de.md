# FCSEQ 0.4.1 — Handbuch

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

Autor: **DAISUKE OBA**. Allgemeiner Famicom-Musiksequencer mit **plita**. GUI, CLI und Python-Bindings verwenden dasselbe Projektmodell, dieselbe Klangerzeugung und dieselben Exporte. Der Start erfolgt mit einem leeren Projekt; Spiel-ROM und BIOS sind nicht erforderlich. Die Oberfläche ist englisch, das Handbuch liegt in neun Sprachen vor.

## Installation

Laden Sie das vollständige ZIP aus [downloads/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads): Windows x86_64, Linux x86_64, macOS Apple Silicon ARM64 oder Intel x86_64. Alle Dateien einschließlich Ressourcen und Lizenzen entpacken. Start: `fcseq.exe` unter Windows, `./fcseq` unter Unix; Terminal: `fcseq-cli` mit `.exe` unter Windows. Einzeldateien stehen auch in [bin/](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/bin). Windows 10/11 nutzt Direct3D 12, macOS Metal, Linux Vulkan/OpenGL, X11/Wayland und ALSA. Linux wurde unter Ubuntu 24.04 gebaut; ältere Systeme benötigen eventuell einen eigenen Build. Ubuntu-Laufzeitpakete: `sudo apt install libasound2t64 libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`. GUI benötigt einen passenden Grafiktreiber, Wiedergabe ein Audiogerät; WAV-Export funktioniert ohne Audiogerät. Programme sind unsigniert, macOS-Versionen nicht notarisiert. Bei einer Sperre Quellen und Prüfsummen prüfen und die normale Freigabe des Betriebssystems verwenden. Bei Bedarf: `chmod +x fcseq fcseq-cli`.

Windows benötigt außerdem [Microsoft Visual C++ x64 Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) (`VCRUNTIME140.dll`). Falls es fehlt, das aktuell unterstützte x64-Paket von Microsoft installieren. GUI/CLI sind nicht mit Python verknüpft; nur optionale Bindings und das Startskript benötigen Python.

## Bearbeitung

SOUND wählt 2A03 mit Pulse 1/2, Triangle, Noise und DPCM (5 Stimmen), VRC6 (+2 Pulse und Saw, 8), VRC7 (+6 FM, 11), FDS (+1 Welle, 6) oder N163 (+1–8 Wellen, bis 13). Pro Projekt ist eine Erweiterung aktiv. Änderungen mit Notenverlust werden abgelehnt.

Doppelklick fügt eine Note ein. Ziehen am Körper verschiebt/transponiert, am rechten Rand ändert es die Länge. Ctrl/Shift-Klick oder Auswahlrechteck markiert mehrere Noten, Ctrl+A die aktuelle Spur. Ctrl+C/X/V kopiert/schneidet/fügt am Cursor ein, Ctrl+D dupliziert, Delete löscht, Ctrl+Z/Y macht rückgängig/wiederholt. Pfeile bewegen/transponieren; Shift+hoch/runter ändert die Oktave. G: Raster, Q: Quantisierung, F: Gesamtansicht, Home: Anfang, +/-: Zoom, Shift+Mausrad: Zeit, UP/DOWN: Kanäle. Space startet/stoppt, LOOP wiederholt, AUDITION spielt die Auswahl. MUTE/SOLO wirkt auch beim Export. SONG verwaltet bis 16 Abschnitte mit je 256 Takten. Namen, BPM und Taktzahl anklicken. Ctrl+N/O/S/Shift+S: neu/öffnen/speichern/speichern unter. Ungültige Sammeländerungen werden vollständig verworfen. Wiederherstellung: `out/recovery_<process-id>.fcq` neben dem Programm öffnen und unter neuem Namen speichern.

## Instrumente

SOUND > Instrument / waveform editor öffnen, Noten des Kanals auswählen und Parameter bearbeiten; AUDITION/UNDO/REDO sind verfügbar. Pulse hat 4 Tastverhältnisse, VRC6 Pulse 8; Triangle ist eine feste 32-stufige Welle. Noise: 16 Perioden, langer/kurzer Modus. Pulse/Noise-Lautstärke 0–15, Abklingen 0 hält den Pegel. VRC6 Saw: Akkumulatorzuwachs 0–63; Überlauf verändert den Klang. Diese Quellen haben keinen frei beschreibbaren Wellenspeicher.

FDS: eine gemeinsame 64-Sample-Welle mit 6 Bit. N163: 4-Bit-Wellen, anfangs 4 × 32 Samples; jede Note wählt einen Platz, Längen sind Vielfache von 4. Gemeinsamer Speicher: `256 - 16 × aktive Kanäle` Samples, also 128 bei acht Stimmen. Vor einer Verlängerung andere Wellen kürzen oder Kanäle reduzieren. Zeichnen, SINE/PULSE/SAW/TRIANGLE, Invertieren und Rotieren sind möglich. Tonhöhen-, RAM- und Kanalgrenzen werden geprüft. VRC7-Patch 0 bietet 24 Parameter (Multiplikatoren, ADSR, Rückkopplung, Pegel, Tastenskalierung, Tremolo/Vibrato, Sinus/Halbsinus) oder 8 Rohbytes. Presets 1–15 sind synthetische Näherungen, keine ROM-Kopien. Kein eigener Editor für FDS-Modulation/Hardwarehüllkurven. Analoge Hardwaregleichheit ist nicht zugesichert.

## DPCM

SAMPLES oder SOUND > Import audio file lädt WAV/FLAC/Ogg Vorbis/MP3; Drag-and-drop ist möglich. START/END in ms, GAIN, NORMALIZE, REVERSE, RATE 0–15 einstellen; AUDITION DPCM abhören, EXPORT .DMC speichern. DPCM-Note setzen, SAMPLE und Wiedergabe-RATE wählen. Original-PCM bleibt erhalten; maximal 60 s, Stereo wird Mono, Gleichanteil wird entfernt und mit Tiefpass neu abgetastet. Samplegröße `16n+1` Bytes, höchstens 4081; maximal 64 Samples und 16320 Bankbytes (FDS 8192). Überschreitungen werden abgelehnt. Umwandlungsrate und Noten-Abspielrate sind getrennt. Keine internen Sampleschleifen oder automatische Tonhöhenkorrektur.

## Dateien und Export

Projekte als `.fcq` speichern; alte `.fcseq` umbenennen. Notenprojekte verwenden JSON `fcseq-2`, `fcseq-1` wird gelesen; NSF-Wiedergabedokumente verwenden `fcseq-3`. NSF/NSFe über Import NSF, OPEN, Dateiabwurf oder Startargument öffnen. PREV/NEXT oder Nummer wählt das Stück; PLAY/STOP, LOOP und 1–600 s steuern die Wiedergabe. Zeitmetadaten werden berücksichtigt, sonst gelten zunächst 30 s. Die vorausgehende Audioberechnung kann abgebrochen werden. FCQ bettet Original, Stückwahl und Dauer ein; es erfolgt keine automatische Notentranskription. Reexport erhält die Originalbytes.

Export NSF erzeugt pro Abschnitt ein Teilstück mit Klangdaten/DPCM und zusätzlich NSFe mit Dauer und Fade 0. NSFe verwenden, wenn der Player NSF-Zeiten ignoriert. Wiedergabe endet am Abschnittsende, unabhängig von Editor-LOOP. Eingabe: NSF v1/NSFe, NTSC oder NTSC-kompatible Doppelregion, ohne Banken oder 4-KiB-Banken; 2A03/DPCM, VRC6/VRC7/FDS/N163. Nur-PAL, NSF2-Ausführungserweiterungen, MMC5, Sunsoft 5B und VT02 werden abgelehnt; nicht zurückkehrendes INIT/PLAY wird begrenzt.

Export C + DPCM schreibt `score.c`, `fcseq_score.h`, `fcseq_player.c`, Abschnittsbinärdateien, `.dmc`, `dpcm_bank.bin`, `fcseq_export.json`. Mapper und CPU-sichtbare Code-/Daten-/DPCM-Adressen müssen integriert werden; keine vollständige Spiel-ROM. VRC6 setzt Mapper 24 voraus (26 benötigt Adressanpassung), VRC7 85, N163 19 oder FDS. Für KITAQFC vor dem Einbinden `FCSEQ_KITAQFC` definieren; `FCSEQ_WRITE(address, value)` bindet eigene Registerzugriffe an.

## CLI und Python

Beispiele im entpackten Ordner ausführen. Abschnitts-/Kanal-/Noten-/Sampleindizes beginnen bei 0, NSF-Stücke bei 1. Python 3.10+ nutzt die mitgelieferte abi3-Erweiterung für passendes Betriebssystem/CPU. GUI/CLI benötigen kein Python. Vollständige API: `fcseq_native.pyi`. CLI bearbeitet Noten durch Verschieben, Transponieren, Längenänderung, Quantisieren, Duplizieren und Löschen; Wellenbearbeitung erfolgt über GUI/Python.

```sh
./fcseq-cli new song.fcq n163
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf song.fcq song.nsf
./fcseq-cli export-nsfe song.fcq song.nsfe
./fcseq-cli import-nsf song.nsf imported.fcq 1 60
./fcseq-cli nsf-info song.nsfe
./fcseq-cli nsf-render song.nsfe preview.wav 1 10
./fcseq-cli play-nsf song.nsfe 1 30
./fcseq-cli import-audio song.fcq voice.wav with_voice.fcq
./fcseq-cli export-dpcm with_voice.fcq 0 voice.dmc
./fcseq-cli edit song.fcq edited.fcq 0 0 0 transpose 12
./fcseq-cli export-kitaqfc song.fcq export
python fcseq.py nsf-render song.nsf preview.wav --song 1 --seconds 10
```

```python
import fcseq_native
c = fcseq_native.Controller()
c.set_expansion('n163')
c.set_n163_channels(8)
c.set_n163_wave(0, [i // 2 for i in range(32)])
c.add_note(0, 5, 0, 8, 69)
c.save_project('example.fcq')
c.export_nsf('example.nsf')
c.render_wav('example.wav')
```

## Bauen und prüfen

In diesem Quellordner Rust 1.93.1 und folgende Befehle verwenden. Lokale Bibliotheken liegen in vendor/. Linux-Entwicklungspakete: `.github/workflows/fcseq.yml` im Repository-Hauptordner; macOS: Xcode Command Line Tools; Windows: MSVC Build Tools. Python ist optional. Skripte prüfen native CLI/Python, WAV, NSF/NSFe-Rundläufe und GUI-Start. Ergebnisse/Grenzen: VERIFICATION.md. Menschliche Maus-/Tastaturbedienung und Klang echter Hardware sind nicht zertifiziert.

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python --lib
python -c "import shutil,sys; from pathlib import Path; n={'win32':'fcseq_native.dll','darwin':'libfcseq_native.dylib'}.get(sys.platform,'libfcseq_native.so'); shutil.copy2(Path('target/release')/n,'fcseq_native.pyd' if sys.platform=='win32' else 'fcseq_native.so')"
```

## Lizenz

FCSEQ steht unter MIT, Copyright (c) 2026 DAISUKE OBA. Abhängigkeiten behalten ihre Lizenzen. LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ und third_party_sources/ mitverteilen. Symphonia-Quellen unter MPL-2.0 sowie Schrift-/Iconhinweise sind enthalten. Importierte Musik und Samples haben eigene Rechte.

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
