# KITAQFC

[English](README.md#english) | [日本語](README.md#japanese) | **简体中文**

**编译器手册** · **库手册**

面向原创NES/Famicom/FDS自制软件的C编译器及支持库，由KITAQGB和NORCAL衍生而来。

本项目处于公开预览阶段，API和行为可能发生变化。

## 开发理念

KITAQFC 注重结合 NES／红白机的硬件特性，以可复现的小步骤推进开发和验证。构建 ROM 后，在 KUROSAKI 中运行，用 SARAKURA 分析运行结果，并在每次修正后重新验证。记录时应区分实测结果与尚未测试的行为。

<!-- development-prompt:zh-CN:start -->
## 游戏开发提示词

填写需求后，将完整提示词交给 AI。内容涵盖实现、模拟器测试、SARAKURA 分析以及修复后的复测。

阅读 HTML 手册中的参考示例

<details>
<summary>展开完整提示词</summary>

### 使用 KITAQFC、KUROSAKI 和 SARAKURA 开发游戏

填写需求后，将本文完整交给 AI。命令假定 `kitaqgb`、`kitaqfc`、`kokura`、`kurosaki`、`sarakura`、`kitaq-docs` 仓库与 `game-gb` 或 `game-fc` 项目位于同一父目录。请从该父目录运行，并按实际环境调整路径。

#### 需求

- 游戏名称：&lt;填写&gt;
- 类型与核心玩法：&lt;填写&gt;
- 操作方式及成功、失败条件：&lt;填写&gt;
- 必需的界面、关卡、敌人和道具：&lt;填写&gt;
- 画面风格、背景音乐和音效：&lt;填写，并注明所提供素材的路径&gt;
- 存档、通信、外设及其他要求：&lt;填写，或无&gt;
- 项目目录：&lt;填写&gt;
- 再分发要求：&lt;例如，自编代码和原创素材可按 MIT 许可公开&gt;

- 目标：&lt;NES/红白机卡带或 FDS&gt;
- Mapper、ROM 大小与镜像方式：&lt;指定，或根据需求选择&gt;
- 视频制式与性能目标：&lt;例如 NTSC，每秒更新游戏逻辑 60 次&gt;

#### 请执行的任务

请使用 KITAQFC 及其库实现游戏。使用 KUROSAKI 运行和调试，使用 SARAKURA 整理诊断并比较修复前后的结果。

不断重复以下过程，直到满足验收标准：明确规格 → 实现一个小改动 → 构建 → 输入操作并观察 → 调查原因 → 修复 → 在相同条件下复测。不能以写出计划、提供代码或编译成功作为完成依据。

##### 确认环境和验收标准

1. 阅读工作目录的说明、各工具的 README、HTML 手册，以及所用库的头文件和实现。记录可执行文件路径及版本或 SHA-256；以实际 `--help` 输出核对命令，以源码核对 API。
2. 为输入、画面、声音、游戏进程和更新频率制定可判断的验收标准。例如，按下并松开 START 后开始游戏；碰撞扣除一条生命；暂停时指定声音静音，恢复后继续播放。
3. 只就重要歧义提问，常规、可撤销的实现决策请自主推进。不得擅自降低需求或验收标准。
4. 先用一个随附的小示例走通编译器、模拟器和 SARAKURA。它只能证明工具之间能衔接，不能代表所需游戏已经完成。

##### 先实现一个可玩的最小流程

- 小型游戏先考虑 NROM；若规模或分库需求需要，再选择 MMC3 等 Mapper。不能仅凭名称认定支持情况，应通过 `inspect-rom`、`mapper-info`、`audit-board` 和实现检查所需卡带板功能。
- 规划 PRG/CHR 大小、CHR-ROM 或 CHR-RAM、镜像、固定库、中断向量和存档 RAM。优化或更改分库后，核对头部与实际布局。`--nes-local-ram` 使用 CPU 内部 RAM `$0000–$07FF`；避免与零页、栈、OAM 缓冲区及运行时、库占用区域重叠。
- 使用 KITAQFC 的 C 方言、FC 库和 `void main(void)`。不要假定 GB API 兼容；有些头文件条目只有声明，需检查实现位置并纳入所需 `.c` 文件。
- 考虑 PPU 寄存器、NMI、OAM DMA、每扫描线精灵上限、滚动、镜像、属性表和 APU/DMC 行为。队列剩余容量不等于 PPU VRAM 容量，应限制每次 NMI 的工作量。
- 将提供的原创 `ascii.c` 字体转换为 FC CHR，检查 CHR、调色板、名称表和属性数据。FDS 的磁盘访问、存档和 BIOS 条件需单独验证，不可套用卡带的启动条件。

- 先连通启动、标题界面、可控制角色、成功或失败及重新开始，再扩充内容。
- 保留可编辑的图形、音乐、音效源文件及生成步骤，确认构建实际读取了导出数据。
- 源码注释用英文，进度报告用简体中文。SARAKURA 的标准报告保持英文。

##### 将每次构建与运行对应起来

使用 `out/iter-001` 等目录区分每轮输出。记录命令、退出码及源码、素材、工具、ROM、元数据的哈希。构建失败后，不得运行遗留的旧 ROM。映射文件、源码映射和调试信息必须与 ROM 来自同一次构建。

下面是无输入的 NROM 检查示例。请准备 `main.c`、所需库实现和 CHR 文件，并选择合适的 Mapper。未经格式核对，不要把编译器构建元数据传给 KUROSAKI 的 `--kitaqfc-debug`。

```powershell
$iteration = '.\game-fc\out\iter-001'
New-Item -ItemType Directory -Force $iteration | Out-Null

# Include all additional implementation units required by the game.
& '.\kitaqfc\kitaqfc.exe' '.\game-fc\src\main.c' `
  -I '.\kitaqfc\lib' -o "$iteration\game.nes" `
  --mapper=nrom '--nes-chr=.\game-fc\assets\game.chr' --no-disasm `
  "--kurosaki-metadata=$iteration\build.json"
if ($LASTEXITCODE -ne 0) { throw 'Build failed; inspect the build log.' }

& '.\kurosaki\kurosaki.exe' run "$iteration\game.nes" `
  --frames 300 --pad1 0 --png "$iteration\frame.png" `
  --json "$iteration\run.json" --emit-diagnostics "$iteration\events.jsonl"
if ($LASTEXITCODE -ne 0) { throw 'Emulator run failed; inspect the run log.' }

& '.\sarakura\sarakura.exe' fc analyze `
  --metadata "$iteration\build.json" --events "$iteration\events.jsonl" `
  --frames 300 --out "$iteration\analysis" --fail-on error
if ($LASTEXITCODE -ne 0) { throw 'Inspect the analysis report and fix the cause.' }
```


无输入运行 300 帧只是初步检查。在认定游戏正常之前，必须加入有先后顺序的游玩场景。

##### 复现有先后顺序的输入

- `--pad1` 和 `--pad2` 使用 NES 原始位掩码：A=1、B=2、SELECT=4、START=8、UP=16、DOWN=32、LEFT=64、RIGHT=128。不要与库中的 `BTN_*` 值混淆。
- `run --pad1` 是固定输入。连续操作应使用回放，并区分按下、按住和松开。阅读 `Replay`、`ReplayFrame` 定义。公开 CLI 的 `replay-record` 记录的是无输入状态，不是玩家的交互操作。
- 自行核对 ROM SHA-256、回放目标和帧范围。`replay-run --verify` 仅在提供预期最终哈希时进行比较，并不全面验证游戏行为或 ROM 对应关系。不能只为让测试通过，就把预期值无条件改成实测值。

```powershell
& '.\kurosaki\kurosaki.exe' replay-run `
  '.\game-fc\out\iter-001\game.nes' '.\game-fc\tests\start-and-play.replay.json' `
  --frames 900 --json '.\game-fc\out\iter-001\play.json' `
  --png '.\game-fc\out\iter-001\play.png' `
  --wav '.\game-fc\out\iter-001\play.wav'
```


为被测 ROM 准备对应回放。公开 CLI 的 `replay-run` 没有 `--emit-diagnostics` 选项。不要编造选项，也不要将 CPU 跟踪当作诊断事件传入。若要用 SARAKURA 分析顺序输入场景，应在项目内编写测试驱动程序，使用公开 `kurosaki-core` 的 `RunOptions.replay_frames` 和 `diagnostic_events_from_trace_and_report`。以相同 ROM、回放和帧条件运行，从该次执行的跟踪与诊断报告生成 JSONL。检查跟踪配置、保留范围，并将驱动程序的画面和观测结果与 CLI 回放对照。不得把无输入运行的诊断当作游玩场景的证据。缺少所需环境时，应将这项验证明确列为未完成。

##### 检查画面、声音、状态和性能

- 保存输入场景，区分按下、按住和松开。覆盖规格中的全部路径：启动、开始、移动、动作、碰撞、滚动、关卡切换、游戏结束、重新开始、暂停，以及适用的存档和通信。
- 保存关键帧 PNG、输入、运行报告、诊断 JSONL、WAV 和必要的状态、内存观测。检查实际到达帧数与停止原因。真正打开图像查看；一张截图不能证明运动或输入响应。将计数器、坐标和状态变化与预期值对照，同时检查画面边缘、图块和属性边界及精灵密集场景。
- 检查音乐、音效、同时发声、断音、暂停和恢复。仅生成 WAV 不能证明声音正确。无法试听时，应区分已完成的波形、数值检查与尚未确认的听感。
- 测量高负载场景的目标 CPU 工作量、游戏更新及传输量，FC 还需计入 NMI 工作。宿主机上模拟器的运行速度不等于游戏更新频率，也不是实机速度证明。使用 `--allow-unimplemented` 后能继续运行，不代表未实现功能已受支持。

##### 分析、修复并复测

- 向 SARAKURA 提交被测 ROM 的构建元数据和该次运行的诊断 JSONL。CPU 跟踪或普通运行报告不能替代它。`--frames` 指定分析条件；SARAKURA 不执行 ROM，也不自动修改源码。
- 阅读 `report.html`、`ai_diagnostics.json`、`repair_prompt.md`、`retest_plan.json`，与复现步骤、画面、声音和源码核对。区分推测的源码位置、原因与已确认事实，并区分正常等待循环与卡死。逐项判断警告，记录未支持事件和分析范围限制。不要通过过滤警告或缩短测试来获得通过结果。
- 把问题缩减为最小复现，修复原因后重新构建。若根因在编译器或模拟器，应与游戏代码问题分离，并为工具修复补充回归验证。
- 复测时保持输入、随机种子、机型和视频制式、Mapper、观测帧和诊断设置一致。新 ROM 使用对应元数据；代码或 RAM 布局变化后，不得盲目复用即时存档。

```powershell
& '.\sarakura\sarakura.exe' baseline-delta `
  --baseline '.\game-fc\out\iter-001\analysis' `
  --current '.\game-fc\out\iter-002\analysis' `
  --out '.\game-fc\out\delta.json' --markdown '.\game-fc\out\delta.md' `
  --fail-on-new error --fail-on-regression error --enforce
```


诊断差异应与操作、画面和声音的验收结果结合使用。相同失败反复出现时，应重审证据与假设，不要无依据地继续修改。

##### 完成条件与交付内容

用交付源码和设置构建最终 ROM，再执行全部必需场景。仅使用无敌状态、自动测试输入或另一种 Mapper，不能证明最终版本的正常游玩。提供需求与测试对应表，说明剩余警告的原因，明确未验证、未支持项目。未做实机测试时标注“实机未验证”。

交付源码、工具和库的标识信息、可编辑素材、可复现的构建与测试脚本、ROM、最终验证证据，以及说明安装、操作和已知限制的 README。按需附上回放和测试驱动程序。仅在明确授权范围内发布或向外部发送文件。验证后删除不必要的中间构建和临时跟踪，但保留源码、素材、最终成果及必要的回归证据。

若环境或权限阻碍必需检查，应报告准确的复现步骤和所需操作，不得标记为完成。

</details>
<!-- development-prompt:zh-CN:end -->

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

- 简体中文编译器手册 / 简体中文库手册
- [英文手册](https://bartaro.github.io/kitaq-docs/en/kitaqfc.html) / [日文手册](https://bartaro.github.io/kitaq-docs/kitaqfc.html)
- [供离线阅读的手册源码](https://github.com/bartaro/kitaq-docs)
- [许可证](LICENSE) / [日文参考译文](LICENSE.ja)

项目许可证不能替代第三方对字体、依赖库、标志或商标规定的条件。再分发时请保留随附声明。
