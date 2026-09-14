# KITAQFC

<!-- manual-language-links:start -->
| Language / 言語 | HTML |
| --- | --- |
| English | [KITAQFC](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/en/fc-library.html) |
| 日本語 | [KITAQFC](https://bartaro.github.io/kitaq-docs/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/fc-library.html) |
| 한국어 | [KITAQFC](https://bartaro.github.io/kitaq-docs/ko/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/ko/fc-library.html) |
| 简体中文 | [KITAQFC](https://bartaro.github.io/kitaq-docs/zh-CN/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html) |
| 繁體中文 | [KITAQFC](https://bartaro.github.io/kitaq-docs/zh-TW/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html) |
| Español | [KITAQFC](https://bartaro.github.io/kitaq-docs/es/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/es/fc-library.html) |
| Português (Brasil) | [KITAQFC](https://bartaro.github.io/kitaq-docs/pt/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/pt/fc-library.html) |
| Français | [KITAQFC](https://bartaro.github.io/kitaq-docs/fr/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/fr/fc-library.html) |
| Deutsch | [KITAQFC](https://bartaro.github.io/kitaq-docs/de/kitaqfc.html) · [KITAQFC Library](https://bartaro.github.io/kitaq-docs/de/fc-library.html) |
<!-- manual-language-links:end -->


[English](#english) | [日本語](#japanese) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Español](README.es.md) | [Português (Brasil)](README.pt-BR.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

<a name="english"></a>

## English

C compiler and support libraries for original NES/Famicom/FDS homebrew software, derived from KITAQGB and NORCAL.

Public preview: APIs and behavior may change.

### Repository layout

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

### Build and first use

Windows, .NET Framework 4.8 Developer Pack and Visual Studio Build Tools (MSBuild). Run from a Developer PowerShell prompt.

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

### Manuals and licenses

- [Japanese HTML manuals](https://bartaro.github.io/kitaq-docs/kitaqfc.html) / [English manuals](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)
- [Offline manual source](https://github.com/bartaro/kitaq-docs)
- [License](LICENSE) / [Japanese reference translation](LICENSE.ja)

The project license does not replace third-party font, dependency, logo or trademark terms. Preserve the accompanying notices when redistributing.

---

<a name="japanese"></a>

## 日本語

KITAQGBとNORCALから派生した、自作のNES・ファミコン・FDSソフトウェア向けCコンパイラと支援ライブラリです。

パブリックプレビュー版です。APIや動作は変更される場合があります。

### リポジトリの構成

```text
kitaqfc/                  # リポジトリのルート
├─ kitaqfc/               # コンパイラのビルド用ソース
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # ビルド済みRelease版コンパイラ
├─ kitaqfc.exe.config     # .NET Frameworkの実行設定
├─ lib/                   # C支援ライブラリ
├─ examples/              # 入門プログラムと自作フォント
├─ scripts/build.ps1      # Release版の再ビルド
├─ LICENSE
└─ LICENSE.ja
```

ビルド用のC#ソースとプロジェクトは `kitaqfc/` にまとめています。ビルド済みRelease版はリポジトリ直下の `kitaqfc.exe` です。実行にはWindowsと.NET Framework 4.8が必要です。リポジトリのZIPを取得すると、実行ファイル、設定ファイル、ライブラリ、権利表記をまとめて入手できます。

再ビルドには.NET Framework 4.8 Developer PackとVisual Studio Build Toolsも必要です。リポジトリ直下で実行してください。

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Releaseビルドでは実行ファイルと設定ファイルをリポジトリ直下にコピーします。Debugビルドは `kitaqfc/bin/Debug` に置かれ、配布用Release版を上書きしません。ビルドキャッシュとPDBファイルは配布していません。ビルド入力とSHA-256は[バイナリのビルド記録](BINARY_BUILD.json)を参照してください。

### ビルドと初回利用

Windows、.NET Framework 4.8 Developer Pack、Visual Studio Build Tools（MSBuild）を用意し、Developer PowerShellから実行します。

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

### マニュアルとライセンス

- [日本語HTMLマニュアル](https://bartaro.github.io/kitaq-docs/kitaqfc.html) / [英語HTMLマニュアル](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)
- [オフライン用マニュアルのソース](https://github.com/bartaro/kitaq-docs)
- [ライセンス英語原文](LICENSE) / [日本語参考訳](LICENSE.ja)

プロジェクトのライセンスは、第三者のフォント、依存ライブラリ、ロゴ、商標に関する条件を置き換えるものではありません。再配布時は付属の権利表記も保持してください。
