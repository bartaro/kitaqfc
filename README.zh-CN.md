# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **简体中文**

**[编译器手册](https://bartaro.github.io/kitaq-docs/zh-CN/kitaqfc.html)** · **[库手册](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html)**

面向原创NES/Famicom/FDS自制软件的C编译器及支持库，由KITAQGB和NORCAL衍生而来。

本项目处于公开预览阶段，API和行为可能发生变化。

## 仓库结构

同名子目录 `kitaqfc/` 存放编译器源码、项目文件和构建配置。已构建的Release可执行文件及其运行时配置位于仓库根目录。`lib/` 存放C库，`examples/` 存放入门程序和原创字体。

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

运行所附编译器需要Windows和.NET Framework 4.8。请下载仓库ZIP，将可执行文件、配置、库和许可证声明一起保存。如需重新构建，还需要.NET Framework 4.8 Developer Pack和Visual Studio Build Tools。请在仓库根目录执行：

```powershell
.\scripts\build.ps1
.\kitaqfc.exe --help
.\examples\build.ps1
```

Release构建会将程序和配置复制到仓库根目录。Debug构建保留在 `kitaqfc/bin/Debug`，不会覆盖发布的Release编译器。发布内容不包含构建缓存或PDB文件。构建输入与SHA-256见[二进制构建记录](BINARY_BUILD.json)。

## 从源码构建并开始使用

在Windows中安装.NET Framework 4.8 Developer Pack和Visual Studio Build Tools后，可直接使用MSBuild。请打开Developer PowerShell并执行：

```powershell
MSBuild.exe .\kitaqfc\kitaqfc.csproj /t:Build /p:Configuration=Release
.\kitaqfc.exe --help
.\examples\build.ps1
```

## 手册与许可证

- [简体中文编译器手册](https://bartaro.github.io/kitaq-docs/zh-CN/kitaqfc.html) / [简体中文库手册](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html)
- [英文手册](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [日文手册](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [供离线阅读的手册源码](https://github.com/bartaro/kitaq-docs)
- [许可证](LICENSE) / [日文参考译文](LICENSE.ja)

项目许可证不能替代第三方对字体、依赖库、标志或商标规定的条件。再分发时请保留随附声明。
