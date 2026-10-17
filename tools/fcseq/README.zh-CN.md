# FCSEQ 0.4.1 — 使用说明

[en](README.en.md) · [ja](README.ja.md) · [ko](README.ko.md) · [zh-CN](README.zh-CN.md) · [zh-TW](README.zh-TW.md) · [fr](README.fr.md) · [es](README.es.md) · [de](README.de.md) · [pt-BR](README.pt-BR.md)

作者 **DAISUKE OBA**。基于 **plita** 的通用红白机音乐音序器。GUI、CLI、Python 绑定共用项目模型、音频生成和导出功能。启动时是空项目，不需要游戏 ROM 或 BIOS。界面标签为英语，说明书提供九种语言。

## 下载与启动

从 [downloads/](downloads/) 下载完整 ZIP：Windows x86_64、Linux x86_64、macOS Apple Silicon ARM64 或 Intel x86_64。请完整解压，保留资源与许可证。Windows 启动 `fcseq.exe`，Unix 运行 `./fcseq`；命令行工具是 `fcseq-cli`（Windows 加 `.exe`）。[bin/](bin/) 也提供单个可执行文件。Windows 10/11 使用 Direct3D 12，macOS 使用 Metal，Linux 使用 Vulkan/OpenGL、X11/Wayland 和 ALSA。Linux 在 Ubuntu 24.04 上构建，旧发行版可能需要自行编译。Ubuntu 运行库：`sudo apt install libasound2t64 libxkbcommon0 libwayland-client0 libx11-6 libxcursor1 libxi6 libxrandr2 libgl1`。GUI 需要兼容的图形驱动，试听需要音频设备；导出 WAV 不需要。程序未签名，macOS 版本未公证。若系统拦截，请核对源码与哈希，再使用系统正常的允许操作。必要时运行 `chmod +x fcseq fcseq-cli`。

## 作曲与编辑

在 SOUND 中选择 2A03（Pulse 1/2、Triangle、Noise、DPCM，共5声）、VRC6（增加2个 Pulse 和 Saw，共8声）、VRC7（增加6个 FM，共11声）、FDS（增加1个波形声部，共6声）、N163（增加1~8个波形声部，最多13声）。每个项目只用一种扩展配置；会丢失音符的切换或减声部操作将被拒绝。

双击添加音符，拖动本体移动或移调，拖动右边缘改变长度。Ctrl/Shift 单击或框选多选，Ctrl+A 选中当前轨道。Ctrl+C/X/V 复制、剪切、在光标处粘贴；Ctrl+D 复制副本；Delete 删除；Ctrl+Z/Y 撤销/重做。方向键移动/移调，Shift+上下改变八度。G 网格、Q 量化、F 全景、Home 开头、+/- 缩放、Shift+滚轮移动时间、UP/DOWN 切换可见声部。Space 播放/停止、LOOP 循环、AUDITION 试听选中音符。MUTE/SOLO 同样影响导出。SONG 管理最多16个段落，每段256小节；点击名称、BPM、小节数编辑。Ctrl+N/O/S/Shift+S 新建/打开/保存/另存为。非法批量操作整体撤销。恢复文件位于程序旁 `out/recovery_<process-id>.fcq`，打开后另存。

## 音色与波形

打开 SOUND > Instrument / waveform editor，选中该声部音符后编辑，使用 AUDITION/UNDO/REDO。Pulse 占空比4级，VRC6 Pulse 8级；Triangle 是固定32级波形；Noise 有16个周期及长/短模式。Pulse/Noise 音量0~15，衰减0表示恒定音量。VRC6 Saw 的累加增量0~63，溢出也会改变音色。这些音源没有可自由绘制的波形 RAM。

FDS 有一个共享64采样点、6位波形。N163 为4位波形，初始4×32点，每个音符可选择波形编号，长度为4的倍数。共享容量为 `256 - 16 × 活跃声部数` 点（8声时128点）；增加长度前需缩短其他波形或减少声部。支持拖动绘制、SINE/PULSE/SAW/TRIANGLE、反转和旋转，并检查音高/RAM/声部限制。VRC7 自定义音色0可编辑24项参数（倍频、ADSR、反馈、音量、键标度、颤音/颤频、正弦/半波）或8字节。预设1~15是独立合成的近似音色，不是音色 ROM 转储。没有 FDS 调制表/硬件包络专用编辑器。模拟音不保证与实机模拟输出一致。

## DPCM

SAMPLES 或 SOUND > Import audio file 导入 WAV/FLAC/Ogg Vorbis/MP3，也支持拖放。设置 START/END（毫秒）、GAIN、NORMALIZE、REVERSE、RATE 0~15，通过 AUDITION DPCM 试听，EXPORT .DMC 保存。向 DPCM 轨道添加音符，在检查器选择 SAMPLE 与播放 RATE。保留原始 PCM，处理可恢复；输入最多60秒，立体声转单声道，转换时去直流并低通重采样。每个音色 `16n+1` 字节、最大4081字节，最多64个，总库16320字节（FDS 8192）。超限操作被拒绝。转换采样率与音符播放速率分别设置。无音色内部循环及自动音高校正。

## 文件与导出

项目保存为 `.fcq`，旧 `.fcseq` 可改名读取。音符项目是 `fcseq-2` JSON，也支持 `fcseq-1`；NSF 播放文档是 `fcseq-3`。通过 Import NSF、OPEN、拖放或启动参数打开 NSF/NSFe。PREV/NEXT 或编号选曲，PLAY/STOP、LOOP、1~600秒控制播放。有曲长信息时采用它，否则默认30秒。播放前音频生成可取消。FCQ 内嵌原文件、选曲与时长；不会自动转录成音符。重新导出保留原始容器字节。

Export NSF 将每段变成一首子曲，包含音源和 DPCM；同时生成带曲长及淡出0的 NSFe。不识别 NSF 曲长的播放器请打开 NSFe。段落结束时停止，编辑器 LOOP 单独生效。输入支持 NSF v1/NSFe、NTSC 或兼容 NTSC 的双区、无分库或4KiB分库、2A03/DPCM、VRC6/VRC7/FDS/N163。拒绝 PAL 专用、NSF2 执行扩展、MMC5、Sunsoft 5B、VT02；不返回的 INIT/PLAY 有执行时间限制。

Export C + DPCM 输出 `score.c`、`fcseq_score.h`、`fcseq_player.c`、段落二进制、`.dmc`、`dpcm_bank.bin`、`fcseq_export.json`。不自动生成完整游戏 ROM；需自行配置 mapper 与 CPU 可访问的代码/数据/DPCM 地址。VRC6 假定 mapper 24（26需转换地址线）、VRC7 为85、N163 为19，或 FDS。KITAQFC 中在 include 前定义 `FCSEQ_KITAQFC`；`FCSEQ_WRITE(address, value)` 可接入自定义写寄存器函数。

## CLI 与 Python

以下示例在解压目录执行。段落/声部/音符/样本编号从0开始，NSF 曲号从1开始。附带 Python 3.10+ 的 abi3 扩展，须匹配 OS/CPU 并从同目录加载。GUI/CLI 不需要 Python。完整 API 见 `fcseq_native.pyi`。CLI 支持音符移动、移调、改长度、量化、复制、删除；波形通过 GUI/Python 编辑。

```sh
./fcseq-cli new song.fcq n163
./fcseq-cli validate song.fcq
./fcseq-cli render-wav song.fcq song.wav 0
./fcseq-cli export-nsf song.fcq song.nsf
./fcseq-cli export-nsfe song.fcq song.nsfe
./fcseq-cli import-nsf song.nsf imported.fcq 1 60
./fcseq-cli nsf-info song.nsfe
./fcseq-cli nsf-render song.nsfe preview.wav 1 10
./fcseq-cli play-nsf song.nsfe 1 30
./fcseq-cli import-audio song.fcq voice.wav with_voice.fcq
./fcseq-cli export-dpcm with_voice.fcq 0 voice.dmc
./fcseq-cli edit song.fcq edited.fcq 0 0 0 transpose 12
./fcseq-cli export-kitaqfc song.fcq export
python fcseq.py nsf-render song.nsf preview.wav --song 1 --seconds 10
```

```python
import fcseq_native
c = fcseq_native.Controller()
c.set_expansion('n163')
c.set_n163_channels(8)
c.set_n163_wave(0, [i // 2 for i in range(32)])
c.add_note(0, 5, 0, 8, 69)
c.save_project('example.fcq')
c.export_nsf('example.nsf')
c.render_wav('example.wav')
```

## 构建与验证

在此源码目录使用 Rust 1.93.1 和以下命令。本地库源码在 vendor/。Linux 开发包见仓库根目录 `.github/workflows/fcseq.yml`；macOS 需要 Xcode Command Line Tools，Windows 需要 MSVC Build Tools。Python 为可选。脚本验证原生 CLI/Python、WAV、NSF/NSFe 往返及 GUI 启动。结果和范围见 VERIFICATION.md；不认证人工鼠标/键盘操作或实机听音。

```sh
cargo build --locked --release
cargo test --locked --release --tests
cargo build --locked --release --no-default-features --bin fcseq-cli
cargo build --locked --release --features python
```

## 许可证

FCSEQ 为 MIT，Copyright (c) 2026 DAISUKE OBA。依赖保留各自许可证。再分发请包含 LICENSE、THIRD_PARTY_NOTICES.md、licenses/、assets/、third_party_sources/。已附 MPL-2.0 Symphonia 源码及字体/图标声明。导入音乐与音频的权利另行处理。

[LICENSE](LICENSE) · [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) · [LICENSE_AUDIT](LICENSE_AUDIT.json) · [VERIFICATION](VERIFICATION.md)
