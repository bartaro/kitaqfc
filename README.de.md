# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **Deutsch**

**Compiler-Handbuch** · **Bibliothekshandbuch**

C-Compiler und Hilfsbibliotheken für selbst entwickelte NES-/Famicom-/FDS-Software. KITAQFC ist aus KITAQGB und NORCAL hervorgegangen.

Öffentliche Vorabversion: APIs und Verhalten können sich noch ändern.

## Entwicklungsgrundsätze

KITAQFC unterstützt eine Entwicklung, die die Eigenschaften der NES-/Famicom-Hardware berücksichtigt und sich in kleinen, reproduzierbaren Schritten überprüfen lässt. Erstellen Sie das ROM, führen Sie es in KUROSAKI aus, analysieren Sie die Ergebnisse mit SARAKURA und wiederholen Sie die Tests nach jeder Korrektur. Halten Sie gemessene Ergebnisse und noch nicht getestetes Verhalten getrennt fest.

<!-- development-prompt:de:start -->
## Prompt zur Spieleentwicklung

Tragen Sie die Anforderungen ein und geben Sie den vollständigen Prompt an Ihre KI weiter. Er umfasst die Implementierung, Emulator-Tests, die Analyse mit SARAKURA und die erneute Prüfung nach Korrekturen.

Das Praxisbeispiel im HTML-Handbuch lesen

<details>
<summary>Vollständigen Prompt anzeigen</summary>

### Spieleentwicklung mit KITAQFC, KUROSAKI und SARAKURA

Tragen Sie die Anforderungen ein und geben Sie dieses gesamte Dokument an die KI weiter. Die Befehle setzen voraus, dass die Repositories `kitaqgb`, `kitaqfc`, `kokura`, `kurosaki`, `sarakura` und `kitaq-docs` sowie das Projekt `game-gb` oder `game-fc` im selben übergeordneten Ordner liegen. Führen Sie die Befehle dort aus und passen Sie die Pfade an die tatsächliche Umgebung an.

#### Anforderungen

- Spieltitel: &lt;ausfüllen&gt;
- Genre und zentrale Spielmechanik: &lt;ausfüllen&gt;
- Steuerung sowie Erfolgs- und Misserfolgsbedingungen: &lt;ausfüllen&gt;
- Erforderliche Bildschirme, Level, Gegner und Gegenstände: &lt;ausfüllen&gt;
- Grafikstil, Musik und Geräusche: &lt;ausfüllen; bereitgestellte Dateien angeben&gt;
- Speichern, Kommunikation, Zusatzgeräte und weitere Anforderungen: &lt;ausfüllen oder keine&gt;
- Projektordner: &lt;ausfüllen&gt;
- Bedingungen für die Weitergabe: &lt;etwa eigener Code und eigene Medien, die unter MIT veröffentlicht werden können&gt;

- Ziel: &lt;NES/Famicom-Steckmodul oder FDS&gt;
- Mapper, ROM-Größe und Mirroring: &lt;festlegen oder anhand der Anforderungen wählen&gt;
- Videonorm und Leistungsziel: &lt;etwa NTSC und 60 Aktualisierungen der Spiellogik pro Sekunde&gt;

#### Arbeitsauftrag

Setzen Sie das Spiel mit KITAQFC und seinen Bibliotheken um. Verwenden Sie KUROSAKI zum Ausführen und Debuggen und SARAKURA zum Auswerten der Diagnosen sowie zum Vergleich vor und nach einer Korrektur.

Wiederholen Sie diesen Ablauf, bis die Abnahmekriterien erfüllt sind: Spezifikation konkretisieren → kleine Änderung umsetzen → bauen → Eingaben ausführen und beobachten → Ursache untersuchen → korrigieren → unter gleichen Bedingungen erneut testen. Ein Plan, ausgegebener Quellcode oder ein erfolgreicher Compilerlauf allein schließen den Auftrag nicht ab.

##### Umgebung und Abnahmekriterien klären

1. Lesen Sie die Anweisungen im Arbeitsverzeichnis, READMEs, HTML-Handbücher sowie Header und Implementierungen der verwendeten Bibliotheken. Erfassen Sie die Pfade der Programme und ihre Versionen oder SHA-256-Werte. Prüfen Sie Befehle anhand der tatsächlichen `--help`-Ausgabe und APIs anhand des Quellcodes.
2. Legen Sie überprüfbare Kriterien für Eingaben, Bild, Ton, Spielverlauf und Aktualisierungsrate fest. Beispiele: START drücken und loslassen beginnt das Spiel; eine Kollision kostet ein Leben; die Pause schaltet die vorgesehenen Töne stumm, Fortsetzen nimmt die Wiedergabe wieder auf.
3. Fragen Sie nur bei wesentlichen Unklarheiten nach. Treffen Sie übliche, rückgängig zu machende Implementierungsentscheidungen selbstständig. Schwächen Sie Anforderungen und Abnahmekriterien nicht ab.
4. Führen Sie zunächst ein kleines mitgeliefertes Beispiel durch Compiler, Emulator und SARAKURA. Das prüft deren Zusammenspiel, nicht die Fertigstellung des beauftragten Spiels.

##### Eine kleine spielbare Fassung umsetzen

- Ziehen Sie für ein kleines Spiel NROM in Betracht. Wählen Sie MMC3 oder einen anderen Mapper, wenn Größe oder Bankwechsel dies erfordern. Prüfen Sie benötigte Platinenfunktionen mit `inspect-rom`, `mapper-info`, `audit-board` und dem Quellcode; der Mappername allein belegt keine Unterstützung.
- Planen Sie PRG/CHR-Größe, CHR-ROM oder CHR-RAM, Mirroring, feste Bänke, Interruptvektoren und Speicher-RAM für Spielstände. Vergleichen Sie nach Optimierungen oder Bankänderungen Header und tatsächliche Belegung. `--nes-local-ram` verwendet internes CPU-RAM in `$0000–$07FF`; vermeiden Sie Überschneidungen mit Zero Page, Stack, OAM-Puffern und Laufzeit- oder Bibliotheksbereichen.
- Nutzen Sie den C-Dialekt von KITAQFC, FC-Bibliotheken und `void main(void)`. Setzen Sie keine Kompatibilität mit GB-APIs voraus. Manche Headereinträge sind reine Deklarationen: suchen Sie die Implementierungen und binden Sie die erforderlichen `.c`-Dateien ein.
- Berücksichtigen Sie PPU-Register, NMI, OAM DMA, Spritegrenzen pro Bildzeile, Scrolling, Mirroring, Attributtabellen und APU/DMC-Verhalten. Freier Warteschlangenplatz ist nicht die VRAM-Kapazität der PPU; begrenzen Sie die Arbeit je NMI.
- Wandeln Sie die bereitgestellte eigene Schrift `ascii.c` in FC-CHR um und prüfen Sie CHR, Paletten, Namens- und Attributtabellen. Bei FDS sind Plattenzugriff, Speichern und BIOS-Anforderungen gesondert zu prüfen; setzen Sie keine Startbedingungen eines Steckmoduls voraus.

- Verbinden Sie zunächst Start, Titelbild, steuerbare Spielfigur, Erfolg oder Misserfolg und Neustart. Erweitern Sie danach den Inhalt.
- Bewahren Sie bearbeitbare Originale von Grafik, Musik und Geräuschen sowie deren Erzeugungsschritte auf. Prüfen Sie, dass der Build tatsächlich die exportierten Daten verwendet.
- Schreiben Sie Codekommentare auf Englisch und Fortschrittsberichte auf Deutsch. Lassen Sie die Standardberichte von SARAKURA auf Englisch.

##### Jeden Build seiner Ausführung zuordnen

Trennen Sie Ausgaben nach Iteration, etwa mit `out/iter-001`. Protokollieren Sie Befehle, Rückgabecodes und Hashes von Quellcode, Medien, Werkzeugen, ROM und Metadaten. Führen Sie nach einem fehlgeschlagenen Build niemals eine alte ROM aus. Speicherbelegungspläne, Quellcodezuordnungen und Debuginformationen müssen aus demselben Build wie die ROM stammen.

Das folgende Beispiel prüft NROM ohne Eingaben. Stellen Sie `main.c`, erforderliche Implementierungen und die CHR-Datei bereit; wählen Sie den passenden Mapper. Übergeben Sie Compiler-Buildmetadaten nicht an KUROSAKIs `--kitaqfc-debug`, ohne das erwartete Format zu prüfen.

```powershell
$iteration = '.\game-fc\out\iter-001'
New-Item -ItemType Directory -Force $iteration | Out-Null

# Include all additional implementation units required by the game.
& '.\kitaqfc\kitaqfc.exe' '.\game-fc\src\main.c' `
  -I '.\kitaqfc\lib' -o "$iteration\game.nes" `
  --mapper=nrom '--nes-chr=.\game-fc\assets\game.chr' --no-disasm `
  "--kurosaki-metadata=$iteration\build.json"
if ($LASTEXITCODE -ne 0) { throw 'Build failed; inspect the build log.' }

& '.\kurosaki\kurosaki.exe' run "$iteration\game.nes" `
  --frames 300 --pad1 0 --png "$iteration\frame.png" `
  --json "$iteration\run.json" --emit-diagnostics "$iteration\events.jsonl"
if ($LASTEXITCODE -ne 0) { throw 'Emulator run failed; inspect the run log.' }

& '.\sarakura\sarakura.exe' fc analyze `
  --metadata "$iteration\build.json" --events "$iteration\events.jsonl" `
  --frames 300 --out "$iteration\analysis" --fail-on error
if ($LASTEXITCODE -ne 0) { throw 'Inspect the analysis report and fix the cause.' }
```


300 Frames ohne Eingaben sind nur eine erste Kontrolle. Ergänzen Sie Spielszenarien mit aufeinanderfolgenden Aktionen, bevor Sie das Spiel als funktionsfähig einstufen.

##### Geordnete Eingaben reproduzieren

- `--pad1` und `--pad2` verwenden rohe NES-Bitmasken: A=1, B=2, SELECT=4, START=8, UP=16, DOWN=32, LEFT=64, RIGHT=128. Verwechseln Sie diese nicht mit den `BTN_*`-Werten der Bibliothek.
- `run --pad1` setzt eine feste Eingabe. Erstellen Sie für Aktionsfolgen ein Replay, das Drücken, Halten und Loslassen unterscheidet. Lesen Sie die Definitionen von `Replay` und `ReplayFrame`. `replay-record` in der öffentlichen CLI zeichnet neutrale Eingaben auf, keine von einem Menschen gespielte Partie.
- Prüfen Sie ROM-SHA-256, Replay-Zuordnung und Framebereich selbst. `replay-run --verify` vergleicht einen erwarteten Endhash nur, wenn dieser vorhanden ist; es prüft weder Spielverhalten noch ROM-Identität umfassend. Überschreiben Sie Sollwerte nicht ungeprüft mit Istwerten, nur damit ein Test besteht.

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


Bereiten Sie das Replay für die getestete ROM vor. Die öffentliche CLI bietet bei `replay-run` kein `--emit-diagnostics`. Erfinden Sie diese Option nicht und geben Sie CPU-Traces nicht als Diagnoseereignisse weiter. Um geordnete Eingaben mit SARAKURA zu analysieren, erstellen Sie im Projekt ein Testprogramm mit den öffentlichen `kurosaki-core`-APIs `RunOptions.replay_frames` und `diagnostic_events_from_trace_and_report`. Verwenden Sie dieselbe ROM, dasselbe Replay und dieselben Framebedingungen; erzeugen Sie JSONL aus Trace und Diagnosebericht genau dieses Laufs. Prüfen Sie Tracekonfiguration und gespeicherten Bereich und vergleichen Sie Bilder und Beobachtungen des Testprogramms mit dem CLI-Replay. Stellen Sie Diagnosen ohne Eingaben nicht als Nachweis für ein Spielszenario dar. Fehlt die erforderliche Umgebung, kennzeichnen Sie diese Prüfung als unvollständig.

##### Bild, Ton, Zustand und Leistung prüfen

- Speichern Sie Eingabeszenarien mit getrenntem Drücken, Halten und Loslassen. Durchlaufen Sie alle vorgesehenen Wege: Start, Spielbeginn, Bewegung, Aktionen, Kollisionen, Scrolling, Levelwechsel, Spielende, Neustart, Pause und gegebenenfalls Speichern oder Kommunikation.
- Bewahren Sie PNGs relevanter Frames, Eingaben, Laufberichte, Diagnose-JSONL, WAVs und nötige Zustands- oder Speicherbeobachtungen auf. Prüfen Sie erreichte Frames und Abbruchgrund. Öffnen Sie die Bilder tatsächlich; ein einzelner Screenshot belegt weder Bewegung noch Eingabereaktion. Vergleichen Sie Zähler, Positionen und Zustandswechsel mit Sollwerten; prüfen Sie Bildschirmränder, Tile- und Attributgrenzen und Szenen mit vielen Sprites.
- Prüfen Sie Musik, Geräusche, gleichzeitige Wiedergabe, Aussetzer, Pause und Fortsetzen. Eine erzeugte WAV-Datei allein belegt keinen korrekten Klang. Ist Anhören nicht möglich, trennen Sie ausgeführte Wellenform- und Zahlenprüfungen von unbestätigten Höreigenschaften.
- Messen Sie aufwendige Szenen, Arbeit der Ziel-CPU, Spielaktualisierungen und Transfers; auf FC auch die NMI-Arbeit. Emulatordurchsatz auf dem Host ist weder Spielaktualisierungsrate noch Nachweis für reale Hardwaregeschwindigkeit. Fortsetzen mit `--allow-unimplemented` belegt keine Unterstützung der fehlenden Funktion.

##### Analysieren, korrigieren und erneut testen

- Übergeben Sie SARAKURA die Buildmetadaten der geprüften ROM und Diagnose-JSONL aus dem zugehörigen Lauf. Ein CPU-Trace oder gewöhnlicher Laufbericht ersetzt dies nicht. `--frames` legt Analysebedingungen fest; SARAKURA führt weder ROMs aus noch ändert es automatisch Quellcode.
- Lesen Sie `report.html`, `ai_diagnostics.json`, `repair_prompt.md` und `retest_plan.json`. Gleichen Sie Diagnosen mit Reproduktionsschritten, Bildern, Ton und Quellcode ab. Trennen Sie vermutete Quellstellen oder Ursachen von bestätigten Tatsachen und normale Warteschleifen von Hängern. Bewerten Sie Warnungen einzeln und dokumentieren Sie nicht unterstützte Ereignisse sowie Analysegrenzen. Verbergen Sie Warnungen nicht durch Filter und verkürzen Sie Tests nicht, um ein Bestehen zu erreichen.
- Reduzieren Sie Fehler auf minimale Reproduktionen, beheben Sie die Ursache und bauen Sie neu. Liegt der Fehler im Compiler oder Emulator, grenzen Sie ihn vom Spielcode ab und ergänzen Sie eine Regressionsprüfung für die Werkzeugkorrektur.
- Wiederholen Sie Tests mit gleichen Eingaben, Zufallsstartwerten, Hardware- und Videomodi, Mappern, Beobachtungsframes und Diagnoseeinstellungen. Nutzen Sie für jede ROM passende Metadaten; verwenden Sie nach Änderungen an Code oder RAM-Belegung nicht blind alte Speicherzustände weiter.

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


Nutzen Sie Diagnosedifferenzen gemeinsam mit den Abnahmekriterien für Steuerung, Grafik und Ton. Wiederholt sich derselbe Fehler, überprüfen Sie Belege und Hypothese, statt beliebige weitere Änderungen vorzunehmen.

##### Abschlusskriterien und Lieferumfang

Führen Sie alle Pflichtszenarien erneut mit der finalen ROM aus, die aus den gelieferten Quellen und Einstellungen gebaut wurde. Unverwundbarkeit, automatische Testeingaben oder ein anderer Mapper allein prüfen kein normales Spiel in der Endfassung. Liefern Sie eine Zuordnung von Anforderungen zu Tests, Gründe für verbleibende Warnungen und klare Angaben zu ungeprüften oder nicht unterstützten Punkten. Kennzeichnen Sie ausdrücklich, wenn nicht auf physischer Hardware getestet wurde.

Liefern Sie Quellcode, Kennungen von Werkzeugen und Bibliotheken, bearbeitbare Medien, reproduzierbare Build- und Testskripte, ROM, abschließende Nachweise und eine README mit Einrichtung, Steuerung und bekannten Grenzen. Fügen Sie bei Bedarf Replays und das Testprogramm hinzu. Veröffentlichen oder versenden Sie Dateien nur im ausdrücklich erlaubten Umfang. Entfernen Sie unnötige Zwischenbuilds und temporäre Traces nach der Prüfung, bewahren Sie jedoch Quellen, Medien, Endergebnisse und erforderliche Regressionsnachweise auf.

Verhindern Umgebung oder Berechtigungen eine Pflichtprüfung, nennen Sie die genauen Reproduktionsschritte und die nötige Maßnahme. Kennzeichnen Sie die Arbeit nicht als abgeschlossen.

</details>
<!-- development-prompt:de:end -->

## Aufbau des Repositorys

Der gleichnamige Unterordner `kitaqfc/` enthält die Compilerquellen, die Projektdatei und die Build-Konfiguration. Die fertige Release-Version liegt zusammen mit ihrer Laufzeitkonfiguration im Stammverzeichnis. `lib/` enthält die C-Bibliotheken, `examples/` die Lernprogramme und die Originalschrift.

```text
kitaqfc/                  # Repository root
├─ kitaqfc/               # Compiler build sources
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # Prebuilt Release compiler
├─ kitaqfc.exe.config     # .NET Framework runtime configuration
├─ lib/                # C support libraries
├─ examples/           # Tutorial programs and original font
├─ scripts/build.ps1   # Rebuild the Release executable
├─ LICENSE
└─ LICENSE.ja
```

Zum Ausführen des mitgelieferten Compilers benötigen Sie Windows mit .NET Framework 4.8. Laden Sie das Repository als ZIP herunter, damit Programm, Konfiguration, Bibliotheken und Lizenzhinweise zusammenbleiben. Für einen eigenen Build benötigen Sie zusätzlich das .NET Framework 4.8 Developer Pack und Visual Studio Build Tools. Führen Sie im Stammverzeichnis aus:

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Ein Release-Build kopiert Programm und Konfiguration ins Stammverzeichnis. Debug-Builds verbleiben unter `kitaqfc/bin/Debug` und überschreiben die bereitgestellte Release-Version nicht. Build-Caches und PDB-Dateien werden nicht mitgeliefert. Der [Build-Nachweis](BINARY_BUILD.json) enthält Eingabedaten und SHA-256-Prüfsummen.

## Selbst kompilieren und starten

Unter Windows mit .NET Framework 4.8 Developer Pack und Visual Studio Build Tools können Sie MSBuild direkt verwenden. Öffnen Sie dazu eine Developer PowerShell:

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Handbücher und Lizenzen

- Deutsches Compiler-Handbuch / Deutsches Bibliothekshandbuch
- [Japanisches Handbuch](https://bartaro.github.io/kitaq-docs/kitaqfc.html) / [Englisches Handbuch](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)
- [Handbuchquellen zum Offline-Lesen](https://github.com/bartaro/kitaq-docs)
- [Lizenz](LICENSE) / [Japanische Übersetzung zur Orientierung](LICENSE.ja)

Die Projektlizenz ersetzt keine Bedingungen Dritter für Schriften, Abhängigkeiten, Logos oder Marken. Bewahren Sie bei einer Weiterverteilung die beiliegenden Hinweise auf.
