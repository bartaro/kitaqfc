# FC/NES 標準ライブラリの構成

[English](README.md) | [日本語](README.ja.md)

[日本語のKITAQFCライブラリ説明書](https://bartaro.github.io/kitaq-docs/fc-library.html)から、各関数の使い方とサンプルコードを直接参照できます。

公開ライブラリの名前はKITAQGBに合わせ、`kitaqfc_` の接頭辞を付けず、機能を表す短い名前にしています。

## 基本機能

- `fc.h` - 共通ヘッダーをまとめて読み込む入口。
- `core.h` - 基本の型。
- `intrinsics.h` - コンパイラが認識する組み込み命令の入口。
- `runtime.h` / `runtime.c` - NMI、OAMの作業用コピー、VRAMキューの実行時処理。
- `system.h` / `system.c` - KITAQGBと同じ形式のフレーム・待機関数。
- `debug.h` / `debug.c` - RAM上の小さなトレース・アサート記録。

## 画像表示

- `vram.h` / `vram.c` - KITAQGBと同じ形式でVRAM更新をキューに予約する関数。
- `sprite.h` / `sprite.c` - KITAQGBと同じ形式のスプライト割り当てとメタスプライト。
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
- `oam_fair.h` / `oam_fair_impl.h` - 優先順位を保ちながら64候補のOAM順を入れ替える処理。詳しくは [oam_fair.md](oam_fair.md) を参照してください。

## ゲームの構成

- `scene.h` / `scene.c`
- `actor.h` / `actor.c`
- `entity.h` / `entity.c`
- `chain.h` / `chain.c`
- `collision.h` / `collision.c`

## 音と周辺機器

- `audio.h` / `audio.c`
- `fds_sound.h`
- `vrc6_sound.h` / `vrc6_sound.c`
- `vrc7_sound.h` / `vrc7_sound.c`
- `pad.h` / `pad.c`
- `input.h` / `input.c` - KITAQGBのボタン状態APIに合わせるラッパー。
- `input_repeat.h` / `input_repeat.c`
- `zapper.h`
- `keyboard.h`
- `rob.h`
- `mic.h`
- `midi.h`

## マッパーとFDS

- `mapper.h`
- `bank.h` / `bank.c`
- `asset.h` / `asset.c`
- `fds.h`
- `fds_file.h`
- `fds_overlay.h`
- `fds_save.h`

## 数値計算

- `fixed.h` / `fixed.c`
- `physics2d.h` / `physics2d.c` - 矩形bodyの積分・重力・AABB接触・面応答と、独自の移動に使うQ5.3型。`fixed.c`と合わせてコンパイルします。
- `physics3d.h` / `physics3d.c` - 回転しない3Dの箱、質量を考慮した反発、衝撃値。`fixed.c`と`physics2d.c`もコンパイルします。
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

シーンの切り替え・更新・描画は、登録したコールバックを同期的に呼びます。呼び出し順と再入制約は各APIの説明を確認してください。
