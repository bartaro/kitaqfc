# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **Deutsch**

**[Compiler-Handbuch](https://bartaro.github.io/kitaq-docs/de/kitaqfc.html)** · **[Bibliothekshandbuch](https://bartaro.github.io/kitaq-docs/de/fc-library.html)**

C-Compiler und Hilfsbibliotheken für selbst entwickelte NES-/Famicom-/FDS-Software. KITAQFC ist aus KITAQGB und NORCAL hervorgegangen.

Öffentliche Vorabversion: APIs und Verhalten können sich noch ändern.

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

- [Deutsches Compiler-Handbuch](https://bartaro.github.io/kitaq-docs/de/kitaqfc.html) / [Deutsches Bibliothekshandbuch](https://bartaro.github.io/kitaq-docs/de/fc-library.html)
- [Japanisches Handbuch](https://bartaro.github.io/kitaq-docs/kitaqfc.html) / [Englisches Handbuch](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)
- [Handbuchquellen zum Offline-Lesen](https://github.com/bartaro/kitaq-docs)
- [Lizenz](LICENSE) / [Japanische Übersetzung zur Orientierung](LICENSE.ja)

Die Projektlizenz ersetzt keine Bedingungen Dritter für Schriften, Abhängigkeiten, Logos oder Marken. Bewahren Sie bei einer Weiterverteilung die beiliegenden Hinweise auf.
