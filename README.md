# KITAQFC

C compiler and support libraries for original NES/Famicom/FDS homebrew software, derived from KITAQGB and NORCAL.

Public preview: APIs and behavior may change.

## Repository layout

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

The prebuilt compiler requires Windows with .NET Framework 4.8. Download the
repository ZIP to keep the executable, runtime configuration, libraries and
license notices together. Rebuilding additionally requires the .NET Framework
4.8 Developer Pack and Visual Studio Build Tools. From the repository root:

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

A Release build copies the executable and its configuration to the repository
root. Debug builds stay inside `kitaqfc/bin/Debug` and do not overwrite the
distributed Release compiler. Build caches and PDB files are not distributed.
See [binary build record](BINARY_BUILD.json) for the build inputs and SHA-256.

日本語: ビルド用のC#ソースとプロジェクトは `kitaqfc/` にまとめています。
ビルド済みRelease版は直下の `kitaqfc.exe` です。実行には .NET Framework 4.8 が必要です。
リポジトリのZIPを取得すると、設定ファイル・ライブラリ・権利表記も一緒に入手できます。
再ビルドはリポジトリ直下で `.\scripts\build.ps1` を実行してください。

## Build and first use

Windows, .NET Framework 4.8 Developer Pack and Visual Studio Build Tools (MSBuild). Run from a Developer PowerShell prompt.

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Manuals and licenses

- [Japanese HTML manuals](https://bartaro.github.io/kitaq-docs/) / [English manuals](https://bartaro.github.io/kitaq-docs/en/)
- [Offline manual source](https://github.com/bartaro/kitaq-docs)
- [License](LICENSE) / [日本語参考訳](LICENSE.ja)

The project license does not replace third-party font, dependency, logo or trademark terms. Preserve the accompanying notices when redistributing.
