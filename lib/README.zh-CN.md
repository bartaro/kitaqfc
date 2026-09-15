# FC/NES标准库结构

[English](README.md) | [日本語](README.ja.md) | **简体中文**

打开KITAQFC库简体中文手册

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
- `physics2d.h` / `physics2d.c` - 字节大小的Q5.3类型及调节常量，分别保存整数像素位置、1/8像素的小数部分、无符号速率、方向和阻力。位置与速度随时间的更新由游戏代码计算；头文件不提供推进物理模拟的函数。分开保存这些字段，可避免高频移动循环通过ABI传递聚合类型或指针的开销。
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

在FC/NES硬件能够支持的范围内，KITAQGB兼容函数保留简短的功能名称。当前采用回调形式的场景与实体功能只保存状态，不会间接调用用户的函数指针。
