# OAM_FairDraw

64候補の開始位置を1回の呼び出しごとに13ずつ進め、OAMに既に配置された重要なスプライトの後ろへ候補を追加するライブラリーです。13と64は互いに素なので、64回で全候補が先頭になります。コンパイラーintrinsicの追加は不要です。

```c
#pragma fixed_bank 0
#include "oam_fair_impl.h" /* 1つの翻訳単位だけに含める */
__location(0x0200) u8 oam_fair_shadow[256];
u8 oam_fair_x[64];
u8 oam_fair_y[64];
u8 oam_fair_active[64];
u8 oam_fair_used;
```

各フレームでOAM shadowを非表示Y座標で初期化し、自機などを先に書きます。`oam_fair_used` は使用済みバイト数で、4の倍数にしてください。配列のX/Yは8×8スプライトの中心座標、activeが0なら非表示です。

```c
oam_fair_used = 16; /* 例: 自機4枚を既に配置 */
oam_fair_limit = 32;
oam_fair_tile = 40;
oam_fair_attr = 2;
OAM_FairDraw();
/* 完成したshadowをVBlankでDMA転送 */
```

`oam_fair_drawn` に追加枚数を返します。`oam_fair_phase` は初期化時に0へ設定してください。asmのシンボルは固定名なので、上記5つの配列／カーソルを定義します。ライブラリー自身がphase、limit、tile、attr、drawnを定義します。

実装はカーソルが252へ達すると停止し、最後のスロットを予約します。呼び出しはメインループで行い、処理途中のshadowをNMIからDMAしないでください。

これはファミコンの走査線あたり8枚制限を解除する機能ではありません。混雑時の非表示候補を毎フレーム変える処理です。異なるタイルや属性の集団は既存の描画処理と組み合わせてください。動作例と入力による検証は `../../../azure_requiem_fc` にあります。
