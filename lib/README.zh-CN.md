# FC/NES标准库结构

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | **简体中文** | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md) | [Português (Brasil)](README.pt-BR.md)
<!-- readme-language-links:end -->

<!-- fc-current-20261004:zh-CN:start -->
### 当前编译器与库的发布配置 — 2026年10月4日

fc.h 包含 physics3d 和 audio_vblank 的声明，所需实现文件仍须单独链接。entity_update_all 和 entity_draw_all 按 ID 升序将活动槽位传给回调。请将回调放在公共库区0，或在调用期间保持相应 PRG 库区的映射。使用 ZX0 时须显式包含 zx0.h。audio_vblank 的记录顺序为 delay, CH1, CH2, CH3, CH4。

使用29个输入比较了当前C#构建和先前发布版。接受的15个输入生成完全相同的ROM字节，另外14个输入产生相同的拒绝诊断。另将两个ROM各在KUROSAKI中运行4帧，分别检查实体分配、回调、槽位复用，以及有符号运算、KQBody3D 布局和音频队列常量。报告列出18字节RAM的预期值和实际值。这些检查不涉及音频播放或全部API；先前的回归测试结果在 BINARY_BUILD.json 中单独标注。

[源代码示例与运行验证记录](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:zh-CN:end -->


[打开KITAQFC库简体中文手册](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html)

公开库的命名沿用KITAQGB风格：名称简短、体现功能，不加 `kitaqfc_` 前缀。

## 基础功能

- `fc.h` - 汇总各头文件的统一入口。
- `core.h` - 基本类型。
- `intrinsics.h` - 编译器识别的内建操作入口。
- `runtime.h` / `runtime.c` - NMI、OAM影子缓冲区和VRAM队列的运行时支持。
- `system.h` / `system.c` - KITAQGB风格的帧处理与等待函数。
- `debug.h` / `debug.c` - RAM中的小型跟踪与断言记录。

## 图形

- `vram.h` / `vram.c` - KITAQGB风格的VRAM更新入队函数。
- `sprite.h` / `sprite.c` - KITAQGB风格的精灵分配与组合精灵功能。
- `ppu.h` / `ppu.c`
- `ppu_direct.h`
- `vram_queue.h`
- `palette.h` / `palette.c`
- `scroll.h` / `scroll.c`
- `tilemap.h` / `tilemap.c`
- `nametable_asset.h` / `nametable_asset.c`
- `attribute.h` / `attribute.c`
- `metasprite.h` / `metasprite.c`
- `oam.h`
- `oam_fair.h` / `oam_fair_impl.h` - 在保留优先级的同时轮换64个OAM候选项的顺序，详见 [oam_fair.md](oam_fair.md)。

## 游戏结构

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## 音频与外设

- `audio.h` / `audio.c`
- `audio_vblank.h` / `audio_vblank.c` - NMI 音乐驱动，提供七条记录的 BGM 队列、暂停／继续、独立的七条记录 SFX 缓冲区和单次噪声包络。
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c` - 兼容KITAQGB按钮状态API的封装层。
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## 映射器与FDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## 数值计算

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c` - 矩形物体积分、重力、AABB 接触、表面响应，以及可选的 Q5.3 数据类型。与 `fixed.c` 一起编译。
- `physics3d.h` / `physics3d.c` - 不旋转的三维箱体、质量加权反弹和碰撞强度值。与 `fixed.c` 和 `physics2d.c` 一起编译。
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

在FC/NES硬件能够支持的范围内，KITAQGB兼容函数保留简短的功能名称。场景切换、更新与绘制会同步调用已注册的处理函数。回调顺序及重入限制请参阅各 API 条目。
