# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **Português (Brasil)**

**[Manual do compilador](https://bartaro.github.io/kitaq-docs/pt/kitaqfc.html)** · **[Manual da biblioteca](https://bartaro.github.io/kitaq-docs/pt/fc-library.html)**

Compilador C e bibliotecas de apoio para programas homebrew originais de NES/Famicom/FDS, derivados de KITAQGB e NORCAL.

Versão de prévia pública: as APIs e o comportamento podem mudar.

## Organização do repositório

```text
kitaqfc/                  # Raiz do repositório
├─ kitaqfc/               # Fontes de compilação do compilador
│  ├─ *.cs
│  ├─ app.config
│  └─ kitaqfc.csproj
├─ kitaqfc.exe            # Compilador pronto, em modo Release
├─ kitaqfc.exe.config     # Configuração do runtime .NET Framework
├─ lib/                   # Bibliotecas de apoio em C
├─ examples/              # Programas didáticos e fonte de caracteres original
├─ scripts/build.ps1      # Reconstrói o executável Release
├─ LICENSE
└─ LICENSE.ja
```

O compilador pronto exige Windows com .NET Framework 4.8. Baixe o ZIP do repositório para manter juntos o executável, sua configuração, as bibliotecas e os avisos de licença. Para recompilar, também são necessários o .NET Framework 4.8 Developer Pack e o Visual Studio Build Tools. A partir da raiz do repositório:

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Uma compilação Release copia o executável e sua configuração para a raiz do repositório. As compilações Debug ficam em `kitaqfc/bin/Debug` e não substituem o compilador Release distribuído. Caches de compilação e arquivos PDB não são distribuídos. Consulte o [registro de compilação binária](BINARY_BUILD.json) para conhecer as entradas e o SHA-256.

## Compilar e começar a usar

Use Windows, .NET Framework 4.8 Developer Pack e Visual Studio Build Tools com MSBuild. Execute a partir de um prompt Developer PowerShell.

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Manuais e licenças

- [Compilador em português](https://bartaro.github.io/kitaq-docs/pt/kitaqfc.html) / [Biblioteca em português](https://bartaro.github.io/kitaq-docs/pt/fc-library.html)
- [Manual em inglês](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [Manual em japonês](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [Fontes do manual para leitura offline](https://github.com/bartaro/kitaq-docs)
- [Licença](LICENSE) / [Tradução de referência em japonês](LICENSE.ja)

A licença do projeto não substitui as condições de terceiros relativas a fontes de caracteres, dependências, logotipos ou marcas. Preserve os avisos incluídos ao redistribuir.
