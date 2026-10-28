# FCSEQ 0.5.2

[English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [Français](README.fr.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Português](README.pt-BR.md)

FCSEQ ist ein Musikeditor für die Klangchips des Famicom. In der mit plita entwickelten Oberfläche setzen Sie Noten, bearbeiten Klänge und hören das Ergebnis direkt an. Oberfläche, CLI und Python-Bindings verwenden dasselbe Projektformat und dieselbe Audio-Engine. Neue Projekte sind leer und enthalten keine Musik aus Spielen.

## Installation und Sprache

Laden Sie unter [downloads](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads) das passende ZIP herunter und entpacken Sie es vollständig. Windows und Linux werden für x86_64 angeboten, macOS für Apple Silicon und Intel. Starten Sie `fcseq.exe` unter Windows oder `./fcseq` unter Linux und macOS. Fehlende Ausführungsrechte setzen Sie mit `chmod +x fcseq fcseq-cli`. Lassen Sie Schriftarten und Lizenzdateien beim Programm.

Die Oberfläche ist auf Englisch und Japanisch verfügbar. **SOUND → 日本語 / English** schaltet die Sprache um; im NSF-Player gibt es dieselbe Schaltfläche. FCSEQ speichert die Auswahl. Beim ersten Start wird die Systemsprache berücksichtigt, sofern sie verfügbar ist. Mit `FCSEQ_LANGUAGE=ja` oder `en` können Sie eine Sprache vorgeben. Noto Sans JP für japanische Texte wird mitgeliefert.

Windows benötigt Direct3D 12 und die Microsoft Visual C++ x64-Laufzeit. macOS verwendet Metal. Die Linux-Binärdateien entstehen unter Ubuntu 24.04 und verwenden ALSA, X11/Wayland und Vulkan/OpenGL. Auf älteren Distributionen kann ein eigener Build nötig sein. Die Wiedergabe benötigt ein Audiogerät, der WAV-Export nicht. Die Programme sind nicht signiert; die Mac-Versionen sind nicht notarisiert.

## Noten und Piano-Roll

Wählen Sie unter SOUND einen Chip. Ein Doppelklick auf eine freie Stelle der Spur legt eine Note an. Ziehen Sie den Notenkörper, um Position und Tonhöhe zu ändern, oder den rechten Rand, um die Länge anzupassen. Jede Spur entspricht einem monophonen Hardwarekanal; überlappende Noten sind nicht zulässig.

Ein Doppelklick auf den Spurnamen oder die Taste E öffnet die einzelne Spur mit Klaviertastatur links. Die Zeilen entsprechen Halbtönen, die waagerechte Achse der Zeit. Klicken Sie auf eine Klaviertaste, um den Ton vorzuhören. Das Mausrad verschiebt den Tonhöhenbereich; −/+ an der Tastatur verändert dessen Umfang. Umschalt+Mausrad verschiebt den Zeitausschnitt. Mit den Spurpfeilen wechseln Sie die Spur, mit E zurück zur Gesamtansicht. Rausch-, DPCM- und Rhythmusspuren zeigen ihre verfügbaren Klänge beziehungsweise Abspielraten.

Mehrere Noten wählen Sie mit Strg/Umschalt+Klick oder einem Auswahlrechteck. Strg+A markiert die aktuelle Spur; Strg+C/X/V kopiert, schneidet aus und fügt am Cursor ein. Strg+D dupliziert, Entf löscht, Strg+Z/Y macht Änderungen rückgängig oder stellt sie wieder her. Pfeiltasten verschieben und transponieren; Umschalt+Auf/Ab transponiert um eine Oktave. G ändert das Raster, Q richtet Noten daran aus, F passt die Ansicht an, Pos1 springt zum Anfang und +/− zoomt die Zeitachse.

Die Leertaste startet oder stoppt die Wiedergabe. LOOP wiederholt den Abschnitt, AUDITION spielt die Auswahl. Stumm- und Soloschaltung wirken auch beim Export. Unter SONG verwalten Sie bis zu 16 Abschnitte mit jeweils 256 Takten. Tempo und Taktzahl lassen sich anklicken und bearbeiten. Strg+N/O/S/Umschalt+S erstellt, öffnet, speichert oder speichert unter neuem Namen. Eine Wiederherstellungsdatei liegt unter `out/recovery_<process-id>.fcq` neben dem Programm; öffnen Sie sie und wählen Sie Speichern unter.

Stumm und Solo lassen sich während der Wiedergabe umschalten, ohne den Abschnitt oder die Schleife neu zu starten. Mehrere Solo-Spuren erklingen gemeinsam; eine stummgeschaltete Spur bleibt auch bei aktiviertem Solo stumm. Das gilt auch beim Vorhören ausgewählter Noten. Der neue Mix wird im Hintergrund berechnet und an der aktuellen Wiedergabeposition übernommen. Kürzlich verwendete Mischungen kommen aus dem Cache. Auch beim Rückgängigmachen oder Wiederherstellen dieser Einstellungen läuft die Wiedergabe weiter.

Die Tempoberechnung sammelt Zeitbruchteile auch bei ganzzahligen BPM. Eine Sechzehntelnote bei 108 BPM dauert etwa 8,35 NTSC-Frames; FCSEQ kombiniert deshalb Abstände von 8 und 9 Frames. Jeder Rasterpunkt weicht höchstens einen halben Wiedergabetakt vom idealen Zeitpunkt ab. PCM verwendet dieselbe Regel bei 4 kHz. Vorschau und Exporte nutzen dieselbe Zeitachse. Die bisherigen Felder frames_per_row/ticks_per_row sind gerundete Näherungen; für Zeitberechnungen dienen die Ereigniszeitpunkte oder average_ticks_per_row.

## Klangchips und Instrumente

Zur 2A03-Grundausstattung gehören zwei Puls-, ein Dreieck-, ein Rausch- und ein DPCM-Kanal. Eine Erweiterung kann hinzukommen: VRC6 mit drei Stimmen, VRC7 mit sechs FM-Stimmen, FDS mit einer Wellentabelle, N163 mit bis zu acht, MMC5 mit zwei Pulsstimmen und PCM, SUNSOFT 5B mit drei SSG-Stimmen, EPSM mit drei SSG-, sechs FM- und sechs Rhythmusstimmen oder VT02 mit zwei Pulsstimmen, Dreieck, Rauschen und PCM. Ein Chipwechsel wird abgelehnt, wenn dadurch Noten verloren gingen. Leeren Sie zuerst die betroffenen Spuren.

Wählen Sie Noten und öffnen Sie INSTRUMENT / WAVEFORM. Pulsstimmen bieten ein Tastverhältnis, Rauschen eine Periode und einen kurzen/langen Modus. Beim VRC6-Sägezahn stellen Sie die Akkumulatorrate ein. Diese Stimmen besitzen keinen frei beschreibbaren Wellenformspeicher.

FDS verwendet eine gemeinsame Welle aus 64 Werten mit 6 Bit. N163 bietet Wellenplätze mit 4 Bit, die pro Note gewählt werden. Zeichnen Sie mit der Maus oder beginnen Sie mit Sinus, Puls, Sägezahn oder Dreieck. Wellen lassen sich invertieren und verschieben. CURVE / LINE wechselt zwischen glatter Kurve und geraden Verbindungen. Die sichtbaren Punkte sind die tatsächlich gespeicherten Werte; die Glättung betrifft nur die Darstellung. Beim Überfahren eines Punktes erscheinen Index und Wert.

N163-Wellenlängen müssen durch vier teilbar sein. Alle Stimmen teilen `256 − 16 × aktive_Kanäle` Werte, also 128 bei acht Kanälen. Kürzen Sie andere Wellen oder verringern Sie die Kanalzahl, wenn der Platz nicht reicht. FDS-Modulationstabelle und Hardwarehüllkurve lassen sich derzeit nicht bearbeiten.

Bei VRC7 ist der benutzerdefinierte Klang 0 editierbar; die Vorgaben 1–15 sind synthetische Annäherungen. EPSM bietet 16 FM-Klänge mit vier Operatoren, Algorithmus, Rückkopplung, Hüllkurven und Modulation. Änderungen gelten für alle Noten, die denselben Klang verwenden. FM- und Rhythmusstimmen haben Panoramaeinstellungen. Der Export erhält diese Registerwerte, während Vorschau und WAV derzeit mono sind. Die EPSM-Schlagzeugklänge werden aus eigenen Daten synthetisiert; ein Yamaha-ROM ist nicht enthalten. EPSM-Hardware verwendet ihre eigenen eingebauten Klänge.

Die drei SSG-Stimmen von 5B beziehungsweise EPSM teilen sich Rauschperiode und Hüllkurvengenerator. Eine spätere Einstellung verändert daher auch andere Stimmen, die denselben Generator nutzen.

## Audiodateien als Instrumente

Importieren Sie unter SAMPLES WAV, FLAC, Ogg Vorbis oder MP3 oder ziehen Sie eine Datei ins Fenster. Anfang, Ende, Verstärkung, Normalisierung und Rückwärtswiedergabe sind einstellbar. Die Originaldaten bleiben erhalten. Stereodateien werden mono gemischt; Quelldateien dürfen bis zu 60 Sekunden lang sein.

Wählen Sie bei einer DPCM-Note das Sample und die Abspielrate. AUDITION DPCM spielt das codierte Ergebnis, EXPORT .DMC speichert es. Die Umwandlung entfernt den Gleichanteil und verwendet einen Tiefpass beim Resampling. Ein Sample darf 4081 Byte, die gemeinsame Bank 16320 Byte enthalten; bei FDS sind es 8192 Byte. Kürzen Sie den Ausschnitt oder senken Sie die Umwandlungsrate. Diese Rate ist unabhängig von der Abspielrate der einzelnen Note.

MMC5- und VT02-PCM nutzen dieselben Audiodaten und Bearbeitungsfunktionen, geben aber 8-Bit-PCM mit 4 kHz aus. Die Tonhöhe verändert die Abspielgeschwindigkeit. Die DPCM-Größenbegrenzung gilt dafür nicht. VT02 teilt seine Sampleeinheit zwischen DPCM und PCM; verwenden Sie pro Abschnitt nur einen dieser Modi. Sampleloops und automatische Tonhöhenkorrektur sind nicht enthalten.

## Speichern und exportieren

Bearbeitbare Projekte tragen `.fcq`. Alte `.fcseq`-Dateien können vor dem Öffnen umbenannt werden. Notenprojekte verwenden `fcseq-2`; `fcseq-1` wird ebenfalls gelesen. Wiedergabeprojekte mit eingebettetem NSF verwenden `fcseq-3`.

NSF, NSF2 und NSFe öffnen Sie über FILE, per Drag-and-drop oder als Startargument. Wählen Sie Teilstück und Wiedergabedauer, dann PLAY. Vorhandene Zeitangaben werden übernommen; sonst gelten zunächst 30 Sekunden, einstellbar von 1 bis 600. Beim Speichern als FCQ werden Originaldaten und Einstellungen eingebettet. Der Import wandelt beliebige NSF-Dateien nicht in editierbare Noten um. Beim Export in denselben Container bleiben die ursprünglichen Bytes erhalten.

Wählen Sie beim Export `.nsf`, `.nsf2` oder `.nsfe`. Jeder Abschnitt wird ein Teilstück. NSF und NSF2 erzeugen zusätzlich ein NSFe mit Längenangaben und ohne Ausblenden. Die Wiedergabe endet an der Abschnittsgrenze; LOOP im Editor wird getrennt behandelt.

Unterstützt werden NTSC und Mehrregionendateien mit NTSC-Timing, 4-KiB-Banking sowie NSF2-Timer-IRQ, Hintergrund-INIT und unterdrücktes PLAY. Reine PAL-Dateien werden abgewiesen. EPSM und der NSF2-Timer belegen dieselben Ports; EPSM zusammen mit gesetztem NSF2-Timer-IRQ-Flag wird deshalb abgewiesen. Weitere Grenzen stehen in [AUDIO_FORMATS](docs/AUDIO_FORMATS.md).

Export C + DPCM erzeugt Registerdaten, C-Treiber, Header, Samplebank und Platzierungsbericht. Eine vollständige Spiel-ROM entsteht damit nicht. Das Zielprogramm muss den Mapper konfigurieren und die Daten für die CPU erreichbar halten. Rufen Sie `fcseq_tick` mit `FCSEQ_TICK_RATE_HZ` auf: normalerweise NTSC-Bildrate, bei PCM 4000 Hz. `FCSEQ_KITAQFC` wählt den KITAQFC-Compiler; `FCSEQ_WRITE` bindet einen eigenen Registerzugriff ein.

## CLI, Python und eigener Build

`fcseq-cli help` zeigt die Befehle. Unter Windows kommt `.exe` hinzu. Indizes beginnen bei 0, NSF-Teilstücknummern bei 1.

```sh
./fcseq-cli new song.fcq epsm
./fcseq-cli validate song.fcq
./fcseq-cli export-nsf2 song.fcq song.nsf2
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli import-nsf music.nsf2 imported.fcq 1 60
```

Die mitgelieferte Erweiterung benötigt Python 3.10 oder neuer für dasselbe Betriebssystem und dieselbe Architektur. GUI und CLI benötigen Python nicht. Die API einschließlich `set_sound` und `set_epsm_parameter` ist in [fcseq_native.pyi](fcseq_native.pyi) beschrieben.

Zum Bauen benötigen Sie Rust 1.93.1 und einen C++14-Compiler. Verwenden Sie `cargo build --locked --release` und `cargo test --locked --tests`. Nur die CLI bauen Sie mit `--no-default-features --bin fcseq-cli`, Python mit `--features python --lib`. Windows benötigt MSVC Build Tools, macOS die Xcode Command Line Tools. Linux-Pakete stehen in `.github/workflows/fcseq.yml`.

FCSEQ steht unter MIT, Copyright (c) 2026 DAISUKE OBA. Abhängigkeiten behalten ihre Lizenzen, darunter BSD-3-Clause für ymfm, MPL-2.0 für Symphonia und OFL für Schriftarten. Geben Sie LICENSE, THIRD_PARTY_NOTICES.md, licenses/, assets/ und third_party_sources/ mit weiter. Spiel-ROMs, FDS-BIOS und kommerzielle Instrument-ROMs sind nicht enthalten. Importierte Musik und Samples behalten ihre eigenen Rechte. Siehe [Hinweise](THIRD_PARTY_NOTICES.md), [Lizenzprüfung](LICENSE_AUDIT.json) und [Verifikation](VERIFICATION.md).

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
