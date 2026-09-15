# FC／NES 標準程式庫結構

[English](README.md) | [日本語](README.ja.md) | **繁體中文**

開啟 KITAQFC 程式庫繁體中文手冊

公開程式庫沿用 KITAQGB 的命名方式：名稱簡短、能表達用途，不加上 `kitaqfc_` 前綴。

## 基礎功能

- `fc.h` — 集合各標頭檔的統一入口。
- `core.h` — 基本型別。
- `intrinsics.h` — 編譯器可辨識的內建操作。
- `runtime.h` / `runtime.c` — NMI、OAM 影子緩衝區與 VRAM 佇列的執行階段支援。
- `system.h` / `system.c` — KITAQGB 風格的影格處理與等待函式。
- `debug.h` / `debug.c` — 在 RAM 中記錄小型追蹤資料與斷言結果。

## 圖形

- `vram.h` / `vram.c` — KITAQGB 風格的 VRAM 更新排入佇列函式。
- `sprite.h` / `sprite.c` — KITAQGB 風格的精靈配置與組合精靈功能。
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
- `oam_fair.h` / `oam_fair_impl.h` — 保留優先順序，同時輪替 64 個 OAM 候選項目的排列；詳見 [oam_fair.md](oam_fair.md)。

## 遊戲架構

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## 音訊與周邊裝置

- `audio.h` / `audio.c`
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c` — 與 KITAQGB 按鍵狀態 API 相容的包裝層。
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## 記憶體映射器與 FDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## 數值運算

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c` — 提供位元組大小的 Q5.3 型別與調整常數，分別儲存整數像素位置、1/8 像素的小數部分、無號速率、方向及阻力。位置與速度的逐步更新由遊戲程式計算，標頭檔沒有提供推進物理模擬的函式。分開保存這些欄位，可讓頻繁執行的移動迴圈避免透過 ABI 傳遞聚合型別或指標的成本。
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`
