# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **Español**

**[Manual del compilador](https://bartaro.github.io/kitaq-docs/es/kitaqfc.html)** · **[Manual de las bibliotecas](https://bartaro.github.io/kitaq-docs/es/fc-library.html)**

Compilador de C y bibliotecas de apoyo para crear software casero original para NES/Famicom/FDS, derivados de KITAQGB y NORCAL.

El proyecto está en fase de versión preliminar pública. Las API y su comportamiento pueden cambiar.

## Organización del repositorio

El subdirectorio homónimo `kitaqfc/` reúne el código fuente del compilador, el archivo de proyecto y la configuración de compilación. El ejecutable Release ya compilado y su configuración de ejecución se encuentran en la raíz. `lib/` contiene las bibliotecas de C y `examples/` los programas introductorios y la tipografía original.

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

El compilador incluido requiere Windows y .NET Framework 4.8. Descargue el ZIP del repositorio para mantener juntos el ejecutable, su configuración, las bibliotecas y los avisos de licencia. Para recompilar también necesita .NET Framework 4.8 Developer Pack y Visual Studio Build Tools. Ejecute lo siguiente desde la raíz del repositorio:

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

La compilación Release copia el programa y su configuración a la raíz. La versión Debug permanece en `kitaqfc/bin/Debug` y no sobrescribe el compilador Release distribuido. No se incluyen cachés de compilación ni archivos PDB. El [registro de compilación del binario](BINARY_BUILD.json) detalla las entradas y sus valores SHA-256.

## Compilar y empezar a usar KITAQFC

Tras instalar .NET Framework 4.8 Developer Pack y Visual Studio Build Tools en Windows, puede utilizar MSBuild directamente. Abra Developer PowerShell y ejecute:

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## Manuales y licencias

- [Compilador: manual en español](https://bartaro.github.io/kitaq-docs/es/kitaqfc.html) / [Bibliotecas: manual en español](https://bartaro.github.io/kitaq-docs/es/fc-library.html)
- [Manual en inglés](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [Manual en japonés](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [Archivos del manual para consultarlo sin conexión](https://github.com/bartaro/kitaq-docs)
- [Licencia](LICENSE) / [Traducción japonesa de referencia](LICENSE.ja)

La licencia del proyecto no sustituye las condiciones de terceros sobre tipografías, dependencias, logotipos o marcas. Conserve los avisos adjuntos al redistribuir el software.
