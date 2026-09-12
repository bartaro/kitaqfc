# KITAQFC

C compiler and support libraries for original NES/Famicom/FDS homebrew software, derived from KITAQGB and NORCAL.

Public preview: APIs and behavior may change.

## Build and first use

Windows, .NET Framework 4.8 Developer Pack and Visual Studio Build Tools (MSBuild). Run from a Developer PowerShell prompt.

```powershell
MSBuild.exe .\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Manuals and licenses

- [Japanese HTML manuals](https://bartaro.github.io/kitaq-docs/)
- [Offline manual source](https://github.com/bartaro/kitaq-docs)
- [License](LICENSE) / [日本語参考訳](LICENSE.ja)

The project license does not replace third-party font, dependency, logo or trademark terms. Preserve the accompanying notices when redistributing.
