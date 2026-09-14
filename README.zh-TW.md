# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **繁體中文**

**[編譯器手冊](https://bartaro.github.io/kitaq-docs/zh-TW/kitaqfc.html)** · **[程式庫手冊](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html)**

用於開發原創 NES／Famicom／FDS 自製軟體的 C 編譯器與支援程式庫，源自 KITAQGB 與 NORCAL。

本專案目前為公開預覽版，API 與行為仍可能調整。

## 儲存庫結構

同名子目錄 `kitaqfc/` 集中存放編譯器原始碼、專案檔與建置設定。已建置的 Release 執行檔及其執行階段設定檔位於根目錄。`lib/` 為 C 程式庫，`examples/` 則提供入門程式與原創字型。

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

隨附編譯器須在 Windows 與 .NET Framework 4.8 環境執行。請下載整個儲存庫的 ZIP，將執行檔、設定、程式庫與授權聲明保存在一起。若要自行建置，還須安裝 .NET Framework 4.8 Developer Pack 和 Visual Studio Build Tools。請在儲存庫根目錄執行：

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Release 建置會將程式與設定檔複製到根目錄。Debug 版本保留在 `kitaqfc/bin/Debug`，不會覆寫發行用的 Release 編譯器。發行內容不包含建置快取或 PDB 檔。建置輸入與 SHA-256 請見[二進位檔建置紀錄](BINARY_BUILD.json)。

## 從原始碼建置與開始使用

在 Windows 安裝 .NET Framework 4.8 Developer Pack 與 Visual Studio Build Tools 後，也可以直接呼叫 MSBuild。請開啟 Developer PowerShell 並執行：

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## 手冊與授權

- [繁體中文編譯器手冊](https://bartaro.github.io/kitaq-docs/zh-TW/kitaqfc.html)／[繁體中文程式庫手冊](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html)
- [英文手冊](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html)／[日文手冊](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [可供離線閱讀的手冊原始檔](https://github.com/bartaro/kitaq-docs)
- [授權條款](LICENSE)／[日文參考譯文](LICENSE.ja)

本專案的授權不取代第三方對字型、相依套件、標誌或商標訂定的條件。再散布時，請一併保留隨附聲明。
