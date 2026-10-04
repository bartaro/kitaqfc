# FC/NES 標準ライブラリの構成

[简体中文](README.zh-CN.md) · [简体中文 HTML](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html)



<!-- fc-current-20261004:ja:start -->
### コンパイラとライブラリの現行配布構成 — 2026年10月4日

fc.h は physics3d と audio_vblank の宣言を含みます。必要な実装ファイルは別途リンクしてください。entity_update_all と entity_draw_all は、使用中のスロットを ID の昇順でコールバックへ渡します。コールバックは共通バンク0に配置するか、呼び出し中は該当 PRG バンクを維持してください。ZX0 を使う場合は zx0.h を明示的にインクルードします。audio_vblank のレコード順は delay, CH1, CH2, CH3, CH4 です。

現行C#ビルドと以前の配布版を29入力で比較しました。受理した15入力の生成ROMはバイト単位で一致し、受理しなかった14入力のエラー診断も一致しました。さらに2本のROMをKUROSAKIで4フレーム実行し、一方でエンティティの割り当て、コールバック、スロット再利用を、他方で符号付き演算、KQBody3D の配置、オーディオキュー定数を確認しました。検証記録にはRAMの期待値18バイトと実際の値を収録しています。この検証は音声再生や全APIを対象としておらず、以前の回帰試験結果は BINARY_BUILD.json で区別しています。

[ソース例と実行検証記録](https://bartaro.github.io/kitaq-docs/fc-library.html#fc-current-20261004-heading)
<!-- fc-current-20261004:ja:end -->


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
- `audio_vblank.h` / `audio_vblank.c` — 7レコードのBGMキュー、一時停止・再開、独立した7レコードのSFXバッファ、1回だけ減衰するノイズ音色を備えたNMI音楽ドライバーです。[各APIと録音付きサンプル](https://bartaro.github.io/kitaq-docs/fc-library.html#module-audio_vblank)
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

<!-- readme-language-links:start -->
[English](README.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md)
<!-- readme-language-links:end -->

<!-- manual-language-links:start -->
| Language / 言語 | HTML |
| --- | --- |
| English | [fc-library](https://bartaro.github.io/kitaq-docs/en/fc-library.html) |
| 日本語 | [fc-library](https://bartaro.github.io/kitaq-docs/fc-library.html) |
| 한국어 | [fc-library](https://bartaro.github.io/kitaq-docs/ko/fc-library.html) |
| 简体中文 | [fc-library](https://bartaro.github.io/kitaq-docs/zh-CN/fc-library.html) |
| 繁體中文 | [fc-library](https://bartaro.github.io/kitaq-docs/zh-TW/fc-library.html) |
| Français | [fc-library](https://bartaro.github.io/kitaq-docs/fr/fc-library.html) |
| Español | [fc-library](https://bartaro.github.io/kitaq-docs/es/fc-library.html) |
| Deutsch | [fc-library](https://bartaro.github.io/kitaq-docs/de/fc-library.html) |
<!-- manual-language-links:end -->

<!-- rust-native-20261004:start -->
## Rust製コンパイラでのライブラリ利用

lib/はゲーム機のROMに組み込むCヘッダーとCソースです。PC用コンパイラと補助ツールはRust製で、ゲームのソース、対象機のライブラリとAPIはCのまま使用します。収録するSOURCE_MANIFEST.jsonとLIBRARY_MERGE.jsonにはライブラリの選択元を記録しています。

Windows・Linux・macOS ARM・macOS Intelでネイティブビルドと実行検証が成功しています。KITAQGBは各環境48件のテストと395件の補助ツール検証、KITAQFCは55件と401件が成功しました。Rust 1.85でも確認済みです。PUBLIC_DISTRIBUTION.jsonに配置済みバイナリのハッシュと検証の出典を記録しています。公開GitHub Actionsでもこのソースを独立してビルド・検証します。

保存済みの比較データでROMのバイト列、診断と補助ツールの形式を検証しています。以前のC#版によるエミュレータ検証は、そのソース指紋に対応する過去の記録として残しています。これだけでRust版の全API、実機動作、FDSのBIOS経由のゲーム起動を保証するものではありません。元のPNG変換スクリプトは入手できないため仕様から再実装し、元スクリプトとのバイト一致は確認できません。

[Rust製コンパイラと補助ツール](../tools/README.ja.md)

<!-- rust-native-20261004:end -->
