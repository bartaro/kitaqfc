# FCSEQ 0.5.1

[English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [Français](README.fr.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Português](README.pt-BR.md)

FCSEQ 是一款用红白机音源创作音乐的编辑器。你可以在基于 plita 的界面中编写音符、调整音色和波形，并随时试听。GUI、CLI 和 Python bindings 共用同一种项目格式和音频引擎。程序从空白项目开始，不依赖特定游戏或 ROM。

## 安装与界面语言

从 [downloads](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq/downloads) 下载适合自己系统的 ZIP，并完整解压。Windows、Linux 提供 x86_64 版本，macOS 提供 Apple Silicon 和 Intel 版本。Windows 运行 `fcseq.exe`，Linux 和 macOS 运行 `./fcseq`。如果缺少执行权限，可运行 `chmod +x fcseq fcseq-cli`。请保留随包附带的字体和许可证文件。

界面支持英语和日语。在 **SOUND → 日本語 / English** 中切换后，程序会记住选择；NSF 播放器的工具栏也有同样的按钮。首次启动会根据系统语言选择，也可通过 `FCSEQ_LANGUAGE=ja` 或 `en` 指定。安装包内含 Noto Sans JP，便于在各系统上显示日文。

Windows 需要 Direct3D 12 驱动和 Microsoft Visual C++ x64 运行库；macOS 使用 Metal。Linux 版本在 Ubuntu 24.04 上构建，使用 ALSA、X11/Wayland 和 Vulkan/OpenGL，较旧的发行版可能需要自行编译。实时播放需要音频设备，导出 WAV 则不需要。程序未做数字签名，macOS 版本也未公证。

## 编写音符与使用键盘编辑器

先在 **SOUND** 中选择音源，再双击轨道空白处添加音符。拖动音符主体可以改变位置和音高，拖动右边缘可以改变长度。每条轨道对应一个硬件声道，因此不能放置互相重叠的音符。

双击轨道名称或按 **E**，即可单独打开这条轨道。键盘编辑器左侧显示钢琴键盘，纵向每格为一个半音，横向为时间。点击琴键可以试听。滚轮移动音域，键盘旁的 −/+ 调整可见音域范围，Shift+滚轮移动时间位置。使用轨道切换箭头可打开其他轨道，再按 E 返回全部轨道。噪声、DPCM 和节奏轨道会显示相应的声音或播放速率范围。

按住 Ctrl/Shift 点击，或在空白处拖出选框，可选择多个音符。Ctrl+A 选择当前轨道；Ctrl+C/X/V 复制、剪切或在光标处粘贴；Ctrl+D 复制一份；Delete 删除。Ctrl+Z/Y 撤销或重做。方向键移动或移调，Shift+上/下移动一个八度。G 改变网格，Q 将音符对齐网格，F 显示整个段落，Home 回到开头，+/− 缩放时间轴。

Space 控制播放与停止。LOOP 重复当前段落，AUDITION 试听选中的音符。静音和独奏也会影响导出。在 SONG 中可添加、命名和调整段落顺序，每个项目最多 16 个段落，每段最多 256 小节。点击速度或小节数即可修改。Ctrl+N/O/S/Shift+S 分别为新建、打开、保存、另存为。恢复文件位于程序旁的 `out/recovery_<process-id>.fcq`；需要时打开它并另存为。

## 音源与音色

基本 2A03 包含两路脉冲波、三角波、噪声和 DPCM，共 5 条轨道。每个项目可选择一种扩展音源。

| 扩展音源 | 增加的声音 | 总轨道数 |
|---|---|---:|
| VRC6 | 两路脉冲波、锯齿波 | 8 |
| VRC7 | 6 路 FM | 11 |
| FDS | 1 路波表 | 6 |
| N163 | 最多 8 路波表 | 13 |
| MMC5 | 两路脉冲波、PCM | 8 |
| SUNSOFT 5B | 3 路 SSG，共用噪声和包络 | 8 |
| EPSM | 3 路 SSG、6 路 FM、6 路节奏 | 20 |
| VT02 | 两路脉冲波、三角波、噪声、PCM | 10 |

如果切换音源或减少声道会丢失音符，程序会拒绝这次修改。请先清空相关轨道，或创建新项目。

选中音符后打开 **INSTRUMENT / WAVEFORM**。脉冲波可以调整占空比，噪声可以调整周期和长短模式，VRC6 锯齿波可以调整累加速率。这些声道使用固定波形，不能任意重画波形。

FDS 提供一条共用的 64 点、6 位波形。N163 使用 4 位波形，可为音符选择波形槽。你可以拖动图形来绘制，也可以从正弦波、脉冲波、锯齿波或三角波预设开始，再反转或循环移位。**CURVE / LINE** 在平滑曲线与折线之间切换。图中的编辑点才是保存给音源的数值；曲线只用于显示插值。这样既能看清正弦波的平滑走势，也能准确编辑量化后的采样值。将指针移近编辑点可查看编号和值。

N163 波形长度必须是 4 的倍数。所有声道共用 `256 − 16 × 活动声道数` 个采样值的内存，8 声道时可用 128 个。空间不足时，可缩短其他波形或减少声道。暂不提供 FDS 调制表和硬件包络编辑。

VRC7 的自定义音色 0 可编辑运算器和包络等参数；1～15 为合成的近似预设。EPSM 提供 16 个可编辑 FM 音色，每个音色有 4 个运算器，以及算法、反馈、包络和调制设置。修改会影响使用同一音色的所有音符。FM 与节奏还可设置声像。当前试听和 WAV 是单声道，导出的寄存器数据会保留声像设置。EPSM 节奏试听使用自行合成的打击乐，不包含 Yamaha ROM 转储；实际 EPSM 设备会播放其内置节奏音。

5B 和 EPSM 的 SSG 可以组合音调、噪声与包络。三个 SSG 声道共用噪声周期和包络发生器，后写入的设置也会影响其他使用该发生器的音符。

## 将音频用作乐器

在 SAMPLES 中导入 WAV、FLAC、Ogg Vorbis 或 MP3，也可以把文件拖入窗口。调整起点、终点、增益、归一化和反向播放。原始音频会保留，因此处理可以重新调整。源音频最长 60 秒，立体声会混合为单声道。

在 DPCM 轨道添加音符，并选择采样和播放速率。AUDITION DPCM 试听转换结果，EXPORT .DMC 保存数据。转换会去除直流分量，并通过低通滤波重采样。单个采样最多 4081 字节，共用采样区最多 16320 字节，FDS 则为 8192 字节。超出时请缩短范围或降低转换速率。采样的转换速率与音符的播放速率是两个独立设置。

MMC5、VT02 的 PCM 轨道也使用同一份音频和处理设置。它们以 4 kHz 流式播放 8 位 PCM，音符音高控制播放速度，不受 DPCM 字节限制。VT02 的 DPCM 和 PCM 共用采样播放单元，所以同一段落只使用其中一种模式。采样循环和自动音高校正尚未提供。

## 项目、NSF 与导出

可编辑项目保存为 `.fcq`。旧 `.fcseq` 文件可改名为 `.fcq` 后打开。音符项目继续使用 `fcseq-2` JSON，也能读取 `fcseq-1`。包含导入音乐的播放项目使用 `fcseq-3`。

打开 NSF、NSF2 或 NSFe 后，选择曲目和时长即可播放。存在时长元数据时按其播放，否则默认 30 秒，可改为 1～600 秒。保存为 FCQ 会嵌入原始音乐和播放设置。导入用于播放，不会把任意 NSF 自动转成可编辑音符。重新导出为同一种容器时会保留原始字节。

导出时选择 `.nsf`、`.nsf2` 或 `.nsfe`，每个段落会成为一个曲目。导出 NSF、NSF2 时还会生成带曲目时长和零淡出的 NSFe。曲目到段落末尾停止，与编辑器的 LOOP 设置分开。播放器支持 NTSC、可按 NTSC 播放的双区域文件、4 KiB 分页和上表音源，也支持 NSF2 的定时器 IRQ、持续运行的 INIT 和禁止调用 PLAY。PAL 专用文件不支持。EPSM 与 NSF2 定时器端口重叠，因此会拒绝同时使用 EPSM 和 NSF2 定时器 IRQ 标志的文件。详见[音频格式说明](docs/AUDIO_FORMATS.md)。

C + DPCM 导出包含 C 驱动、头文件、寄存器流、采样区和放置说明，不会生成完整游戏 ROM。你需要设置目标映射器，保证代码和数据始终能被 CPU 访问。按 `FCSEQ_TICK_RATE_HZ` 调用 `fcseq_tick`：通常为 NTSC 帧率，使用 PCM 的项目为 4000 Hz。KITAQFC 使用时定义 `FCSEQ_KITAQFC`，也可用 `FCSEQ_WRITE(address, value)` 替换寄存器写入操作。

## CLI、Python 与构建

运行 `fcseq-cli help` 查看命令；Windows 文件名加 `.exe`。段落、声道、音符、音色和采样索引从 0 开始，NSF 曲目从 1 开始。

```sh
./fcseq-cli new song.fcq epsm
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf2 song.fcq song.nsf2
./fcseq-cli nsf-render song.nsf2 preview.wav 1 10
./fcseq-cli set-epsm-parameter song.fcq edited.fcq 0 44 4
```

Python 3.10 及以上可加载同包中与系统、CPU 匹配的原生扩展。GUI 和 CLI 不需要 Python。[API 列表](fcseq_native.pyi)包含 `Controller`、`set_sound`、`set_epsm_parameter`、波形编辑和导出。例如依次调用 `c.set_expansion('epsm')`、`c.add_note(0, 8, 0, 8, 69)`、`c.export_nsf('song.nsf2')`，即可创建 FM 音符并导出。

构建需要 Rust 1.93.1 和 C++14 编译器。运行 `cargo build --locked --release`，测试使用 `cargo test --locked --tests`。只构建 CLI 时加 `--no-default-features --bin fcseq-cli`，Python 扩展用 `--features python --lib`。Windows 需要 MSVC Build Tools，macOS 需要 Xcode Command Line Tools；Linux 开发库见仓库的 `.github/workflows/fcseq.yml`。

FCSEQ 应用代码采用 MIT 许可证，Copyright (c) 2026 DAISUKE OBA。ymfm（BSD-3-Clause）、Symphonia（MPL-2.0）和字体（OFL）保留各自许可证。再分发时请保留 LICENSE、THIRD_PARTY_NOTICES.md、licenses/、assets/ 和 third_party_sources/。不含游戏 ROM、FDS BIOS 或商业音源 ROM 转储。导入的音乐与采样另有权利归属。参见[第三方声明](THIRD_PARTY_NOTICES.md)和[验证记录](VERIFICATION.md)。

[Source](https://github.com/bartaro/kitaqfc/tree/main/tools/fcseq)
