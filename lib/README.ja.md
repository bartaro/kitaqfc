# FC/NES 標準ライブラリの構成

[English](README.md) | [日本語](README.ja.md) | [Français](README.fr.md) | [Deutsch](README.de.md)

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
- `physics2d.h` / `physics2d.c` - バイト単位のQ5.3形式の型と調整用定数。ピクセル単位の整数位置、1/8ピクセル単位の端数、符号なしの速さ、方向、抵抗を分けて持ちます。時間経過に応じた位置・速度の更新はゲーム側で行います。ヘッダーが物理更新関数を提供するわけではありません。各項目を分けることで、頻繁に実行する移動処理で構造体やポインターをABI経由で渡す負担を避けられます。
- `math_fast.h`
- `math_fixed.h`
- `math_lut.h` / `math_lut.c`

KITAQGB互換の補助関数は、FC/NESのハードウェアで対応できる範囲で、短い機能名を維持しています。コールバック形式のシーン・エンティティ機能は、現在は状態を保持するだけで、ユーザーの関数ポインターを間接呼び出ししません。
