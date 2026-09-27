# ZX0-kompatible Ressourcenkompression

<!-- readme-language-links:start -->
[English](README.md#english) | [日本語](README.md#%E6%97%A5%E6%9C%AC%E8%AA%9E) | [한국어](README.ko.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | **Deutsch**
<!-- readme-language-links:end -->

[API und Beispiele](https://bartaro.github.io/kitaq-docs/de/fc-library.html#module-zx0)

Der PC-Kompressor und der FC-Dekompressor sind eigenständige KITAQ-Implementierungen für vorwärts gelesene ZX0-v2-Datenströme. Die KITAQ-Implementierung steht unter der MIT-Lizenz, Copyright (c) 2026 DAISUKE OBA.

Das ZX0-Format und der ursprüngliche Kompressionsalgorithmus wurden von [Einar Saukas](https://github.com/einar-saukas/ZX0) entworfen. Diese Würdigung des Formats ist von Urheberrecht und Lizenz der KITAQ-Implementierung getrennt. Siehe [LICENSE](../../LICENSE) und [LICENSE.ja](../../LICENSE.ja).

Bauen Sie das PC-Werkzeug im Stammverzeichnis des Repositorys:

```powershell
.\tools\zx0\build.ps1
.\kitaqfc-zx0.exe input.bin output.zx0
.\kitaqfc-zx0.exe input.bin asset.h --header=level_data
.\kitaqfc-zx0.exe output.zx0 restored.bin --decompress
```

Das PC-Werkzeug verwendet .NET Framework 4.x. Der Kompressor nutzt eine begrenzte Hashketten-Suche und garantiert nicht die kleinstmögliche Ausgabe. Rückwärtsströme, externe Präfixwörterbücher und ZX0 v1 sind nicht Teil dieser Schnittstelle. Die Rechte an den ursprünglichen Ressourcen verbleiben bei ihren Urhebern.

Eingabe und kodierte Ausgabe dürfen jeweils höchstens 65535 Byte umfassen. `--format=auto` vergleicht raw-, RLE- und ZX0-Nutzlasten und versieht die kleinste mit einem neun Byte langen KQA1-Header; dieser zählt zur Ausgabegrenze. Verwenden Sie `asset_decompress` für KQA1. Teilen Sie Ressourcen außerdem passend zu den tatsächlichen CPU-Bankfenstern und der RAM-Kapazität auf. Ein leerer raw-C-Header enthält ein Platzhalterbyte bei logischer `_SIZE` 0.

Binden Sie `zx0.h` ein und kompilieren Sie `lib/zx0.c` für das Ziel. `zx0_decompress` erhält Zieladresse, Zielkapazität, komprimierte Quelle und deren Größe. Prüfen Sie sowohl die zurückgegebene Bytezahl als auch `zx0_error`. Ein Fehler kann eine Teilausgabe hinterlassen; zeigen oder verwenden Sie diese nach einem Fehlschlag nicht. Quell- und Zielpuffer dürfen sich weder überlappen noch Grenzen der aktuell eingeblendeten CPU-Bankfenster überschreiten. Die Routinen verwenden gemeinsamen Arbeitsspeicher und dürfen während ihrer Ausführung nicht aus Interrupts erneut aufgerufen werden.

`zx0_decompress_vram` benötigt einen RAM-Arbeitsbereich für die vollständig entpackte Ressource. Bei ausgeschaltetem Rendering entpackt es dort die Daten und überträgt sie in CHR-RAM oder Nametable-Speicher. PPUCTRL bleibt erhalten; setzen Sie die Scrollposition vor dem erneuten Einschalten der Darstellung. Paletten werden nicht übertragen und CHR-ROM wird nicht beschrieben.
