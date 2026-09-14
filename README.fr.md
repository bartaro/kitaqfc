# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **Français**

**[Manuel du compilateur](https://bartaro.github.io/kitaq-docs/fr/kitaqfc.html)** · **[Manuel de la bibliothèque](https://bartaro.github.io/kitaq-docs/fr/fc-library.html)**

Compilateur C et bibliothèques pour créer des logiciels homebrew originaux NES/Famicom/FDS, issus de KITAQGB et de NORCAL.

Version publique préliminaire : les API et leur comportement peuvent évoluer.

## Organisation du dépôt

```text
kitaqfc/                  # Racine du dépôt
├─ kitaqfc/               # Sources nécessaires à la compilation du compilateur
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # Compilateur prêt à l'emploi, en mode Release
├─ kitaqfc.exe.config     # Configuration du runtime .NET Framework
├─ lib/                   # Bibliothèques C
├─ examples/              # Programmes pédagogiques et police originale
├─ scripts/build.ps1      # Reconstruction de l'exécutable Release
├─ LICENSE
└─ LICENSE.ja
```

Le compilateur prêt à l'emploi nécessite Windows avec .NET Framework 4.8. Téléchargez l'archive ZIP du dépôt pour conserver ensemble l'exécutable, sa configuration, les bibliothèques et les mentions de licence. Pour recompiler, il faut également le Developer Pack .NET Framework 4.8 et Visual Studio Build Tools. Depuis la racine du dépôt :

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Une compilation Release copie l'exécutable et sa configuration à la racine du dépôt. Les compilations Debug se trouvent dans `kitaqfc/bin/Debug` et ne remplacent pas le compilateur Release distribué. Les caches de compilation et fichiers PDB ne sont pas distribués. Consultez le [compte rendu de compilation binaire](BINARY_BUILD.json) pour les entrées et les empreintes SHA-256.

## Compiler et commencer à utiliser l'outil

Utilisez Windows, le Developer Pack .NET Framework 4.8 et Visual Studio Build Tools avec MSBuild. Lancez les commandes depuis une invite Developer PowerShell.

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Manuels et licences

- [Compilateur en français](https://bartaro.github.io/kitaq-docs/fr/kitaqfc.html) / [Bibliothèque en français](https://bartaro.github.io/kitaq-docs/fr/fc-library.html)
- [Manuel en anglais](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [Manuel en japonais](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [Sources du manuel pour lecture hors connexion](https://github.com/bartaro/kitaq-docs)
- [Licence](LICENSE) / [Traduction japonaise à titre de référence](LICENSE.ja)

La licence du projet ne remplace pas les conditions de tiers relatives aux polices, dépendances, logos ou marques. Conservez les mentions jointes lors de la redistribution.
