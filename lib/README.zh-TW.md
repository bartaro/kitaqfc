# FC／NES 標準程式庫結構

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | **繁體中文** | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md) | [Português (Brasil)](README.pt-BR.md)
<!-- readme-language-links:end -->

<!-- fc-current-20261004:zh-TW:start -->
### 目前編譯器與程式庫的發行配置 — 2026年10月4日

fc.h 包含 physics3d 和 audio_vblank 的宣告，所需實作檔案仍須另外連結。entity_update_all 和 entity_draw_all 依 ID 遞增順序將使用中的槽位傳給回呼。請將回呼放在共用 bank 0，或在呼叫期間保持對應 PRG bank 的映射。使用 ZX0 時須明確引入 zx0.h。audio_vblank 的紀錄順序為 delay, CH1, CH2, CH3, CH4。

使用29個輸入比較目前C#建置與先前發行版。接受的15個輸入產生完全相同的ROM位元組，另外14個輸入產生相同的拒絕診斷。另外將兩個ROM各在KUROSAKI執行4影格，分別檢查實體配置、回呼、槽位重用，以及有號運算、KQBody3D 配置和音訊佇列常數。報告列出18位元組RAM的預期值與實際值。這些檢查未涵蓋音訊播放或全部API；先前的回歸測試結果在 BINARY_BUILD.json 中分開標示。

[原始碼範例與執行驗證紀錄](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:zh-TW:end -->


**[KITAQFC · HTML 手冊](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html)**

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
- `audio_vblank.h` / `audio_vblank.c` — NMI 音樂驅動程式，提供七筆紀錄的 BGM 佇列、暫停／繼續、獨立的七筆紀錄 SFX 緩衝區，以及單次觸發的雜訊衰減包絡。 [HTML 手冊](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html#module-audio_vblank)
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
- `physics2d.h` / `physics2d.c` — 提供矩形剛體積分、重力、AABB 接觸、表面反應，以及可選用的 Q5.3 資料型別。請搭配 `fixed.c` 編譯。
- `physics3d.h` / `physics3d.c` — 提供不旋轉的三維箱體、依質量分配的反彈反應及碰撞強度值。請搭配 `fixed.c` 與 `physics2d.c` 編譯。
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

場景切換、更新與繪製會同步呼叫已註冊的處理函式。回呼順序與重入限制請參閱各 API 項目。
